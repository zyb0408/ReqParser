# ReqParser UI 设计规格说明书

## 1. 整体布局

### 1.1 布局架构

```
+------------------------------------------------------------------+
|                        Toolbar (h: 48px)                         |
|  [Logo ReqParser]   [Paste][Clear][Parse]   [Clip][Pin][Mask][T] |
+----------+------------------------------+-----------------------+
|          |                              |                       |
|  Input   |     K-V Tree Table           |    Detail Panel       |
|  Panel   |                              |                       |
|          |                              |                       |
| (w: 25%) |       (w: 50%)              |     (w: 25%)          |
|          |                              |                       |
| min:240  |      min: 400               |    min: 280           |
| max:480  |      flex: 1                |    max: 480           |
|          |                              |                       |
|          |                              |                       |
|          |                              |                       |
|          |                              |                       |
+----------+------------------------------+-----------------------+
|                     StatusBar (h: 28px)                          |
+------------------------------------------------------------------+
```

### 1.2 尺寸规格

| 区域 | 默认宽度 | 最小宽度 | 最大宽度 | 高度 |
|------|---------|---------|---------|------|
| 工具栏 | 100% | - | - | 48px |
| 左侧输入面板 | 25% | 240px | 480px | calc(100vh - 76px) |
| 中间 K-V 展示区 | 50% (弹性) | 400px | 无限制 | calc(100vh - 76px) |
| 右侧详情面板 | 25% | 280px | 480px | calc(100vh - 76px) |
| 状态栏 | 100% | - | - | 28px |

### 1.3 面板交互

- 三栏之间使用可拖拽分隔条（4px 宽，hover 时高亮为 `--ring` 色）
- 右侧详情面板默认收起，选中某行后自动展开；可手动收起
- 当窗口宽度 < 900px 时，右侧面板以覆盖层（overlay）方式弹出
- 面板拖拽使用 `cursor: col-resize`，拖拽中面板内容不重排（使用 `pointer-events: none`）

### 1.4 窗口约束

- 最小窗口尺寸：900 x 600 px
- 默认窗口尺寸：1200 x 800 px
- 支持 Tauri `setAlwaysOnTop` 切换置顶

---

## 2. 组件树

### 2.1 完整组件层级

```
App
  +-- ThemeProvider
      +-- AppLayout
          +-- Toolbar
          |     +-- ToolbarBrand
          |     +-- ToolbarActions
          |     |     +-- Button (粘贴)
          |     |     +-- Button (清空)
          |     |     +-- Button (解析)
          |     +-- ToolbarToggles
          |           +-- Toggle (剪贴板监听)
          |           +-- Toggle (始终置顶)
          |           +-- Toggle (隐私脱敏)
          |           +-- ThemeToggle
          +-- PanelGroup
          |     +-- InputPanel
          |     |     +-- PanelHeader
          |     |     +-- RawTextEditor
          |     |     +-- InputMeta
          |     +-- ResizeHandle
          |     +-- KVPanel
          |     |     +-- PanelHeader
          |     |     +-- KVTreeTable
          |     |     |     +-- KVTreeRow (recursive)
          |     |     |           +-- RowIndent
          |     |     |           +-- ExpandToggle
          |     |     |           +-- KeyCell
          |     |     |           +-- ValueCell
          |     |     |           +-- TypeBadge
          |     |     |           +-- RowActions
          |     |     +-- EmptyState
          |     +-- ResizeHandle
          |     +-- DetailPanel
          |           +-- PanelHeader
          |           +-- DetailContent
          |           |     +-- DetailSection (描述)
          |           |     +-- DetailSection (解码内容)
          |           |     +-- DetailSection (安全建议)
          |           +-- DetailEmpty
          +-- StatusBar
```

### 2.2 核心组件 Props 接口

