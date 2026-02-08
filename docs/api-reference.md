# ReqParser API 参考文档

## 目录

- [1. Tauri 命令](#1-tauri-命令)
- [2. 数据模型](#2-数据模型)
- [3. 事件定义](#3-事件定义)
- [4. 错误类型](#4-错误类型)

---

## 1. Tauri 命令

所有命令通过 `@tauri-apps/api/core` 的 `invoke()` 函数从前端调用。

### 1.1 parse_text

解析 HTTP 文本，自动检测输入格式（cURL / fetch / 原始 HTTP）。

**签名：**

```typescript
invoke<ParseResult>("parse_text", { rawText: string }): Promise<ParseResult>
```

**参数：**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `rawText` | `string` | 是 | 待解析的原始文本 |

**返回值：** `ParseResult` 对象

**错误：**
- 当 `rawText` 为空字符串时，抛出 `ParseError("Input text is empty")`

**示例：**

```typescript
import { invoke } from "@tauri-apps/api/core";

// 解析 cURL 命令
const result = await invoke<ParseResult>("parse_text", {
  rawText: "curl 'https://example.com/api' -H 'accept: application/json'"
});
console.log(result.method);  // "GET"
console.log(result.headers); // [{ key: "accept", value: "application/json", ... }]

// 解析原始 HTTP 请求
const result2 = await invoke<ParseResult>("parse_text", {
  rawText: "GET /api/users?page=1 HTTP/1.1\nHost: example.com"
});
console.log(result2.queryParams); // [{ key: "page", value: "1", ... }]

// 解析 fetch 调用
const result3 = await invoke<ParseResult>("parse_text", {
  rawText: `fetch("https://example.com/api", { "method": "POST", "headers": { "content-type": "application/json" }, "body": "{\\"name\\":\\"test\\"}" });`
});
console.log(result3.body); // '{"name":"test"}'
```

**自动格式检测逻辑：**

| 条件 | 检测结果 | 使用解析器 |
|------|----------|-----------|
| 以 `curl ` 或 `curl\t` 开头 | Curl | `curl_parser::parse_curl` |
| 以 `fetch(` 或 `fetch (` 开头 | Fetch | `fetch_parser::parse_fetch` |
| 首行匹配 HTTP 请求/响应行或 2+ 个 Header 行 | RawHttp | `parser::parse_http_text` |
| 其他 | Unknown | `parser::parse_http_text`（fallback） |

---

### 1.2 check_http_like

检测文本是否像 HTTP 数据。

**签名：**

```typescript
invoke<boolean>("check_http_like", { text: string }): Promise<boolean>
```

**参数：**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `text` | `string` | 是 | 待检测的文本 |

**返回值：** `boolean` -- 文本是否像 HTTP 请求/响应/cURL/fetch 数据

**示例：**

```typescript
await invoke<boolean>("check_http_like", {
  text: "GET /api HTTP/1.1\nHost: example.com"
}); // true

await invoke<boolean>("check_http_like", {
  text: "curl 'https://example.com'"
}); // true

await invoke<boolean>("check_http_like", {
  text: "Hello world"
}); // false
```

---

### 1.3 toggle_clipboard_watcher

开关剪贴板监听。传入 `enabled` 强制设置状态，不传则切换当前状态。

**签名：**

```typescript
invoke<boolean>("toggle_clipboard_watcher", { enabled?: boolean }): Promise<boolean>
```

**参数：**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `enabled` | `boolean` | 否 | 强制设为指定状态；省略时切换 |

**返回值：** `boolean` -- 切换后的最新状态

**示例：**

```typescript
// 切换状态
const newState = await invoke<boolean>("toggle_clipboard_watcher", {});

// 强制开启
const state = await invoke<boolean>("toggle_clipboard_watcher", { enabled: true });

// 强制关闭
const state2 = await invoke<boolean>("toggle_clipboard_watcher", { enabled: false });
```

---

### 1.4 get_clipboard_watcher_status

获取剪贴板监听的当前状态。

**签名：**

```typescript
invoke<boolean>("get_clipboard_watcher_status"): Promise<boolean>
```

**返回值：** `boolean` -- 当前是否正在监听

**示例：**

```typescript
const isWatching = await invoke<boolean>("get_clipboard_watcher_status");
console.log(isWatching); // false (默认关闭)
```

---

## 2. 数据模型

### 2.1 ParseNode

解析树中的单个键值节点。

**Rust 定义（`models.rs`）：**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseNode {
    pub key: String,
    pub value: String,
    pub children: Option<Vec<ParseNode>>,
    pub description: Option<String>,
    pub decoded_value: Option<String>,
    pub value_type: Option<String>,
}
```

**TypeScript 接口：**

```typescript
interface ParseNode {
  key: string;
  value: string;
  children?: ParseNode[];
  description?: string;
  decodedValue?: string;
  valueType?: string;
}
```

**字段说明：**

| 字段 | 类型 | 说明 |
|------|------|------|
| `key` | `string` | 键名，如 `"Content-Type"`、`"session_id"` |
| `value` | `string` | 原始值，如 `"application/json"`、`"abc123"` |
| `children` | `ParseNode[]?` | 子节点，用于 Cookie 等递归解析结果 |
| `description` | `string?` | 来自内置词典的字段解释 |
| `decodedValue` | `string?` | 解码后的值（如 Base64 解码、URL 解码） |
| `valueType` | `string?` | 值类型标记（如 "jwt"、"base64"、"timestamp"） |

**子节点示例：**

Cookie header 会自动拆解为子节点：

```json
{
  "key": "Cookie",
  "value": "session=abc123; theme=dark",
  "children": [
    { "key": "session", "value": "abc123" },
    { "key": "theme", "value": "dark" }
  ]
}
```

Set-Cookie 的标志属性（如 HttpOnly、Secure）作为无值节点：

```json
{
  "key": "Set-Cookie",
  "value": "token=xyz; Path=/; HttpOnly; Secure",
  "children": [
    { "key": "token", "value": "xyz" },
    { "key": "Path", "value": "/" },
    { "key": "HttpOnly", "value": "" },
    { "key": "Secure", "value": "" }
  ]
}
```

---

### 2.2 ParseResult

解析引擎返回的完整结果。

**Rust 定义（`models.rs`）：**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseResult {
    pub content_type: HttpContentType,
    pub method: Option<String>,
    pub url: Option<String>,
    pub status_code: Option<u16>,
    pub status_text: Option<String>,
    pub protocol: Option<String>,
    pub headers: Vec<ParseNode>,
    pub query_params: Option<Vec<ParseNode>>,
    pub body: Option<String>,
    pub raw_text: String,
}
```

**TypeScript 接口：**

```typescript
interface ParseResult {
  contentType: string;     // HttpContentType 枚举值
  method?: string;         // HTTP 方法
  url?: string;            // 请求 URL
  statusCode?: number;     // 响应状态码
  statusText?: string;     // 响应状态文本
  protocol?: string;       // 协议版本，如 "HTTP/1.1"
  headers: ParseNode[];    // Header 列表
  queryParams?: ParseNode[];  // URL 查询参数
  body?: string;           // 请求/响应体
  rawText: string;         // 原始输入文本
}
```

**字段说明：**

| 字段 | 类型 | 说明 |
|------|------|------|
| `contentType` | `HttpContentType` | 内容类型枚举 |
| `method` | `string?` | HTTP 方法（GET、POST 等），仅请求类型有值 |
| `url` | `string?` | 请求 URL 或路径 |
| `statusCode` | `number?` | HTTP 状态码，仅响应类型有值 |
| `statusText` | `string?` | 状态文本（如 "OK"），仅响应类型有值 |
| `protocol` | `string?` | 协议版本（如 "HTTP/1.1"） |
| `headers` | `ParseNode[]` | 所有 Header 节点 |
| `queryParams` | `ParseNode[]?` | URL 查询参数节点 |
| `body` | `string?` | 请求/响应体（空行之后的内容） |
| `rawText` | `string` | 原始输入文本（原样保留） |

---

### 2.3 HttpContentType

检测到的 HTTP 内容类型。

**Rust 定义：**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HttpContentType {
    Request,
    Response,
    HeadersOnly,
    Unknown,
}
```

**JSON 序列化值：** `"request"` / `"response"` / `"headersOnly"` / `"unknown"`

| 值 | 说明 |
|----|------|
| `request` | 检测到 HTTP 请求（有请求行或来自 cURL/fetch） |
| `response` | 检测到 HTTP 响应（有状态行） |
| `headersOnly` | 仅有 Header 行，无法判断是请求还是响应 |
| `unknown` | 无法识别（fetch 解析失败时使用） |

---

### 2.4 InputFormat

输入格式类型（仅在 Rust 内部使用，不序列化到前端）。

**Rust 定义（`detector.rs`）：**

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum InputFormat {
    Curl,
    Fetch,
    RawHttp,
    Unknown,
}
```

---

## 3. 事件定义

### 3.1 clipboard-http-detected

当剪贴板监听器检测到 HTTP 文本时触发。

| 属性 | 值 |
|------|-----|
| 事件名 | `"clipboard-http-detected"` |
| 发送方 | Rust 后端（`clipboard.rs`） |
| 接收方 | 前端（通过 `listen()` 监听） |
| Payload 类型 | `string` |
| Payload 内容 | 剪贴板中检测到的 HTTP 文本原文 |

**前端监听示例：**

```typescript
import { listen } from "@tauri-apps/api/event";

const unlisten = await listen<string>("clipboard-http-detected", (event) => {
  console.log("检测到 HTTP 文本:", event.payload);
  // 自动解析
  handleParse(event.payload);
});

// 清理
unlisten();
```

**触发条件：**

1. 剪贴板监听已开启（`enabled` 为 `true`）
2. 剪贴板内容发生变化（与上次不同）
3. 内容通过 `is_http_like()` 检测为 HTTP 文本

---

## 4. 错误类型

### AppError

应用级错误枚举，所有 Tauri 命令可能返回的错误类型。

**Rust 定义（`error.rs`）：**

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Clipboard error: {0}")]
    ClipboardError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}
```

**序列化行为：** 所有错误变体序列化为其 `Display` 字符串形式（如 `"Parse error: Input text is empty"`）。

前端通过 `catch` 接收错误字符串：

```typescript
try {
  await invoke<ParseResult>("parse_text", { rawText: "" });
} catch (e) {
  console.error(String(e)); // "Parse error: Input text is empty"
}
```

| 变体 | 触发场景 |
|------|---------|
| `ParseError` | 输入文本为空 |
| `ClipboardError` | 剪贴板访问失败（预留） |
| `InternalError` | 内部逻辑错误（预留） |
