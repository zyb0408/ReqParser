# ReqParser 前端组件文档

## 目录

- [1. 当前组件](#1-当前组件)
- [2. 计划中的组件](#2-计划中的组件)

---

## 1. 当前组件

### 1.1 App.tsx -- 应用入口组件

Phase 1 的最小测试 UI，所有功能集中在单个组件中。

**职责：**

- 管理应用状态（输入文本、解析结果、错误信息、监听状态）
- 调用 Tauri 命令进行 HTTP 文本解析
- 监听 `clipboard-http-detected` 事件，自动解析剪贴板内容
- 渲染输入区、结果展示区和剪贴板监听开关

**状态：**

| 状态变量 | 类型 | 说明 |
|---------|------|------|
| `inputText` | `string` | 用户输入的原始文本 |
| `parseResult` | `ParseResult \| null` | 解析结果 |
| `error` | `string \| null` | 错误信息 |
| `watcherEnabled` | `boolean` | 剪贴板监听是否开启 |
| `clipboardEvent` | `string \| null` | 最近一次检测到的剪贴板文本 |

**方法：**

| 方法 | 说明 |
|------|------|
| `handleParse(text?)` | 调用 `parse_text` 命令解析文本；不传参时使用 `inputText` |
| `handleToggleWatcher()` | 调用 `toggle_clipboard_watcher` 切换剪贴板监听 |

**UI 结构：**

```
<main>
  ├── <h1> 标题
  ├── 剪贴板监听开关按钮 + 检测事件提示
  ├── <textarea> 原始文本输入
  ├── Parse 按钮
  ├── 错误提示 (条件渲染)
  └── 结果展示区 (条件渲染)
      ├── 类型/方法/状态码/协议 信息行
      ├── URL 显示
      ├── Headers JSON 预览
      ├── Query Params JSON 预览
      └── Body 预览
```

---

## 2. 计划中的组件

基于 Req.md 需求文档，后续版本计划实现以下组件。

### 2.1 布局组件

| 组件 | 用途 |
|------|------|
| **TopToolbar** | 顶部工具栏：粘贴按钮、清空按钮、监听开关、Diff 模式切换、截图脱敏开关 |
| **LeftPanel** | 左侧面板：原始文本输入区或历史记录列表 |
| **MainContent** | 中间主展示区：K-V 表格 |
| **RightDetailPanel** | 右侧详情面板：选中字段的官方定义、MDN 链接和解码内容 |

### 2.2 核心展示组件

| 组件 | 用途 |
|------|------|
| **KVTreeTable** | K-V 树形表格，支持展开/收起嵌套的 Cookie 或 JSON，列包含 Key、Value、Action |
| **TreeNode** | 树形表格中的单行节点，支持子节点展开 |
| **HeaderExplainer** | 显示 HTTP Header 的中文解释和 MDN 文档链接 |
| **ValueDecoder** | 值解码切换器，支持"原始值/美化值"切换 |

### 2.3 功能组件

| 组件 | 用途 |
|------|------|
| **DiffView** | 双窗口对比两段 Header/Payload 的差异，高亮不同点 |
| **CodeExporter** | 将解析的请求导出为 cURL、Fetch、Python、Go 等代码片段 |
| **PrivacyMask** | 敏感数据遮罩，对 Authorization、Token、Set-Cookie 等字段进行脱敏 |
| **ClipboardWatcher** | 剪贴板监听控制组件，显示监听状态和最近检测到的内容 |

### 2.4 通用 UI 组件

| 组件 | 用途 |
|------|------|
| **CopyButton** | 点击复制到剪贴板 |
| **JWTViewer** | JWT 解析展示（Header、Payload、Signature） |
| **TimestampConverter** | 时间戳自动转换为本地时间 |
| **JSONFormatter** | JSON 语法高亮和折叠展示 |
| **Base64Decoder** | Base64 编解码切换 |

### 2.5 Shadcn/ui 组件（已配置，待使用）

项目已初始化 Shadcn/ui（new-york 风格），后续可按需引入：

- Button、Input、Textarea -- 基础表单控件
- Table -- K-V 表格基础
- Tabs -- 多标签页
- Dialog -- 弹窗
- Tooltip -- 字段悬浮提示
- Switch -- 开关控件
- ScrollArea -- 滚动区域
- Separator -- 分隔线
- Badge -- 状态标签（如 method 类型标记）
