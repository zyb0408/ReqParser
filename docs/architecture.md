# ReqParser 系统架构文档

## 目录

- [1. 项目概述](#1-项目概述)
- [2. 技术栈](#2-技术栈)
- [3. 整体架构](#3-整体架构)
- [4. 数据流向](#4-数据流向)
- [5. Rust 模块职责](#5-rust-模块职责)
- [6. Tauri IPC 通信机制](#6-tauri-ipc-通信机制)
- [7. 前端组件架构](#7-前端组件架构)

---

## 1. 项目概述

ReqParser 是一款轻量级、跨平台的桌面工具，用于自动解析、解码并解释 HTTP 请求/响应的 Header、Payload 和 Response Data。核心痛点是解决开发者在调试时面对冗长、编码过的字符串（如 Cookie、JWT、URL Params）难以阅读和理解字段含义的问题。

目标平台：macOS、Windows、Linux。

---

## 2. 技术栈

| 层级 | 选型 | 说明 |
|------|------|------|
| 容器外壳 | **Tauri 2.0** | 内存占用低（约 20MB），安装包小，跨平台支持 |
| 后端逻辑 | **Rust** | 高性能正则匹配、系统剪贴板监听、文本解析 |
| 前端框架 | **React 19 + TypeScript 5.8** | 响应式数据绑定，适合展示 K-V 树形结构 |
| UI 样式 | **Tailwind CSS 4 + Shadcn/ui** (new-york 风格) | 现代感、开发者友好的界面 |
| 构建工具 | **Vite 7** | 快速的前端构建和 HMR |
| 包管理 | **pnpm** | 前端依赖管理 |
| CI/CD | **GitHub Actions + tauri-action** | 跨平台自动构建和发布 |

### Rust 依赖（Cargo.toml）

| crate | 版本 | 用途 |
|-------|------|------|
| `tauri` | 2 | Tauri 框架核心 |
| `serde` / `serde_json` | 1 | JSON 序列化/反序列化 |
| `arboard` | 3 | 跨平台剪贴板访问 |
| `regex` | 1 | 正则表达式匹配 |
| `tokio` | 1 (features: time) | 异步运行时定时器 |
| `thiserror` | 2 | 错误类型派生宏 |
| `url` | 2 | URL 解析 |
| `base64` | 0.22 | Base64 编解码 |
| `chrono` | 0.4 | 时间处理 |

### 前端依赖（package.json）

| 包 | 用途 |
|----|------|
| `@tauri-apps/api` | Tauri IPC 调用 |
| `react` / `react-dom` | UI 框架 |
| `tailwindcss` / `@tailwindcss/vite` | 样式系统 |
| `shadcn` / `class-variance-authority` / `clsx` / `tailwind-merge` | Shadcn/ui 组件库 |
| `lucide-react` | 图标库 |
| `tw-animate-css` | 动画 |

---

## 3. 整体架构

```
+---------------------------------------------------------------+
|                        ReqParser 桌面应用                       |
+---------------------------------------------------------------+
|                                                               |
|  +---------------------------+   +-------------------------+  |
|  |       前端 (WebView)       |   |    Rust 后端 (Tauri)     |  |
|  |                           |   |                         |  |
|  |  +---------------------+  |   |  +-------------------+  |  |
|  |  |     App.tsx         |  |   |  |    lib.rs         |  |  |
|  |  |  (React 入口组件)    |  |   |  |  (Tauri 命令注册)  |  |  |
|  |  +---------------------+  |   |  +-------------------+  |  |
|  |           |                |   |         |               |  |
|  |  invoke() / listen()      |   |  +------+------+        |  |
|  |           |                |   |  |      |      |        |  |
|  |  +--------+--------+      |   |  |  detector   |        |  |
|  |  |  Tauri IPC 桥    |<--->|---|->|  parser     |        |  |
|  |  +-----------------+      |   |  |  curl_parser|        |  |
|  |                           |   |  |  fetch_parser        |  |
|  |  +---------------------+  |   |  |  clipboard  |        |  |
|  |  |  Tailwind + Shadcn  |  |   |  |  models     |        |  |
|  |  |  (样式 + 组件)      |  |   |  |  error      |        |  |
|  |  +---------------------+  |   |  +-------------+        |  |
|  +---------------------------+   +-------------------------+  |
+---------------------------------------------------------------+
```

---

## 4. 数据流向

```
输入文本
   |
   v
+------------------+     +--------------------+     +------------------+
|   输入层 (Input)   | --> |  格式检测 (Detect)   | --> |  解析层 (Parse)   |
|                  |     |                    |     |                  |
| - 用户手动粘贴    |     | detector.rs        |     | - parser.rs      |
| - 剪贴板自动监听  |     | detect_input_format |     | - curl_parser.rs |
+------------------+     +--------------------+     | - fetch_parser.rs|
                                |                    +------------------+
                                |                           |
                         InputFormat:                       v
                         - Curl                    +------------------+
                         - Fetch                   |  结构化结果       |
                         - RawHttp                 |  ParseResult     |
                         - Unknown                 +------------------+
                                                           |
                                                           v
                                                   +------------------+
                                                   |  前端渲染         |
                                                   |  K-V 表格展示     |
                                                   +------------------+
```

### 详细流程

1. **输入层**：用户在前端文本框粘贴 HTTP 文本，或通过剪贴板监听自动捕获
2. **格式检测**：`detector.rs` 中的 `detect_input_format()` 使用正则和前缀匹配判断输入格式
   - `curl ` 开头 -> `InputFormat::Curl`
   - `fetch(` 开头 -> `InputFormat::Fetch`
   - 符合 HTTP 请求/响应行特征或包含多个 Header 行 -> `InputFormat::RawHttp`
   - 其他 -> `InputFormat::Unknown`（fallback 到 RawHttp 解析器）
3. **解析层**：根据格式分发到对应的解析器
   - `parser::parse_http_text()` -- 原始 HTTP 文本
   - `curl_parser::parse_curl()` -- cURL 命令
   - `fetch_parser::parse_fetch()` -- fetch() 调用
4. **子解析**：Header 值进行递归子解析（如 Cookie 按 `;` 拆解为子节点）
5. **结果输出**：返回 `ParseResult` 结构体，包含 method、url、headers、query_params、body 等字段
6. **前端渲染**：React 组件接收 JSON 结果并渲染为可交互的界面

---

## 5. Rust 模块职责

### 5.1 detector.rs -- 输入格式检测

负责自动识别输入文本的格式类型。

- **核心函数**：
  - `detect_input_format(text: &str) -> InputFormat`：自动检测文本格式
  - `is_http_like(text: &str) -> bool`：判断文本是否像 HTTP 数据（用于剪贴板监听）
- **检测逻辑**：使用 `LazyLock<Regex>` 预编译的正则表达式匹配 HTTP 请求行、响应行和 Header 行
- **InputFormat 枚举**：`Curl`、`Fetch`、`RawHttp`、`Unknown`

### 5.2 parser.rs -- 原始 HTTP 解析器

解析标准 HTTP 请求/响应文本。

- **核心函数**：`parse_http_text(raw: &str) -> ParseResult`
- **处理能力**：
  - 识别请求行（`GET /path HTTP/1.1`）和响应行（`HTTP/1.1 200 OK`）
  - 提取所有 Header 为 K-V 结构
  - 通过空行分离 Header 和 Body
  - 从 URL 解析 Query Parameters
  - Cookie/Set-Cookie 值的子节点解析

### 5.3 curl_parser.rs -- cURL 命令解析器

解析浏览器 DevTools "Copy as cURL" 格式的文本。

- **核心函数**：`parse_curl(input: &str) -> ParseResult`
- **处理能力**：
  - 合并反斜杠续行为单行
  - Shell 风格 token 化（处理单引号、双引号、`$'...'` ANSI-C quoting）
  - 识别 `-H`/`--header`、`-X`/`--request`、`-d`/`--data`/`--data-raw`/`--data-binary`/`--data-urlencode`、`-b`/`--cookie` 等标志
  - 智能忽略 `--compressed`、`-L`、`-k` 等无参数标志
  - 根据是否有 `-d` 推断默认 method（有 data 则 POST，否则 GET）

### 5.4 fetch_parser.rs -- fetch() 调用解析器

解析浏览器 DevTools "Copy as fetch" 格式的文本。

- **核心函数**：`parse_fetch(input: &str) -> ParseResult`
- **处理能力**：
  - 提取 `fetch(...)` 内部的 URL 和 options 对象
  - 使用 `serde_json` 解析 options JSON
  - 从 JSON 中提取 method、headers、body
  - Cookie header 的子节点解析
  - 支持末尾有无分号

### 5.5 clipboard.rs -- 剪贴板异步监听

后台监听系统剪贴板变化，自动检测 HTTP 文本。

- **核心结构**：`ClipboardWatcherState`（使用 `AtomicBool` 存储启用状态）
- **核心函数**：`start_clipboard_watcher(app_handle, state)` -- 在 app setup 时调用一次
- **实现细节**：
  - 使用 `tauri::async_runtime::spawn` 启动异步循环
  - 使用 `spawn_blocking` 包装 `arboard::Clipboard` 操作（因为 macOS 上不是 Send）
  - 每 500ms 轮询一次剪贴板
  - 内容变化且符合 HTTP 特征时，通过 `app_handle.emit("clipboard-http-detected", &content)` 发送事件到前端
  - 通过 `AtomicBool` 实现开关控制，无需停止/重启异步任务

### 5.6 models.rs -- 核心数据结构

定义所有解析结果的数据模型。

- **ParseNode**：解析树的单个键值节点，支持子节点（递归结构）
- **ParseResult**：解析引擎返回的完整结果
- **HttpContentType**：枚举（Request、Response、HeadersOnly、Unknown）
- 所有模型使用 `#[serde(rename_all = "camelCase")]` 确保 JSON 键名符合 JavaScript 惯例

### 5.7 error.rs -- 错误类型

定义应用级别的错误类型。

- **AppError 枚举**：`ParseError`、`ClipboardError`、`InternalError`
- 使用 `thiserror` 派生 `Error` trait
- 手动实现 `Serialize`，将错误序列化为字符串（满足 Tauri 命令返回值要求）

---

## 6. Tauri IPC 通信机制

### 6.1 命令调用（invoke）

前端通过 `@tauri-apps/api/core` 的 `invoke()` 函数调用 Rust 后端命令：

```typescript
// 前端调用示例
const result = await invoke<ParseResult>("parse_text", { rawText: text });
const isHttp = await invoke<boolean>("check_http_like", { text: content });
const enabled = await invoke<boolean>("toggle_clipboard_watcher", {});
const status = await invoke<boolean>("get_clipboard_watcher_status");
```

Rust 端通过 `#[tauri::command]` 宏注册命令，并在 `invoke_handler` 中统一管理：

```rust
.invoke_handler(tauri::generate_handler![
    parse_text,
    check_http_like,
    toggle_clipboard_watcher,
    get_clipboard_watcher_status,
])
```

### 6.2 事件通信（emit / listen）

剪贴板监听使用 Tauri 事件系统进行后端到前端的异步通信：

- **后端发送**：`app_handle.emit("clipboard-http-detected", &content)`
- **前端监听**：`listen<string>("clipboard-http-detected", callback)`

### 6.3 状态管理（State）

使用 Tauri 的 `manage()` + `State<>` 在命令间共享状态：

```rust
// 注册状态
.manage(watcher_state.clone())

// 在命令中使用
fn toggle_clipboard_watcher(state: State<'_, Arc<ClipboardWatcherState>>, ...) -> bool
```

---

## 7. 前端组件架构

### 当前状态（Phase 1）

目前为最小测试 UI，所有逻辑集中在 `App.tsx` 单个组件中：

```
App.tsx
├── 剪贴板监听开关按钮
├── 文本输入区 (textarea)
├── 解析按钮
├── 错误提示
└── 结果展示区
    ├── 类型/方法/状态/协议信息
    ├── URL 显示
    ├── Headers 列表 (JSON 预览)
    ├── Query Params (JSON 预览)
    └── Body 预览
```

### 样式系统

- 使用 Tailwind CSS v4，通过 `@tailwindcss/vite` 插件集成
- 使用 Shadcn/ui 的 CSS 变量体系（oklch 色彩空间）
- 支持亮色/暗色主题（通过 `.dark` 类切换）
- Vite 配置了 `@` 路径别名指向 `src/` 目录