```typescript
// === 布局组件 ===

interface AppLayoutProps {
  children: React.ReactNode;
}

interface PanelGroupProps {
  children: React.ReactNode;
  /** 面板尺寸持久化 key */
  storageKey?: string;
}

interface ResizeHandleProps {
  /** 拖拽方向 */
  direction: "horizontal";
  /** 拖拽时回调 */
  onResize: (delta: number) => void;
}

// === 工具栏 ===

interface ToolbarProps {
  parseState: "idle" | "parsing" | "done" | "error";
  clipboardWatching: boolean;
  alwaysOnTop: boolean;
  privacyMask: boolean;
  onPaste: () => void;
  onClear: () => void;
  onParse: () => void;
  onToggleClipboard: () => void;
  onToggleAlwaysOnTop: () => void;
  onTogglePrivacyMask: () => void;
}

interface ToolbarBrandProps {
  /* 无需外部 props */
}

interface ToolbarActionsProps {
  parseState: "idle" | "parsing" | "done" | "error";
  onPaste: () => void;
  onClear: () => void;
  onParse: () => void;
}

interface ToolbarTogglesProps {
  clipboardWatching: boolean;
  alwaysOnTop: boolean;
  privacyMask: boolean;
  onToggleClipboard: () => void;
  onToggleAlwaysOnTop: () => void;
  onTogglePrivacyMask: () => void;
}

interface ThemeToggleProps {
  /* 内部通过 ThemeProvider context 管理 */
}

// === 输入面板 ===

interface InputPanelProps {
  value: string;
  onChange: (value: string) => void;
  contentType: HttpContentType | null;
  lineCount: number;
}

interface RawTextEditorProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
}

interface InputMetaProps {
  contentType: HttpContentType | null;
  lineCount: number;
  charCount: number;
}

// === K-V 展示区 ===

interface KVPanelProps {
  result: ParseResult | null;
  selectedKey: string | null;
  onSelectRow: (node: ParseNode, path: string[]) => void;
  privacyMask: boolean;
}

interface KVTreeTableProps {
  nodes: ParseNode[];
  section: "headers" | "queryParams" | "body";
  selectedKey: string | null;
  onSelectRow: (node: ParseNode, path: string[]) => void;
  privacyMask: boolean;
  depth?: number;
}

interface KVTreeRowProps {
  node: ParseNode;
  section: string;
  depth: number;
  isExpanded: boolean;
  isSelected: boolean;
  hasChildren: boolean;
  privacyMask: boolean;
  onToggleExpand: () => void;
  onSelect: () => void;
  onCopyKey: () => void;
  onCopyValue: () => void;
}

interface TypeBadgeProps {
  /** 值的检测类型 */
  type: "jwt" | "base64" | "timestamp" | "json" | "url" | "encoded" | "plain";
}

interface RowActionsProps {
  onCopyKey: () => void;
  onCopyValue: () => void;
  onDecode?: () => void;
}

// === 详情面板 ===

interface DetailPanelProps {
  node: ParseNode | null;
  path: string[];
  dictionary: DictionaryEntry | null;
  isOpen: boolean;
  onClose: () => void;
}

interface DetailSectionProps {
  title: string;
  icon?: React.ReactNode;
  children: React.ReactNode;
  defaultOpen?: boolean;
}

interface DetailContentProps {
  node: ParseNode;
  path: string[];
  dictionary: DictionaryEntry | null;
}

// === 状态栏 ===

interface StatusBarProps {
  clipboardWatching: boolean;
  parseTime?: number;
  itemCount?: number;
}

// === 数据类型 ===

interface DictionaryEntry {
  name: string;
  category: "general" | "request" | "response" | "entity" | "security" | "custom";
  description: string;
  mdnUrl?: string;
  securityTips?: string[];
  values?: Record<string, string>;
}
```

### 2.3 组件文件路径

| 组件 | 文件路径 |
|------|---------|
| AppLayout | `src/components/layout/AppLayout.tsx` |
| Toolbar | `src/components/toolbar/Toolbar.tsx` |
| ToolbarBrand | `src/components/toolbar/ToolbarBrand.tsx` |
| ToolbarActions | `src/components/toolbar/ToolbarActions.tsx` |
| ToolbarToggles | `src/components/toolbar/ToolbarToggles.tsx` |
| ThemeToggle | `src/components/toolbar/ThemeToggle.tsx` |
| PanelGroup | `src/components/panels/PanelGroup.tsx` |
| ResizeHandle | `src/components/panels/ResizeHandle.tsx` |
| InputPanel | `src/components/panels/InputPanel.tsx` |
| RawTextEditor | `src/components/panels/RawTextEditor.tsx` |
| InputMeta | `src/components/panels/InputMeta.tsx` |
| KVPanel | `src/components/kv/KVPanel.tsx` |
| KVTreeTable | `src/components/kv/KVTreeTable.tsx` |
| KVTreeRow | `src/components/kv/KVTreeRow.tsx` |
| TypeBadge | `src/components/kv/TypeBadge.tsx` |
| RowActions | `src/components/kv/RowActions.tsx` |
| EmptyState | `src/components/kv/EmptyState.tsx` |
| DetailPanel | `src/components/detail/DetailPanel.tsx` |
| DetailContent | `src/components/detail/DetailContent.tsx` |
| DetailSection | `src/components/detail/DetailSection.tsx` |
| DetailEmpty | `src/components/detail/DetailEmpty.tsx` |
| StatusBar | `src/components/layout/StatusBar.tsx` |
| ThemeProvider | `src/components/ThemeProvider.tsx` |
| PanelHeader | `src/components/shared/PanelHeader.tsx` |

