# phi-tools

[![Crates.io](https://img.shields.io/crates/v/phi-tools.svg)](https://crates.io/crates/phi-tools)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

通用 Agent 工具集，基于 [agent-base](https://crates.io/crates/agent-base) 运行时。

每个工具独立实现 `agent_base::Tool` trait，消费方按需注册到 `AgentBuilder`。

[English](README.md)

## 功能特性

| Feature | 说明 | 额外依赖 |
|---------|------|----------|
| *(默认)* | `LocalShellTool` — 执行 Shell 命令 | — |
| `browser` | 22 个浏览器自动化工具（Chrome DevTools Protocol） | `headless_chrome`, `html2md` |

```toml
[dependencies]
# 仅 Shell
phi-tools = "0.1.5"

# 包含浏览器自动化
phi-tools = { version = "0.1.5", features = ["browser"] }
```

## 工具列表

### 本地 Shell

| 工具 | 函数名 | 说明 |
|------|--------|------|
| `LocalShellTool` | `execute_command` | 通过 `sh -c` 执行 Shell 命令，支持超时和取消 |

```rust
use phi_tools::LocalShellTool;

builder.register_tool(LocalShellTool::new(30_000));  // 30 秒超时
```

### 浏览器自动化 (feature = `browser`)

22 个基于 CDP 的浏览器控制工具：

**导航与交互：**

| 工具 | 函数名 | 说明 |
|------|--------|------|
| `BrowserNavigateTool` | `browser_navigate` | 导航到指定 URL |
| `BrowserClickTool` | `browser_click` | 通过选择器或坐标点击元素 |
| `BrowserInputTool` | `browser_input` | 在输入框中输入文本 |
| `BrowserScrollTool` | `browser_scroll` | 滚动页面 |
| `BrowserHoverTool` | `browser_hover` | 悬停在元素上 |
| `BrowserSelectTool` | `browser_select` | 从下拉框中选择选项 |
| `BrowserPressKeyTool` | `browser_press_key` | 按下键盘按键 |

**内容提取：**

| 工具 | 函数名 | 说明 |
|------|--------|------|
| `BrowserScreenshotTool` | `browser_screenshot` | 截取页面截图 |
| `BrowserSnapshotTool` | `browser_snapshot` | 捕获页面可访问性树 |
| `BrowserEvaluateTool` | `browser_evaluate` | 在页面中执行 JavaScript |
| `BrowserExtractTool` | `browser_extract` | 从页面提取结构化内容 |
| `BrowserGetMarkdownTool` | `browser_get_markdown` | 将页面转换为 Markdown |
| `BrowserReadLinksTool` | `browser_read_links` | 列出页面上所有链接 |

**标签页管理：**

| 工具 | 函数名 | 说明 |
|------|--------|------|
| `BrowserGoBackTool` | `browser_go_back` | 后退 |
| `BrowserGoForwardTool` | `browser_go_forward` | 前进 |
| `BrowserNewTabTool` | `browser_new_tab` | 打开新标签页 |
| `BrowserCloseTabTool` | `browser_close_tab` | 关闭标签页 |
| `BrowserSwitchTabTool` | `browser_switch_tab` | 切换到指定标签页 |
| `BrowserTabListTool` | `browser_tab_list` | 列出所有标签页 |

**生命周期：**

| 工具 | 函数名 | 说明 |
|------|--------|------|
| `BrowserWaitTool` | `browser_wait` | 等待条件或超时 |
| `BrowserCloseTool` | `browser_close` | 关闭浏览器 |
| `BrowserRestartTool` | `browser_restart` | 重启浏览器（崩溃恢复） |

#### 使用方式

**方式一：`register_browser_tools` 助手（一次性注册全部 22 个工具）**

```rust
use phi_tools::{BrowserToolset, register_browser_tools};

let browser = BrowserToolset::launch(Default::default())?;
let builder = register_browser_tools(builder, &browser);
```

**方式二：手动注册特定工具**

```rust
use phi_tools::{BrowserToolset, BrowserNavigateTool, BrowserClickTool, BrowserScreenshotTool};

let browser = BrowserToolset::launch(Default::default())?;
let session = browser.session();

builder
    .register_tool(BrowserNavigateTool::new(session.clone()))
    .register_tool(BrowserClickTool::new(session.clone()))
    .register_tool(BrowserScreenshotTool::new(session.clone()));
```

**方式三：连接到已有的浏览器实例**

```rust
use phi_tools::{BrowserToolset, ConnectionOptions};

let opts = ConnectionOptions {
    url: Some("http://localhost:9222".into()),
    ..Default::default()
};
let browser = BrowserToolset::connect(opts)?;
```

#### 配置

```rust
use phi_tools::{BrowserToolset, BrowserLaunchOptions, LaunchOptions};

// 使用便捷别名
let opts = BrowserLaunchOptions::default();  // 等同于 LaunchOptions

// 或详细配置
let opts = LaunchOptions {
    headless: false,
    window_size: Some((1920, 1080)),
    ..Default::default()
};

let browser = BrowserToolset::launch(opts)?;
```

## License

MIT

[English](README.md)
