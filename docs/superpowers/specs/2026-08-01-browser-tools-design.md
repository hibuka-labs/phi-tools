# Browser Tools for phi-tools

**Date:** 2026-08-01
**Status:** Draft

## 1. Overview

Add a `browser` module to `phi-tools`, gated behind a `browser` Cargo feature, providing 21 browser automation tools via Chrome DevTools Protocol (CDP). Each tool implements `agent_base::Tool`, enabling phi-agent consumers to give AI agents the ability to view and interact with web pages.

Core CDP/DOM logic is ported from `browser-use-rs`, while the Tool trait layer is adapted to `agent_base::Tool` following the API conventions established in phi-agent's `browser-tools` branch.

## 2. Motivation

- **phi-agent should be able to browse the web.** This is a critical capability for general-purpose agents — reading docs, scraping data, filling forms, testing web apps.
- **The implementation already existed.** phi-tools previously had a `browser-tools` branch (now deleted from remote). phi-agent's `browser-tools` branch still documents the expected API shape.
- **browser-use-rs provides battle-tested CDP/DOM logic.** Reuse rather than rewrite.

## 3. Architecture

### 3.1 Module Layout

```
phi-tools/
├── Cargo.toml
├── src/
│   ├── lib.rs                       # Feature-gated re-exports
│   ├── local_shell.rs               # Existing — unchanged
│   └── browser/                     # New — #[cfg(feature = "browser")]
│       ├── mod.rs                   # BrowserToolset + re-exports
│       ├── session.rs               # BrowserSession (CDP lifecycle)
│       ├── dom.rs                   # DomTree + AriaNode
│       ├── config.rs                # LaunchOptions / ConnectionOptions
│       ├── extract_dom.js           # Injected JS for DOM extraction
│       └── tools/
│           ├── mod.rs               # Re-exports all tool structs
│           ├── utils.rs             # URL normalization, ARIA rendering
│           ├── navigate.rs
│           ├── go_back.rs
│           ├── go_forward.rs
│           ├── wait.rs
│           ├── click.rs
│           ├── hover.rs
│           ├── input.rs
│           ├── select.rs
│           ├── press_key.rs
│           ├── scroll.rs
│           ├── screenshot.rs
│           ├── snapshot.rs
│           ├── evaluate.rs
│           ├── extract.rs
│           ├── markdown.rs
│           ├── read_links.rs
│           ├── new_tab.rs
│           ├── tab_list.rs
│           ├── switch_tab.rs
│           ├── close_tab.rs
│           └── close.rs
```

### 3.2 Dependency Graph

```
phi-agent (CLI binary)
  └─ registers tools via agent_base::AgentBuilder
phi-tools (this crate)
  ├─ agent_base::Tool trait          ← each tool implements this
  ├─ headless_chrome (Browser, Tab)  ← CDP browser control
  ├─ html2md                         ← HTML → Markdown conversion
  └─ tokio (spawn_blocking)          ← async/sync bridge
```

### 3.3 Component Responsibilities

| Component | Role |
|---|---|
| **BrowserToolset** | Owns `BrowserSession`. Entry point for consumers. Exposes `launch()` and `connect()`. |
| **BrowserSession** | Wraps `headless_chrome::Browser`. Tab management, navigation, DOM extraction, element finding. No awareness of phi-tools or agent infrastructure. |
| **DomTree / AriaNode** | Parsed ARIA accessibility tree extracted from the page. Indexes interactive elements for AI-friendly targeting. Serialized to JSON-friendly structures. |
| **Each Tool struct** | Thin wrapper: receives `Arc<Mutex<BrowserSession>>`, implements `agent_base::Tool`, dispatches to `BrowserSession` methods inside `spawn_blocking`. |

## 4. Key Design Decisions

### 4.1 Async/Sync Bridge: `tokio::task::spawn_blocking`

`headless_chrome` APIs are all synchronous. `agent_base::Tool::call` is async. We use `spawn_blocking` to bridge the gap:

```
agent loop (async worker)     tokio blocking pool
     │                              │
     ├── call(args) ───────────────>│
     │                              ├── session.lock()
     │                              ├── CDP operations (sync)
     │                              ├── DOM extraction
     │                              ├── construct ToolOutput
     │                              └── return
     │<─────────────────────────────│
```

**Rationale:** In complex deployment scenarios (web server with concurrent agent sessions, multi-agent parallelism), direct sync calls would starve tokio worker threads during slow operations (e.g., `wait_until_navigated` may take 30s+). `spawn_blocking` isolates blocking operations to the dedicated blocking thread pool (default 512 threads), keeping async workers free.

**Cost:** One extra thread per tool call. Negligible compared to CDP operation latency (100ms–30s).