### 2.4 状态管理

采用 React Context + useReducer 集中管理全局状态，避免 prop drilling：

```typescript
interface AppState {
  // 输入
  rawText: string;
  // 解析结果
  parseResult: ParseResult | null;
  parseError: string | null;
  parseState: "idle" | "parsing" | "done" | "error";
  // 选中
  selectedNode: ParseNode | null;
  selectedPath: string[];
  // 开关
  clipboardWatching: boolean;
  alwaysOnTop: boolean;
  privacyMask: boolean;
  theme: "light" | "dark" | "system";
  // 面板
  detailPanelOpen: boolean;
  panelSizes: [number, number, number]; // 百分比
}

type AppAction =
  | { type: "SET_RAW_TEXT"; payload: string }
  | { type: "PARSE_START" }
  | { type: "PARSE_SUCCESS"; payload: ParseResult }
  | { type: "PARSE_ERROR"; payload: string }
  | { type: "SELECT_NODE"; payload: { node: ParseNode; path: string[] } }
  | { type: "CLEAR_SELECTION" }
  | { type: "TOGGLE_CLIPBOARD" }
  | { type: "TOGGLE_ALWAYS_ON_TOP" }
  | { type: "TOGGLE_PRIVACY_MASK" }
  | { type: "SET_THEME"; payload: "light" | "dark" | "system" }
  | { type: "SET_PANEL_SIZES"; payload: [number, number, number] }
  | { type: "TOGGLE_DETAIL_PANEL" }
  | { type: "CLEAR_ALL" };
```

Context 提供者：

```typescript
// src/lib/app-context.tsx
const AppContext = React.createContext<{
  state: AppState;
  dispatch: React.Dispatch<AppAction>;
} | null>(null);

function useApp() {
  const ctx = React.useContext(AppContext);
  if (!ctx) throw new Error("useApp must be used within AppProvider");
  return ctx;
}
```

---

## 3. 工具栏设计

### 3.1 布局结构

```
+------------------------------------------------------------------+
| [Logo] ReqParser  |  [Paste] [Clear] [Parse]  |  [C] [P] [M] [T] |
|     左侧品牌       |     中间操作按钮            |   右侧功能开关    |
+------------------------------------------------------------------+
  h: 48px, px: 16px, bg: background, border-bottom: border
```

### 3.2 左侧 -- 品牌区

- Logo：16x16 SVG 图标（使用 lucide-react `Terminal` 或自定义）
- 应用名：`ReqParser`，`text-sm font-semibold`
- 版本号：`v0.1.0`，`text-xs text-muted-foreground`，仅在宽度 > 1000px 时显示

### 3.3 中间 -- 操作按钮

| 按钮 | 图标 | 快捷键 | 说明 |
|------|------|--------|------|
| 粘贴 | `ClipboardPaste` | `Cmd+V` | 从剪贴板读取文本到输入区 |
| 清空 | `Trash2` | `Cmd+Shift+Delete` | 清空输入和解析结果 |
| 解析 | `Play` | `Cmd+Enter` | 触发解析（解析中显示旋转 `Loader2`） |

按钮样式：
- 默认使用 Shadcn `Button` variant="outline" size="sm"
- 解析按钮使用 variant="default"（主色）
- 解析中禁用按钮，图标替换为旋转的 `Loader2`

### 3.4 右侧 -- 功能开关

| 开关 | 图标（关） | 图标（开） | 说明 |
|------|-----------|-----------|------|
| 剪贴板监听 | `ClipboardList` | `ClipboardCheck` | 开启/关闭剪贴板自动检测 |
| 始终置顶 | `Pin` | `PinOff` | 窗口是否始终在最前 |
| 隐私脱敏 | `Eye` | `EyeOff` | 敏感字段值是否遮罩 |
| 主题切换 | `Sun` | `Moon` | 浅色/深色模式 |

