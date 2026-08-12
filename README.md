# phi-tools

[![Crates.io](https://img.shields.io/crates/v/phi-tools.svg)](https://crates.io/crates/phi-tools)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

General-purpose Agent toolset for the [agent-base](https://crates.io/crates/agent-base) runtime.

Each tool independently implements the `agent_base::Tool` trait. Consumers register tools with `AgentBuilder` on demand.

[中文文档](README_CN.md)

## Features

| Feature | Description | Extra deps |
|---------|-------------|------------|
| *(default)* | `LocalShellTool` — execute shell commands | — |
| `browser` | 22 browser automation tools via Chrome DevTools Protocol (CDP) | `headless_chrome`, `html2md` |

```toml
[dependencies]
# Shell only
phi-tools = "0.1.5"

# With browser automation
phi-tools = { version = "0.1.5", features = ["browser"] }
```

## Tools

### Local Shell

| Tool | Function Name | Description |
|------|---------------|-------------|
| `LocalShellTool` | `execute_command` | Execute shell commands via `sh -c`, with timeout and cancellation |

```rust
use phi_tools::LocalShellTool;

builder.register_tool(LocalShellTool::new(30_000));  // 30s timeout
```

### Browser Automation (feature = `browser`)

22 CDP-based tools for full browser control:

**Navigation & Interaction:**

| Tool | Function Name | Description |
|------|---------------|-------------|
| `BrowserNavigateTool` | `browser_navigate` | Navigate to a URL |
| `BrowserClickTool` | `browser_click` | Click an element by selector or coordinates |
| `BrowserInputTool` | `browser_input` | Type text into an input field |
| `BrowserScrollTool` | `browser_scroll` | Scroll the page |
| `BrowserHoverTool` | `browser_hover` | Hover over an element |
| `BrowserSelectTool` | `browser_select` | Select an option from a dropdown |
| `BrowserPressKeyTool` | `browser_press_key` | Press a keyboard key |

**Content Extraction:**

| Tool | Function Name | Description |
|------|---------------|-------------|
| `BrowserScreenshotTool` | `browser_screenshot` | Take a screenshot of the page |
| `BrowserSnapshotTool` | `browser_snapshot` | Capture the page accessibility tree |
| `BrowserEvaluateTool` | `browser_evaluate` | Execute JavaScript in the page |
| `BrowserExtractTool` | `browser_extract` | Extract structured content from the page |
| `BrowserGetMarkdownTool` | `browser_get_markdown` | Convert the page to Markdown |
| `BrowserReadLinksTool` | `browser_read_links` | List all links on the page |

**Tab Management:**

| Tool | Function Name | Description |
|------|---------------|-------------|
| `BrowserGoBackTool` | `browser_go_back` | Navigate back in history |
| `BrowserGoForwardTool` | `browser_go_forward` | Navigate forward in history |
| `BrowserNewTabTool` | `browser_new_tab` | Open a new tab |
| `BrowserCloseTabTool` | `browser_close_tab` | Close a tab |
| `BrowserSwitchTabTool` | `browser_switch_tab` | Switch to a different tab |
| `BrowserTabListTool` | `browser_tab_list` | List all open tabs |

**Lifecycle:**

| Tool | Function Name | Description |
|------|---------------|-------------|
| `BrowserWaitTool` | `browser_wait` | Wait for a condition or timeout |
| `BrowserCloseTool` | `browser_close` | Close the browser |
| `BrowserRestartTool` | `browser_restart` | Restart the browser (recovers from crashes) |

#### Usage

**Option 1: `register_browser_tools` helper (register all 22 tools at once)**

```rust
use phi_tools::{BrowserToolset, register_browser_tools};

let browser = BrowserToolset::launch(Default::default())?;
let builder = register_browser_tools(builder, &browser);
```

**Option 2: Manual registration of specific tools**

```rust
use phi_tools::{BrowserToolset, BrowserNavigateTool, BrowserClickTool, BrowserScreenshotTool};

let browser = BrowserToolset::launch(Default::default())?;
let session = browser.session();

builder
    .register_tool(BrowserNavigateTool::new(session.clone()))
    .register_tool(BrowserClickTool::new(session.clone()))
    .register_tool(BrowserScreenshotTool::new(session.clone()));
```

**Option 3: Connect to an existing browser**

```rust
use phi_tools::{BrowserToolset, ConnectionOptions};

let opts = ConnectionOptions {
    url: Some("http://localhost:9222".into()),
    ..Default::default()
};
let browser = BrowserToolset::connect(opts)?;
```

#### Configuration

```rust
use phi_tools::{BrowserToolset, BrowserLaunchOptions, LaunchOptions};

// Ergonomic aliases available
let opts = BrowserLaunchOptions::default();  // same as LaunchOptions

// Or configure in detail
let opts = LaunchOptions {
    headless: false,
    window_size: Some((1920, 1080)),
    ..Default::default()
};

let browser = BrowserToolset::launch(opts)?;
```

## License

MIT

[中文文档](README_CN.md)