**JoinError handling:** If the blocking closure panics, `spawn_blocking` returns `JoinError`. Each tool maps this to `AgentError::Internal` and the agent can decide whether to retry.

### 4.2 Thread Safety: `Arc<Mutex<BrowserSession>>`

`BrowserSession` is wrapped in `Arc<Mutex<...>>` for two reasons:

1. `headless_chrome::Browser` is not `Sync` — needs mutex protection.
2. Multiple tools share the same session instance — they need shared ownership.

`std::sync::Mutex` is sufficient (no async hold). Lock contention is not a concern: agent runtime executes tools serially (one tool at a time per session).

### 4.3 Error Handling: Summary Strings, Not Structured Errors

- **Internal layer** (BrowserSession, DomTree): Uses `anyhow::Result` and `anyhow::bail!` for convenience.
- **Tool layer**: All errors are caught inside `spawn_blocking` and converted to `ToolOutput` with a descriptive `summary` string. The agent reads `summary` to understand what went wrong and can self-correct (e.g., "element not found at index 5, try refreshing the snapshot" → agent calls `browser_snapshot` again).
- **`control_flow`**: Always `ToolControlFlow::Continue` — browser errors are recoverable; the agent should try alternative approaches rather than aborting.
- **JoinError** (panic in blocking closure): Returned as `AgentError::Internal`. This is a genuine bug, not a recoverable browser error.

### 4.4 Feature Flag: `browser`

```toml
[features]
default = []
browser = ["headless_chrome", "tokio/rt-multi-thread", "html2md"]

[dependencies]
headless_chrome = { version = "1.0", optional = true }
html2md = { version = "0.2", optional = true }
```

- `headless_chrome` is the heavyweight dependency (~Chrome/Chromium required at runtime).
- `html2md` is needed for `browser_get_markdown`.
- `tokio/rt-multi-thread` is required for `spawn_blocking` (not always pulled by default in library crates).
- No sub-features — markdown conversion is a core viewing capability; splitting would add complexity without benefit.

### 4.5 Tool Naming Convention: `browser_` prefix

All tools use the `browser_` prefix (e.g., `browser_navigate`, `browser_click`). This follows the convention in phi-agent's browser-tools branch and avoids name collisions with other tool providers (shell, filesystem, etc.).

### 4.6 Snapshot-Based Interaction

Following browser-use-rs's Playwright-inspired approach:

1. `browser_navigate` / `browser_snapshot` return an ARIA tree snapshot with numbered interactive elements.
2. AI selects elements by index (e.g., `browser_click index=5`).
3. Under the hood, indices map to CSS selectors extracted during DOM parsing.
4. This avoids AI needing to write fragile CSS/XPath selectors.

## 5. Tool Catalog (21 tools)

### Navigation (4)

| Tool | Key Params | Description |
|---|---|---|
| `browser_navigate` | `url` | Navigate to URL, return page snapshot |
| `browser_go_back` | — | Navigate back in history |
| `browser_go_forward` | — | Navigate forward in history |
| `browser_wait` | `timeout_ms` | Wait for a duration |

### Interaction (6)

| Tool | Key Params | Description |
|---|---|---|
| `browser_click` | `index` or `selector` | Click element |
| `browser_hover` | `index` or `selector` | Hover over element |
| `browser_input_fill` | `index`/`selector`, `text` | Type text into input |
| `browser_select` | `index`/`selector`, `value` | Select dropdown option |
| `browser_press_key` | `key` | Press keyboard key |
| `browser_scroll` | `direction`, `amount` | Scroll page |

### Viewing (5)

| Tool | Key Params | Description |
|---|---|---|
| `browser_snapshot` | — | Get ARIA snapshot with indexed elements |
| `browser_screenshot` | — | Capture page screenshot (base64 PNG) |
| `browser_get_markdown` | — | Convert page to Markdown |
| `browser_read_links` | — | Extract all links from page |
| `browser_evaluate` | `script` | Execute JavaScript, return result |

### Tab Management (4)

| Tool | Key Params | Description |
|---|---|---|
| `browser_new_tab` | `url` | Open new tab, navigate to URL |
| `browser_tab_list` | — | List all tabs (titles, URLs) |
| `browser_switch_tab` | `index` | Switch to tab by index |
| `browser_close_tab` | — | Close current tab |

### Content Extraction (1)

| Tool | Key Params | Description |
|---|---|---|
| `browser_extract_content` | `include_html` | Extract text/HTML from page |

### Control (1)

| Tool | Key Params | Description |
|---|---|---|
| `browser_close` | — | Close browser, end session |

## 6. Tool Implementation Template

Every tool follows the same pattern:

```rust
use std::sync::{Arc, Mutex};
use agent_base::{AgentResult, Tool, ToolContext, ToolControlFlow, ToolOutput};
use async_trait::async_trait;
use serde_json::{Value, json};
use crate::browser::session::BrowserSession;

pub struct BrowserNavigateTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserNavigateTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserNavigateTool {
    fn name(&self) -> &'static str {
        "browser_navigate"
    }

    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "browser_navigate",
                "description": "Navigate to a URL in the browser. Returns a page snapshot with numbered interactive elements.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to navigate to"
                        }
                    },
                    "required": ["url"]
                }
            }
        })
    }

    async fn call(&self, args: &Value, _ctx: &ToolContext) -> AgentResult<ToolOutput> {
        let session = self.session.clone();
        let url = args["url"].as_str().unwrap_or("").to_string();

        tokio::task::spawn_blocking(move || {
            let session = session.lock().unwrap();
            match session.navigate(&url) {
                Ok(_) => {
                    let dom = session.extract_dom()?;
                    let snapshot = render_aria_tree(&dom);
                    Ok(ToolOutput {
                        summary: snapshot,
                        raw: Some(json!({"url": url, "success": true})),
                        control_flow: ToolControlFlow::Continue,
                        truncation: None,
                    })
                }
                Err(e) => Ok(ToolOutput {
                    summary: format!("Failed to navigate: {}", e),
                    raw: Some(json!({"url": url, "success": false, "error": e.to_string()})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                })
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(
            format!("browser_navigate: blocking task panicked: {}", e)
        ))?
    }
}
```

## 7. Porting from browser-use-rs

### What We Port (with minimal changes)

| Source File | Target | Changes |
|---|---|---|
| `browser/config.rs` | `browser/config.rs` | None — pure data structs |
| `browser/session.rs` | `browser/session.rs` | Remove `ToolRegistry` member, remove `tool_registry()`/`execute_tool()` methods |
| `dom/element.rs` | `browser/dom.rs` | Merge tree.rs + element.rs → single dom.rs |
| `dom/tree.rs` | `browser/dom.rs` | Same as above |
| `dom/extract_dom.js` | `browser/extract_dom.js` | Copy verbatim |
| `tools/utils.rs` | `browser/tools/utils.rs` | Keep `normalize_url()`, port ARIA rendering functions |
| `tools/snapshot.rs` (ARIA rendering) | `browser/tools/utils.rs` | ARIA tree → text snapshot rendering |

### What We DON'T Port

| Source | Reason |
|---|---|
| `tools/mod.rs` — `Tool` trait, `ToolRegistry`, `ToolResult`, `DynTool` | Replaced by `agent_base::Tool` + `AgentBuilder` |
| `mcp/` module | MCP is handled upstream |
| `mcp/handler.rs` — `BrowserServer` | Replaced by `BrowserToolset` |
| `error.rs` — `BrowserError` enum | Replaced by `anyhow::Result` + string summaries |
| `Readability.min.js` / `convert_to_markdown.js` | Fallback JS scripts; `html2md` crate handles markdown conversion |
| `hover.js`, `scroll.js`, `select.js` | Tool-specific JS; port logic inline or use CDP equivalents via headless_chrome |

## 8. Phi-Agent Integration (Future PR)

Once phi-tools' browser module is published, phi-agent's integration requires:

1. **`Cargo.toml`**: Enable `browser` feature on `phi-tools`.
2. **`src/bin/phi/tools/mod.rs`**: Add `BrowserToolset` and browser tool re-exports.
3. **`src/bin/phi/args.rs`**: Add `--enable-browser`, `--headed`, `--connect-ws` CLI flags.
4. **`src/bin/phi/main.rs`**: Create `BrowserToolset`, call `register_browser_tools()`.

This follows the exact pattern in phi-agent's `browser-tools` branch and is out of scope for this phi-tools change.

## 9. Testing Strategy

- **Unit tests**: `DomTree` construction and traversal, `AriaNode` methods, URL normalization.
- **Integration tests** (ignored by default): Require Chrome installed. Test `BrowserSession::launch()`, navigate to `about:blank`, extract DOM, click elements.
- **No MCP/E2E tests** in phi-tools — those belong in phi-agent integration tests.

## 10. Risks and Mitigations

| Risk | Mitigation |
|---|---|
| `headless_chrome` API changes break build | Pin to `1.0.x`; the crate is stable |
| Chrome not installed at runtime | `BrowserSession::launch` returns descriptive error; agent treats as recoverable |
| `spawn_blocking` performance overhead | Extra ~0.1ms per call; negligible vs. CDP latency |
| DOM extraction JS breaks on some sites | Fallback: return error summary; agent can use `browser_evaluate` or `browser_screenshot` instead |