开关样式：
- 使用 Shadcn `Toggle` 组件
- 开启状态：`bg-accent text-accent-foreground`
- 图标大小：16px
- Tooltip 显示功能名称和快捷键

### 3.5 工具栏分隔

- 品牌区和操作按钮之间：`Separator` 竖线（`h-6 mx-3`）
- 操作按钮和功能开关之间：弹性空间 `flex-1`
- 按钮之间间距：`gap-1`

---

## 4. K-V 树形表格设计（核心）

### 4.1 表头

```
+--------+------------------------------------------+--------+--------+
|  Key   |             Value                        |  Type  | Actions|
+--------+------------------------------------------+--------+--------+
  w: 200px          flex: 1                          w: 72px   w: 80px
  min: 120px                                         fixed     fixed
```

表头样式：
- `h-8 bg-muted/50 text-xs font-medium text-muted-foreground uppercase tracking-wider`
- 固定在顶部（sticky）
- Key 列可拖拽调整宽度

### 4.2 分区标题

当解析结果包含多个区域（Headers、Query Params、Body）时，用分区行分隔：

```
+------------------------------------------------------------------+
| > Headers (12)                                          [折叠全部] |
+------------------------------------------------------------------+
|   Content-Type     application/json           plain      [C][C]  |
|   Authorization    Bearer eyJhbGci...         jwt        [C][D]  |
|   v Cookie         session=abc; theme=dark    encoded    [C][D]  |
|     +-- session    abc                        plain      [C]     |
|     +-- theme      dark                       plain      [C]     |
+------------------------------------------------------------------+
| > Query Params (3)                                      [折叠全部] |
+------------------------------------------------------------------+
```

分区标题样式：
- `h-9 bg-muted/30 px-3 font-medium text-sm`
- 左侧展开/收起箭头 + 区域名称 + 计数 Badge
- 右侧"折叠全部"按钮

### 4.3 行样式

| 状态 | 样式 |
|------|------|
| 默认 | `h-9 px-3 border-b border-border/50` |
| Hover | `bg-accent/50` |
| 选中 | `bg-accent border-l-2 border-l-primary` |
| 展开的父节点 | 正常样式，展开箭头旋转 90 度 |
| 子节点 | 左侧缩进 `pl-{depth * 24}px`，带树线 |

### 4.4 缩进与树线

- 每级缩进 24px
- 树线使用 `border-l border-muted-foreground/20`，从父节点左侧 12px 处垂直向下
- 最后一个子节点使用 L 形连接线（`border-l + border-b`），连接线高度为行高一半
- 展开/收起使用 lucide `ChevronRight` 图标，展开时旋转 90 度

```
depth=0:  [>] Content-Type      application/json
depth=0:  [v] Cookie             session=abc; theme=dark
depth=1:   |-- session           abc
depth=1:   +-- theme             dark
```

### 4.5 值类型颜色编码（TypeBadge）

| 类型 | 标签文字 | 背景色（浅色模式） | 背景色（深色模式） | 文字色 |
|------|---------|-------------------|-------------------|--------|
| JWT | `JWT` | `oklch(0.95 0.03 300)` | `oklch(0.25 0.05 300)` | 紫色 `oklch(0.55 0.2 300)` |
| Base64 | `B64` | `oklch(0.95 0.03 240)` | `oklch(0.25 0.05 240)` | 蓝色 `oklch(0.55 0.15 240)` |
| Timestamp | `TIME` | `oklch(0.95 0.03 150)` | `oklch(0.25 0.05 150)` | 绿色 `oklch(0.55 0.15 150)` |
| JSON | `JSON` | `oklch(0.95 0.04 75)` | `oklch(0.25 0.05 75)` | 橙色 `oklch(0.6 0.2 75)` |
| URL Encoded | `URL` | `oklch(0.95 0.03 200)` | `oklch(0.25 0.05 200)` | 青色 `oklch(0.55 0.15 200)` |
| Encoded (复合) | `ENC` | `oklch(0.95 0.03 30)` | `oklch(0.25 0.05 30)` | 红色 `oklch(0.6 0.2 30)` |
| Plain | - | 不显示 Badge | - | - |

