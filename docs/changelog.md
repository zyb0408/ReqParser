# ReqParser 变更日志

所有显著变更都记录在此文件中。格式基于 [Keep a Changelog](https://keepachangelog.com/)。

---

## v0.1.0 (Phase 1) -- 基础架构

### 新增

#### Rust 后端

- **三格式解析引擎**：
  - `parser.rs` -- 原始 HTTP 请求/响应文本解析
  - `curl_parser.rs` -- 浏览器 "Copy as cURL" 格式解析
  - `fetch_parser.rs` -- 浏览器 "Copy as fetch" 格式解析
- **输入格式自动检测**（`detector.rs`）：
  - 基于前缀匹配识别 cURL 和 fetch 格式
  - 基于正则表达式识别原始 HTTP 请求行、响应行和 Header 行
  - `is_http_like()` 统一入口，支持所有格式
- **剪贴板异步监听**（`clipboard.rs`）：
  - 使用 `arboard` + `spawn_blocking` 实现跨平台剪贴板访问
  - 500ms 轮询间隔
  - 通过 `AtomicBool` 实现开关控制
  - 检测到 HTTP 文本时通过 Tauri Event 通知前端
- **核心数据模型**（`models.rs`）：
  - `ParseNode` -- 递归键值树节点
  - `ParseResult` -- 完整解析结果
  - `HttpContentType` -- 内容类型枚举
  - 所有模型使用 `camelCase` JSON 序列化
- **错误处理**（`error.rs`）：
  - `AppError` 枚举（ParseError、ClipboardError、InternalError）
  - `thiserror` 派生 + 手动 `Serialize` 实现
- **4 个 Tauri 命令**：
  - `parse_text` -- 自动检测格式并解析
  - `check_http_like` -- 文本 HTTP 特征检测
  - `toggle_clipboard_watcher` -- 剪贴板监听开关
  - `get_clipboard_watcher_status` -- 查询监听状态

#### 解析能力

- HTTP 请求行解析（支持 GET/POST/PUT/DELETE/PATCH/HEAD/OPTIONS/TRACE/CONNECT）
- HTTP 响应行解析（状态码 + 状态文本）
- Header K-V 提取
- URL Query Parameters 自动解析（自动 URL 解码）
- Cookie / Set-Cookie 按 `;` 拆解为子节点
- Cookie 标志属性（HttpOnly、Secure 等）识别
- cURL 反斜杠续行合并
- cURL Shell token 化（单引号、双引号、`$'...'` ANSI-C quoting）
- cURL `-H`/`-X`/`-d`/`-b` 等常用标志处理
- cURL 无参数标志（`--compressed`、`-L`、`-k` 等）自动忽略
- fetch `options` JSON 解析（method、headers、body）
- Body 通过空行分隔提取

#### 测试

- 50 个 Rust 单元测试覆盖所有解析器和检测器
- 包含真实世界样例测试用例

#### 前端

- 最小测试 UI（`App.tsx`）
  - 文本输入框
  - 解析按钮
  - 剪贴板监听开关
  - JSON 格式结果展示
- Tailwind CSS v4 + Shadcn/ui 主题体系
- 亮色/暗色主题 CSS 变量

#### 基础设施

- 跨平台 CI/CD workflow（`.github/workflows/publish.yml`）
  - macOS（Apple Silicon + Intel）
  - Ubuntu（x86_64 + ARM）
  - Windows
  - Tag-based 自动发布
  - Rust 缓存加速
- Vite 开发服务器配置（端口 1420，HMR 支持）
- TypeScript 严格模式配置
- Shadcn/ui 初始化配置（new-york 风格）

### 技术决策

- 使用 `LazyLock<Regex>` 替代 `lazy_static!`（Rust 1.80+ 标准库原生支持）
- 使用 `spawn_blocking` 包装 `arboard::Clipboard`（macOS 上不是 Send）
- 使用 `thiserror` + 手动 `Serialize` impl 处理 Tauri 命令错误
- 解析器设计为纯函数，便于测试
