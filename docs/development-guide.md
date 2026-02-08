# ReqParser 开发者指南

## 目录

- [1. 环境要求](#1-环境要求)
- [2. 项目克隆与安装](#2-项目克隆与安装)
- [3. 开发命令](#3-开发命令)
- [4. 项目目录结构](#4-项目目录结构)
- [5. 如何添加新的解析器](#5-如何添加新的解析器)
- [6. 如何扩展词典](#6-如何扩展词典)
- [7. 如何添加新的 Shadcn 组件](#7-如何添加新的-shadcn-组件)
- [8. 测试指南](#8-测试指南)
- [9. CI/CD 说明](#9-cicd-说明)
- [10. 常见问题](#10-常见问题)

---

## 1. 环境要求

| 工具 | 最低版本 | 说明 |
|------|---------|------|
| **Rust** | 1.80+ | 需要 `LazyLock` 稳定支持 |
| **Node.js** | LTS | 推荐使用最新 LTS 版本 |
| **pnpm** | latest | 包管理器 |
| **Tauri CLI** | 2.x | 通过 `pnpm tauri` 调用 |

### 平台特定依赖

**macOS：** 无额外依赖（Xcode Command Line Tools 需要安装）

**Ubuntu/Debian：**

```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

**Windows：** 需要安装 Microsoft Visual Studio C++ Build Tools。

---

## 2. 项目克隆与安装

```bash
# 克隆项目
git clone <repository-url>
cd ReqParser

# 安装前端依赖
pnpm install

# 如果 esbuild 构建脚本被阻止，需要手动批准
pnpm approve-builds
```

> **注意：** pnpm 默认会阻止 esbuild 等包的 postinstall 构建脚本。如果安装后出现 esbuild 相关错误，运行 `pnpm approve-builds` 批准构建。

---

## 3. 开发命令

### 日常开发

```bash
# 启动 Tauri 开发模式（同时启动 Vite 和 Rust 编译）
pnpm tauri dev

# 仅启动前端开发服务器（不含 Tauri）
pnpm dev
```

### 构建

```bash
# 构建生产版本
pnpm tauri build

# 仅构建前端
pnpm build
```

### Rust 相关

```bash
# 运行所有 Rust 测试
cd src-tauri && cargo test

# 运行特定模块的测试
cd src-tauri && cargo test --lib curl_parser
cd src-tauri && cargo test --lib fetch_parser
cd src-tauri && cargo test --lib parser
cd src-tauri && cargo test --lib detector

# 检查 Rust 代码
cd src-tauri && cargo check

# 格式化 Rust 代码
cd src-tauri && cargo fmt

# Lint 检查
cd src-tauri && cargo clippy
```

### 前端相关

```bash
# TypeScript 类型检查
pnpm tsc --noEmit

# 预览构建产物
pnpm preview
```

---

## 4. 项目目录结构

```
ReqParser/
├── .github/
│   └── workflows/
│       └── publish.yml          # CI/CD 跨平台构建和发布
├── docs/                        # 项目文档
│   ├── architecture.md          # 系统架构
│   ├── api-reference.md         # API 参考
│   ├── development-guide.md     # 开发者指南（本文档）
│   ├── changelog.md             # 变更日志
│   └── ui-components.md         # 前端组件文档
├── public/                      # 静态资源
├── src/                         # 前端源代码
│   ├── assets/                  # 静态资源（图片等）
│   ├── data/                    # 数据文件（词典等）
│   ├── lib/                     # 工具库
│   ├── App.tsx                  # React 入口组件
│   ├── index.css                # 全局样式（Tailwind + Shadcn 主题）
│   ├── main.tsx                 # React 挂载入口
│   └── vite-env.d.ts            # Vite 类型声明
├── src-tauri/                   # Rust 后端源代码
│   ├── src/
│   │   ├── lib.rs               # Tauri 入口，命令注册
│   │   ├── main.rs              # 程序入口
│   │   ├── models.rs            # 核心数据模型
│   │   ├── error.rs             # 错误类型定义
│   │   ├── detector.rs          # 输入格式检测
│   │   ├── parser.rs            # 原始 HTTP 解析器
│   │   ├── curl_parser.rs       # cURL 命令解析器
│   │   ├── fetch_parser.rs      # fetch() 调用解析器
│   │   └── clipboard.rs         # 剪贴板异步监听
│   ├── Cargo.toml               # Rust 依赖配置
│   ├── tauri.conf.json          # Tauri 应用配置
│   └── build.rs                 # Tauri 构建脚本
├── components.json              # Shadcn/ui 配置
├── index.html                   # HTML 入口
├── package.json                 # 前端依赖和脚本
├── pnpm-lock.yaml               # 依赖锁文件
├── tsconfig.json                # TypeScript 配置
├── tsconfig.node.json           # Node.js TypeScript 配置
├── vite.config.ts               # Vite 构建配置
├── Req.md                       # 需求说明书
└── Tech.md                      # 技术说明书
```

---

## 5. 如何添加新的解析器

以 `curl_parser.rs` 为模板，添加一个新的输入格式解析器的步骤：

### 步骤 1：创建解析器模块

在 `src-tauri/src/` 下创建新文件，例如 `my_parser.rs`：

```rust
use crate::models::{HttpContentType, ParseNode, ParseResult};

/// 解析 MyFormat 格式的文本。
pub fn parse_my_format(input: &str) -> ParseResult {
    let raw_text = input.to_string();

    // 1. 提取 URL
    let url_str: Option<String> = /* 你的提取逻辑 */;

    // 2. 提取 method
    let method: Option<String> = /* 你的提取逻辑 */;

    // 3. 提取 headers
    let headers: Vec<ParseNode> = /* 你的提取逻辑 */;

    // 4. 提取 body
    let body: Option<String> = /* 你的提取逻辑 */;

    // 5. 解析 query params
    let query_params = url_str.as_ref().and_then(|u| parse_query_params(u));

    ParseResult {
        content_type: HttpContentType::Request,
        method,
        url: url_str,
        status_code: None,
        status_text: None,
        protocol: None,
        headers,
        query_params,
        body,
        raw_text,
    }
}
```

### 步骤 2：注册模块

在 `src-tauri/src/lib.rs` 中添加模块声明：

```rust
mod my_parser;
```

### 步骤 3：扩展格式检测

在 `src-tauri/src/detector.rs` 中：

1. 在 `InputFormat` 枚举中添加新变体：

```rust
pub enum InputFormat {
    Curl,
    Fetch,
    RawHttp,
    MyFormat,  // 新增
    Unknown,
}
```

2. 在 `detect_input_format()` 中添加检测逻辑：

```rust
if trimmed.starts_with("my_prefix") {
    return InputFormat::MyFormat;
}
```

### 步骤 4：接入路由

在 `src-tauri/src/lib.rs` 的 `parse_text` 命令中添加分支：

```rust
let result = match detector::detect_input_format(&raw_text) {
    InputFormat::Curl => curl_parser::parse_curl(&raw_text),
    InputFormat::Fetch => fetch_parser::parse_fetch(&raw_text),
    InputFormat::MyFormat => my_parser::parse_my_format(&raw_text),  // 新增
    InputFormat::RawHttp => parser::parse_http_text(&raw_text),
    InputFormat::Unknown => parser::parse_http_text(&raw_text),
};
```

### 步骤 5：添加测试

在解析器文件底部添加 `#[cfg(test)] mod tests` 模块：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parse() {
        let input = "my_prefix ...";
        let result = parse_my_format(input);
        assert_eq!(result.method.as_deref(), Some("GET"));
    }
}
```

---

## 6. 如何扩展词典

HTTP Header 百科词典存储在 `src/data/` 目录下。词典用于在解析结果中为 Header 字段提供中文解释。

词典文件格式为 JSON，结构如下：

```json
{
  "Header-Name": {
    "desc": "字段的中文描述",
    "values": {
      "value1": "该值的解释",
      "value2": "该值的解释"
    }
  }
}
```

要添加新的 Header 解释，在词典 JSON 文件中追加对应条目即可。

---

## 7. 如何添加新的 Shadcn 组件

项目使用 Shadcn/ui（new-york 风格）。添加新组件：

```bash
# 添加 Shadcn 组件（例如 Button）
pnpm dlx shadcn@latest add button

# 添加多个组件
pnpm dlx shadcn@latest add dialog table tabs
```

组件会被安装到 `src/components/ui/` 目录下。

> **注意：** 如果 `shadcn` CLI 因 esbuild 问题而挂起，可以手动从 Shadcn 仓库复制组件文件到 `src/components/ui/` 下，然后安装缺失的依赖。

`components.json` 已配置好路径别名和样式选项：

```json
{
  "style": "new-york",
  "tailwind": {
    "cssVariables": true
  },
  "aliases": {
    "components": "@/components",
    "utils": "@/lib/utils"
  }
}
```

---

## 8. 测试指南

### 当前测试覆盖

项目目前包含 **50 个 Rust 单元测试**，分布如下：

| 模块 | 测试数量 | 覆盖内容 |
|------|---------|---------|
| `detector.rs` | 15 | 格式检测、is_http_like 判断 |
| `curl_parser.rs` | 15 | cURL 命令解析、续行合并、token 化、Cookie 拆解 |
| `parser.rs` | 10 | HTTP 请求/响应解析、query params、Cookie 子节点 |
| `fetch_parser.rs` | 10 | fetch 调用解析、JSON options、Cookie 拆解 |

### 运行测试

```bash
# 运行全部测试
cd src-tauri && cargo test

# 带输出运行
cd src-tauri && cargo test -- --nocapture

# 运行单个测试
cd src-tauri && cargo test test_basic_get_with_headers
```

### 测试编写规范

- 每个解析器至少覆盖：基本解析、边界情况、真实世界样例
- 测试命名使用 `test_` 前缀 + 描述性名称
- 使用 `assert_eq!` 精确匹配，使用 `assert!(matches!(...))` 匹配枚举
- 真实世界测试用例应使用从浏览器 DevTools 复制的实际数据

---

## 9. CI/CD 说明

### 发布流程

项目使用 GitHub Actions 实现跨平台自动构建和发布，配置在 `.github/workflows/publish.yml`。

**触发条件：**
- 推送以 `v` 开头的 tag（如 `v0.1.0`）
- 手动触发（`workflow_dispatch`）

**构建矩阵：**

| 平台 | 目标 |
|------|------|
| macOS | `aarch64-apple-darwin` (Apple Silicon) |
| macOS | `x86_64-apple-darwin` (Intel) |
| Ubuntu | x86_64 |
| Ubuntu | ARM |
| Windows | x86_64 |

**发布步骤：**

1. 创建 git tag 并推送：

```bash
git tag v0.1.0
git push origin v0.1.0
```

2. GitHub Actions 自动：
   - 在所有平台上安装依赖（Rust、Node.js、pnpm、系统库）
   - 使用 Rust 缓存加速编译
   - 运行 `pnpm install` 和 `tauri build`
   - 创建 GitHub Release（草稿模式）
   - 上传各平台安装包

3. 在 GitHub Releases 页面审核并发布 Release

**Release 命名格式：** `ReqParser v{VERSION}`

---

## 10. 常见问题

### Q: pnpm install 后 esbuild 报错

**A:** 运行 `pnpm approve-builds` 批准 esbuild 的构建脚本。或者在 `.npmrc` 中添加配置。

### Q: Shadcn CLI 初始化时卡住

**A:** 如果 `pnpm dlx shadcn@latest init` 卡住，很可能是 esbuild 未批准构建。先运行 `pnpm approve-builds`，再重试。也可以手动安装依赖并复制组件文件。

### Q: macOS 上 arboard 编译失败

**A:** 确保安装了 Xcode Command Line Tools：`xcode-select --install`。

### Q: cargo test 找不到某个测试

**A:** 确保使用 `--lib` flag：`cargo test --lib test_name`。Tauri 的 integration tests 需要单独配置。

### Q: Vite 开发服务器端口冲突

**A:** 项目固定使用 1420 端口（`strictPort: true`）。如果端口被占用，需要先释放该端口。