Badge 样式：
- `text-[10px] font-mono font-semibold px-1.5 py-0.5 rounded-sm`
- 使用上述颜色作为 CSS 变量，挂载在自定义 data 属性上

### 4.6 值显示

- 普通值：`font-mono text-sm` 原样显示
- 长值（超过列宽）：`truncate`，末尾显示省略号
- 隐私脱敏模式下的敏感值：替换为 `***`，显示为 `text-muted-foreground italic`
- 敏感 Key 判定规则：Key 包含 `token`、`auth`、`key`、`secret`、`password`、`cookie`（不区分大小写）

### 4.7 行操作按钮（RowActions）

| 按钮 | 图标 | 说明 | 可见条件 |
|------|------|------|---------|
| 复制 Key | `Copy` (12px) | 复制当前行的 Key | 始终（hover 时显示） |
| 复制 Value | `Copy` (12px) | 复制当前行的原始 Value | 始终（hover 时显示） |
| 解码 | `ArrowRightLeft` (12px) | 切换原始值/解码值 | 仅当 type 不为 plain 时 |

按钮默认 `opacity-0`，行 hover 时 `opacity-100`；选中行时始终显示。

### 4.8 表格空状态

当没有解析结果时，显示居中的空状态提示：

```
+------------------------------------------------------------------+
|                                                                  |
|                    [ClipboardPaste 图标 48px]                      |
|                                                                  |
|               粘贴 HTTP 请求/响应文本开始解析                       |
|           支持 Request、Response、Headers、JSON 等格式              |
|                                                                  |
|                   [Cmd+V 快捷粘贴]                                 |
|                                                                  |
+------------------------------------------------------------------+
```

---

## 5. 输入面板设计

### 5.1 布局

```
+---------------------------+
| [InputMeta]  [语法提示灯]  |   h: 36px
+---------------------------+
|                           |
|   RawTextEditor           |   flex: 1
|   (monospace textarea)    |
|                           |
|                           |
+---------------------------+
| 行数: 42  字符: 2,301     |   h: 28px
+---------------------------+
```

### 5.2 文本编辑器

- 使用 HTML `<textarea>` 元素（无需代码编辑器的复杂性）
- `font-mono text-sm leading-relaxed`
- 显示行号：通过 CSS `counter` 或左侧行号槽实现
- 背景色：`bg-muted/30`
- 自动检测粘贴事件并触发解析（可在设置中关闭）

### 5.3 输入元信息（InputMeta）

底部显示：
- 检测到的内容类型 Badge（如 `Request`、`Response`、`Headers Only`）
- 行数统计
- 字符数统计

---

## 6. 侧边详情面板

### 6.1 布局

```
+---------------------------+
| [X]  Header 名称          |   h: 48px
|      [分类标签]            |
+---------------------------+
| -- 描述 --                |
| 官方描述文本...             |
| [MDN 文档 链接]            |
+---------------------------+
| -- 解码内容 --            |
| {                         |
|   "sub": "1234567890",    |
|   "exp": 1735689600       |   flex: 1, 可滚动
| }                         |
+---------------------------+
| -- 安全建议 --            |
| [!] Cookie 缺少 HttpOnly  |
| [!] 未设置 Secure 标志    |
+---------------------------+
```

### 6.2 Header 区域

- 关闭按钮 `X`：右上角，`Button` variant="ghost" size="icon"
- 字段名称：`text-lg font-semibold`
- 分类标签：使用 Shadcn `Badge` 组件
  - General: `variant="secondary"`
  - Request: `variant="default"`
  - Response: `variant="outline"`
  - Security: `variant="destructive"`

### 6.3 描述区域（DetailSection）

- 使用 Shadcn `Collapsible` 组件
- 标题行：图标 + 标题文字 + 展开/收起箭头
- 描述文本：`text-sm text-muted-foreground leading-relaxed`
- MDN 链接：`text-xs text-primary underline`，点击通过 `shell.open()` 在系统浏览器打开

### 6.4 解码内容区域

- 当值为 JWT 时：显示三个可折叠区域（Header、Payload、Signature）
- 当值为 JSON 时：使用语法高亮的 JSON 格式化显示
- 当值为 Base64 时：显示解码后的文本
- 当值为时间戳时：显示转换后的本地时间格式
- 代码区域样式：`bg-muted rounded-md p-3 font-mono text-xs overflow-auto`

