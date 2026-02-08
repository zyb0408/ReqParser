
# 🛠️ ReqParser 需求说明书 (Software Requirements Specification)

## 1. 项目概述

* **目标：** 开发一款轻量级、跨平台的桌面工具，用于自动解析、解码并解释 HTTP 请求/响应的 Header、Payload 和 Response Data。
* **核心痛点：** 解决开发者在调试时面对冗长、编码过的字符串（如 Cookie, JWT, URL Params）难以阅读和理解字段含义的问题。
* **目标平台：** macOS, Windows, Linux。

---

## 2. 核心功能需求

### 2.1 智能解析引擎 (Smart Parsing)

* **自动识别：** 粘贴内容后，自动识别是 Request Header、Response Header、Query String 还是 JSON Payload。
* **K-V 结构化展示：** 所有信息以表格或树形结构展示，支持点击 Key 快速复制，点击 Value 切换“原始值/美化值”。
* **递归拆解 (Recursive Deep-Dive)：**
* **Cookie 解析：** 将 `Cookie` 字符串按 `;` 拆解为二级 K-V。
* **复合 Value 解析：** 识别 Value 中包含的 `k1=v1&k2=v2` 或 `k1:v1;k2:v2` 格式并支持展开。


* **内置百科词典：**
* 集成标准 HTTP Header 解释库（如 `Cache-Control` 的各种指令含义）。
* 针对自定义 Header 提供“可能作用”的智能推测。



### 2.2 自动解码与转换 (Auto-Decoder)

* **JWT 识别：** 自动检测 JWT 格式并解密显示 `Header`、`Payload`、`Signature`。
* **时间戳转换：** 识别数字时间戳，自动显示本地时间。
* **URL/Base64 编解码：** 自动对 URL 编码字符或 Base64 编码进行还原展示。
* **JSON 格式化：** 对压缩过的 JSON 进行高亮美化，支持查看嵌套层级。

### 2.3 效率增强功能 (Developer UX)

* **Diff 对比模式：** 支持双窗口对比两段 Header/Payload 的差异，高亮不同点（常用于排查“为什么这个请求通了，那个没通”）。
* **代码片段生成：** 将解析的请求一键导出为 `cURL`、`Fetch`、`Python`、`Go` 等代码。
* **敏感数据遮罩 (Privacy Mask)：** 一键对 `Authorization`、`Token`、`Set-Cookie` 等敏感字段进行脱敏处理，方便截图分享。

---

## 3. 交互与系统集成需求

* **剪贴板监听 (Clipboard Watcher)：**
* 开启后，软件后台运行。一旦剪贴板内容符合 HTTP 文本特征，通过系统托盘或悬浮窗提示“一键解析”。


* **窗口管理：**
* 支持“始终置顶”模式。
* 支持多标签页或多窗口，同时解析多个请求。


* **快速唤起：** 支持全局快捷键（如 `Option + Space`）快速调出界面。

---

## 4. 非功能性需求

* **隐私安全 (Zero-Server)：** **绝对禁止**将用户粘贴的内容上传至服务器。所有解析、解码逻辑必须在本地完成。
* **性能：**
* 启动时间应在 1 秒以内。
* 处理万级行数的 Payload 时不应出现界面卡顿。


* **轻量化：** 安装包体积应尽量控制在 10MB 左右（推荐使用 Tauri）。

---

## 5. 界面原型建议 (UI Layout)

| 模块 | 描述 |
| --- | --- |
| **顶部工具栏** | 粘贴按钮、清空按钮、监听开关、Diff 模式切换、截图脱敏开关。 |
| **左侧输入/列表** | 原始文本输入区（或历史记录列表）。 |
| **中间主展示区** | K-V 表格。列：`Key` | `Value` | `Action (解码/查看)`。 |
| **右侧详情面板** | 选中某个字段后，展示该字段的**官方定义**、**MDN 链接**以及**解码后的完整内容**。 |

---

## 6. 推荐技术栈

* **后端 (System Layer):** **Rust** (基于 Tauri V2)，负责剪贴板监听、系统原生 API 调用、复杂字符串正则匹配。
* **前端 (UI Layer): **React** + **Tailwind CSS**。
* **数据字典:** 一个本地的 `zh-CN.json`，存储所有标准 HTTP Headers 的中文解释。

---
