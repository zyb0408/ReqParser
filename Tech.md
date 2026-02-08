
# 🛠️ ReqParser 技术说明书 (Technical Specification)

## 1. 技术栈选型 (Technology Stack)

| 模块 | 选型 | 理由 |
| --- | --- | --- |
| **容器外壳** | **Tauri 2.0** | 内存占用极低（约 20MB），安装包小（< 5MB），完美支持跨平台，安全性高。 |
| **后端逻辑** | **Rust** | 处理高性能正则匹配、系统剪贴板监听（Clipboard Observer）及跨平台系统调用。 |
| **前端框架** | **React + TypeScript** | 极速构建，响应式数据绑定适合展示复杂的 K-V 树形结构。 |
| **UI 样式** | **Tailwind CSS + Shadcn Vue** | 快速实现现代感、开发者友好的 UI 界面。 |
| **数据持久化** | **Local Storage / JSON File** | 仅用于存储历史记录及用户自定义词典，无需数据库。 |

---

## 2. 核心架构设计 (System Architecture)

### 2.1 数据流向

1. **输入层 (Input):** Rust 层通过线程循环监听系统剪贴板（使用 `arboard` 或 `tauri-plugin-clipboard`）。
2. **触发层 (Trigger):** 当匹配到符合 HTTP 特征的字符串（如以 `GET/POST` 开头或包含多个 `: ` 行），通过 `Tauri Event` 发送到前端。
3. **解析层 (Parser):**
* **一级拆解:** 将原始文本按行切分，识别 Header、Query 和 Body。
* **二级拆解 (Recursive):** 对 `Cookie`、`Set-Cookie` 或 `x-www-form-urlencoded` 进行递归解析。


4. **展示层 (Presentation):** 前端渲染 K-V 表格，并关联内置的解释词典。

---

## 3. 核心功能实现细节 (Core Implementation)

### 3.1 智能解析引擎逻辑 (Rust/TS)

解析算法需要处理多种模糊场景。

* **正则匹配模式:**
* Header 行识别: `^([\w-]+):\s*(.*)$`
* JWT 识别: `^([a-zA-Z0-9_-]+)\.([a-zA-Z0-9_-]+)\.([a-zA-Z0-9_-]+)$`


* **递归逻辑 (Recursive Parsing):**
```typescript
interface ParseNode {
  key: string;
  value: string;
  children?: ParseNode[]; // 用于存放嵌套解析结果
  description?: string;   // 来自内置词典的解释
}

```


当检测到 Value 包含 `;` 且内部存在多个 `=` 时，触发二级解析函数，将结果作为 `children` 挂载。

### 3.2 内置百科实现 (The Explainer)

维护一个本地静态的 `dictionary.json`：

```json
{
  "Cache-Control": {
    "desc": "控制缓存的行为",
    "values": {
      "no-cache": "强制发送请求给服务器进行验证",
      "max-age": "资源能够被缓存的最长时间（秒）"
    }
  }
}

```

**实现方式：** 渲染 K-V 时，通过 `Key` 在 JSON 中进行  查询。

### 3.3 剪贴板监听 (Rust 侧)

利用 Rust 的 `tokio` 运行时开启一个异步任务：

```rust
tauri::async_runtime::spawn(async move {
    let mut last_clipboard = String::new();
    loop {
        if let Ok(content) = clipboard.get_text() {
            if content != last_clipboard && is_http_like(&content) {
                app_handle.emit("clipboard-change", &content).unwrap();
                last_clipboard = content;
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
});

```

---

## 4. 关键 UI 组件设计

1. **K-V Tree Table:**
* 支持“展开/收起”嵌套的 Cookie 或 JSON。
* Hover 展示字段详细描述浮窗。


2. **Quick Action Bar:**
* **Base64 Decode:** 一键切换字段值的编码状态。
* **Time Transformer:** 自动识别并转换时间戳。


3. **Side Panel:**
* 展示选中字段的 MDN 文档简略版和可能的安全风险提示（例如 Cookie 缺少 `HttpOnly` 标志）。



---

## 5. 跨平台适配方案

* **macOS:** 支持毛玻璃效果（Vibrancy），集成在 Menubar 运行。
* **Windows:** 优化字体显示（使用微软雅黑或 Segoe UI），支持任务栏图标最小化。
* **Linux:** 通过 AppImage 分发，适配 GTK 环境。

---

## 6. 安全性规范 (Privacy & Security)

* **本地加密:** 历史记录（可选）加密存储。
* **网络隔离:** 在 `tauri.conf.json` 中配置严格的 `allowlist`，除了可能的软件更新检测，**禁止任何业务数据外流**。
* **敏感信息脱敏:** 前端实现“遮罩逻辑”，根据 Key 的黑名单（Token, Key, Secret, Auth）默认对 Value 进行模糊处理。

---

## 7. 下一步开发计划 (Roadmap)

1. **Phase 1 (Rust Backend):** 实现稳定、低占用的剪贴板监听器。
2. **Phase 2 (Parser Engine):** 完成 Header 与 Cookie 的递归解析算法。
3. **Phase 3 (UI/UX):** 基于 Tailwind 构建响应式 K-V 视图。
4. **Phase 4 (Intelligence):** 完善内置词典及 JWT/时间戳自动转换功能。

---