### 6.5 安全建议区域

- 仅在词典中定义了 `securityTips` 时显示
- 每条建议前缀三角警告图标 `AlertTriangle`
- 背景色：`bg-destructive/5 border border-destructive/20 rounded-md p-3`
- 文字：`text-sm text-destructive`

### 6.6 空状态

当没有选中任何行时：

```
+---------------------------+
|                           |
|     [MousePointer 图标]    |
|                           |
|   点击左侧表格中的任意行   |
|   查看字段详细信息         |
|                           |
+---------------------------+
```

---

## 7. 状态栏设计

```
+------------------------------------------------------------------+
| [Dot] 剪贴板监听中  |  解析耗时: 12ms  |  共 42 个字段  |  v0.1.0 |
+------------------------------------------------------------------+
  h: 28px, text-xs, text-muted-foreground, bg-muted/30, border-top
```

- 剪贴板状态：绿色圆点表示监听中，灰色表示关闭
- 解析耗时：仅在解析完成后显示
- 字段计数：显示解析结果的总项目数
- 版本号：右对齐

---

## 8. 主题系统

### 8.1 主题切换机制

- 三种模式：`light`、`dark`、`system`（跟随系统）
- 通过在 `<html>` 元素上添加/移除 `dark` class 实现
- 用户偏好持久化到 `localStorage` key `reqparser-theme`
- 使用 `ThemeProvider` context 管理

### 8.2 浅色模式变量（基于 Shadcn neutral 基色）

已在 `src/index.css` 中定义，保持现有变量不变。

### 8.3 深色模式变量

已在 `src/index.css` 的 `.dark` 选择器中定义，保持现有变量不变。

### 8.4 扩展语义变量

在现有 CSS 变量基础上，为 ReqParser 特有功能新增以下变量：

```css
:root {
  /* 值类型颜色 */
  --type-jwt: oklch(0.55 0.2 300);
  --type-jwt-bg: oklch(0.95 0.03 300);
  --type-base64: oklch(0.55 0.15 240);
  --type-base64-bg: oklch(0.95 0.03 240);
  --type-timestamp: oklch(0.55 0.15 150);
  --type-timestamp-bg: oklch(0.95 0.03 150);
  --type-json: oklch(0.6 0.2 75);
  --type-json-bg: oklch(0.95 0.04 75);
  --type-url: oklch(0.55 0.15 200);
  --type-url-bg: oklch(0.95 0.03 200);
  --type-encoded: oklch(0.6 0.2 30);
  --type-encoded-bg: oklch(0.95 0.03 30);

  /* 树线 */
  --tree-line: oklch(0.7 0 0);

  /* 状态指示 */
  --status-active: oklch(0.65 0.2 150);
  --status-inactive: oklch(0.7 0 0);
}

.dark {
  --type-jwt: oklch(0.7 0.15 300);
  --type-jwt-bg: oklch(0.25 0.05 300);
  --type-base64: oklch(0.7 0.12 240);
  --type-base64-bg: oklch(0.25 0.05 240);
  --type-timestamp: oklch(0.7 0.12 150);
  --type-timestamp-bg: oklch(0.25 0.05 150);
  --type-json: oklch(0.75 0.15 75);
  --type-json-bg: oklch(0.25 0.05 75);
  --type-url: oklch(0.7 0.12 200);
  --type-url-bg: oklch(0.25 0.05 200);
  --type-encoded: oklch(0.75 0.15 30);
  --type-encoded-bg: oklch(0.25 0.05 30);

  --tree-line: oklch(0.4 0 0);

  --status-active: oklch(0.7 0.18 150);
  --status-inactive: oklch(0.4 0 0);
}
```

### 8.5 代码区域语法高亮

JSON 格式化显示的语法高亮色（纯 CSS 实现，不依赖外部高亮库）：

```css
:root {
  --syntax-key: oklch(0.55 0.2 300);      /* 紫色 - JSON key */
  --syntax-string: oklch(0.55 0.15 150);   /* 绿色 - 字符串值 */
  --syntax-number: oklch(0.6 0.2 75);      /* 橙色 - 数字 */
  --syntax-boolean: oklch(0.55 0.15 240);  /* 蓝色 - 布尔值 */
  --syntax-null: oklch(0.6 0 0);           /* 灰色 - null */
  --syntax-bracket: oklch(0.5 0 0);        /* 灰色 - 括号 */
}

.dark {
  --syntax-key: oklch(0.75 0.15 300);
  --syntax-string: oklch(0.7 0.12 150);
  --syntax-number: oklch(0.75 0.15 75);
  --syntax-boolean: oklch(0.7 0.12 240);
  --syntax-null: oklch(0.5 0 0);
  --syntax-bracket: oklch(0.6 0 0);
}
```

---

## 9. 交互动画

### 9.1 面板切换

- 右侧详情面板展开/收起：`transition-[width] duration-200 ease-out`
- 面板从 `width: 0` 到目标宽度，内容使用 `opacity` 渐显（`transition-opacity duration-150 delay-100`）

### 9.2 树节点展开/收起

- 展开箭头旋转：`transition-transform duration-150 ease-out`（0deg -> 90deg）
- 子节点展开：使用 `height: auto` 动画，通过 `grid-template-rows: 0fr -> 1fr` 技巧实现平滑过渡
- 时长：`duration-200 ease-out`

### 9.3 复制成功反馈

- 点击复制按钮后，图标从 `Copy` 切换为 `Check`
- 使用绿色 `text-status-active` 颜色
- 1.5 秒后自动恢复原始图标
- 图标切换使用 `transition-opacity duration-100`

### 9.4 行 Hover 效果

- 背景色过渡：`transition-colors duration-75`
- 操作按钮显隐：`transition-opacity duration-100`

### 9.5 解析按钮状态

- 点击后图标切换为旋转的 `Loader2`：`animate-spin`
- 解析成功：短暂闪烁绿色边框（`ring-2 ring-status-active`，150ms 后消失）
- 解析失败：短暂闪烁红色边框（`ring-2 ring-destructive`，150ms 后消失）

### 9.6 TypeBadge 出现

- 首次渲染时使用 `animate-in fade-in-0 zoom-in-95 duration-150`

---

## 10. 可视化参考

### 10.1 完整界面示意图（浅色模式）

```
+====================================================================+
|  [T] ReqParser v0.1  |  [Paste] [Clear] [>Parse]  | [C][P][M][Sun] |
+====================================================================+
|  Input Panel    ||   K-V Tree Table              ||  Detail Panel   |
|  -----------    ||   ----------------------      ||  -------------  |
|  [Request  v]   ||  Key        Value    Ty  Act  ||  [X] Cookie     |
|                 ||  ---------  -------  --  ---  ||  [Security]     |
|  GET /api/user  ||  > Headers (8)                ||                 |
|  Host: exam...  ||  Content-T  app/json       [C]||  -- 描述 --     |
|  Accept: */*    ||  Accept     */*            [C]||  HTTP Cookie    |
|  Cookie: se...  ||  v Cookie   session=..  ENC   ||  用于在请求中   |
|  Authorizat...  ||    session  abc123           ||  携带服务端设   |
|  Cache-Cont...  ||    theme    dark             ||  定的 Cookie。  |
|  X-Request-...  ||  Authoriz  Bearer.. JWT  [D] ||  [MDN 文档]     |
|  X-Custom-H...  ||  Cache-Co  no-cache      [C] ||                 |
|                 ||  X-Reques  abc123        [C] ||  -- 安全建议 -- |
|                 ||  X-Custom  test          [C] ||  [!] 缺少       |
|                 ||                              ||  HttpOnly 标志  |
|                 ||  > Query Params (2)          ||                 |
|                 ||  page       1            [C] ||                 |
|                 ||  limit      20           [C] ||                 |
|                 ||                              ||                 |
| L:12  C:341     ||                              ||                 |
+-----------------||------------------------------||-----------------|
| [*] 监听中  |  解析: 8ms  |  共 12 项  |              v0.1.0       |
+====================================================================+
```

### 10.2 关键尺寸标注

```
+-- 48px (Toolbar height)
|
+-- 全高 calc(100vh - 76px) -- 主内容区
|   |
|   +-- 240~480px (左侧面板, 默认 25%)
|   |   +-- 16px padding
|   |
|   +-- 4px (ResizeHandle)
|   |
|   +-- 400px+ (中间面板, flex: 1)
|   |   +-- 32px (表头 height)
|   |   +-- 36px (分区标题 height)
|   |   +-- 36px (每行 height)
|   |   +-- 24px (每级缩进)
|   |
|   +-- 4px (ResizeHandle)
|   |
|   +-- 280~480px (右侧面板, 默认 25%)
|       +-- 16px padding
|
+-- 28px (StatusBar height)
```

### 10.3 间距规范

| 位置 | 间距 |
|------|------|
| 工具栏内边距 | `px-4 py-2` (16px / 8px) |
| 面板内边距 | `p-4` (16px) |
| 表格行内边距 | `px-3 py-1.5` (12px / 6px) |
| 按钮间距 | `gap-1` (4px) |
| 分区标题与行间距 | `gap-0`（无间距，紧贴） |
| 详情面板区块间距 | `space-y-4` (16px) |
| TypeBadge 与 Value 间距 | `ml-2` (8px) |

### 10.4 字体规范

| 用途 | 字体 | 大小 | 权重 |
|------|------|------|------|
| 应用名 | 系统字体 | 14px (`text-sm`) | 600 (`font-semibold`) |
| 分区标题 | 系统字体 | 14px (`text-sm`) | 500 (`font-medium`) |
| 表头 | 系统字体 | 12px (`text-xs`) | 500 (`font-medium`) |
| Key 列 | 等宽字体 | 13px (`text-[13px]`) | 500 (`font-medium`) |
| Value 列 | 等宽字体 | 13px (`text-[13px]`) | 400 (`font-normal`) |
| TypeBadge | 等宽字体 | 10px (`text-[10px]`) | 600 (`font-semibold`) |
| 详情标题 | 系统字体 | 18px (`text-lg`) | 600 (`font-semibold`) |
| 详情正文 | 系统字体 | 14px (`text-sm`) | 400 (`font-normal`) |
| 代码区域 | 等宽字体 | 12px (`text-xs`) | 400 (`font-normal`) |
| 状态栏 | 系统字体 | 12px (`text-xs`) | 400 (`font-normal`) |

等宽字体栈：`font-mono` 即 `ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace`

---

## 11. 键盘快捷键

| 快捷键 | 功能 |
|--------|------|
| `Cmd+V` | 粘贴并自动解析 |
| `Cmd+Enter` | 手动触发解析 |
| `Cmd+Shift+Delete` | 清空所有内容 |
| `Cmd+Shift+C` | 切换剪贴板监听 |
| `Cmd+Shift+P` | 切换始终置顶 |
| `Cmd+Shift+M` | 切换隐私脱敏 |
| `Cmd+,` | 主题切换 |
| `Escape` | 关闭详情面板 / 取消选中 |
| `ArrowUp/Down` | 在 K-V 表格中导航行 |
| `ArrowLeft` | 收起当前节点 |
| `ArrowRight` | 展开当前节点 |
| `Enter` | 选中当前行并打开详情 |
| `c` | 复制当前行的值（焦点在表格时） |

---

## 12. 响应式行为

| 窗口宽度 | 布局变化 |
|---------|---------|
| >= 1200px | 三栏完整布局 |
| 900 ~ 1199px | 右侧面板以 overlay 方式覆盖在中间面板上方 |
| < 900px | 不支持（窗口最小宽度限制为 900px） |

---

## 13. 无障碍（Accessibility）

- 所有交互元素需有 `aria-label`
- 表格使用 `role="treegrid"` + `role="row"` + `role="gridcell"`
- 展开/收起使用 `aria-expanded`
- 功能开关使用 `aria-pressed`
- 颜色对比度遵循 WCAG AA 标准（文字与背景对比度 >= 4.5:1）
- 支持键盘完整导航（Tab、Arrow、Enter、Escape）

---

## 14. Shadcn/ui 组件使用清单

以下列出需要安装的 Shadcn/ui 组件：

| 组件 | 用途 |
|------|------|
| `button` | 工具栏按钮、行操作按钮 |
| `toggle` | 功能开关 |
| `badge` | 类型标签、分类标签 |
| `separator` | 工具栏分隔线 |
| `tooltip` | 按钮和开关的提示 |
| `collapsible` | 详情面板可折叠区域、树节点展开 |
| `scroll-area` | 面板内滚动区域 |
| `resizable` | 三栏面板拖拽调整（基于 react-resizable-panels） |

安装命令：

```bash
pnpm dlx shadcn@latest add button toggle badge separator tooltip collapsible scroll-area resizable
```
