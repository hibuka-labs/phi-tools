//! phi-tools: General-purpose Agent toolset — local shell, file operations,
//! and browser automation (optional, gated behind the `browser` Cargo feature).
//!
//! Each tool independently implements the `agent_base::Tool` trait.
//! Consumers register tools with `AgentBuilder` on demand.
//!
//! # Example
//!
//! ```ignore
//! use phi_tools::LocalShellTool;
//! builder.register_tool(LocalShellTool::new(30_000));
//!
//! // With browser feature enabled:
//! #[cfg(feature = "browser")]
//! {
//!     use phi_tools::{BrowserToolset, BrowserNavigateTool};
//!     let browser = BrowserToolset::launch(Default::default())?;
//!     let session = browser.session();
//!     builder.register_tool(BrowserNavigateTool::new(session.clone()));
//! }
//! ```

pub mod local_shell;
pub use local_shell::LocalShellTool;

#[cfg(feature = "browser")]
pub mod browser;

#[cfg(feature = "browser")]
pub use browser::{
    BrowserToolset,
    config::{ConnectionOptions, LaunchOptions},
    tools::{
        BrowserClickTool, BrowserCloseTabTool, BrowserCloseTool, BrowserEvaluateTool,
        BrowserExtractTool, BrowserGetMarkdownTool, BrowserGoBackTool, BrowserGoForwardTool,
        BrowserHoverTool, BrowserInputTool, BrowserNavigateTool, BrowserNewTabTool,
        BrowserPressKeyTool, BrowserReadLinksTool, BrowserRestartTool, BrowserScreenshotTool,
        BrowserScrollTool, BrowserSelectTool, BrowserSnapshotTool, BrowserSwitchTabTool,
        BrowserTabListTool, BrowserWaitTool,
    },
};

// ── Ergonomic aliases ──

/// Alias for [`LaunchOptions`] — more self-documenting in CLI code.
#[cfg(feature = "browser")]
pub type BrowserLaunchOptions = LaunchOptions;

/// Alias for [`ConnectionOptions`].
#[cfg(feature = "browser")]
pub type BrowserConnectionOptions = ConnectionOptions;

/// Register all 21 browser-automation tools on the builder.
#[cfg(feature = "browser")]
pub fn register_browser_tools(
    builder: agent_base::AgentBuilder,
    browser: &BrowserToolset,
) -> agent_base::AgentBuilder {
    let session = browser.session();
    builder
        .register_tool(BrowserNavigateTool::new(session.clone()))
        .register_tool(BrowserClickTool::new(session.clone()))
        .register_tool(BrowserInputTool::new(session.clone()))
        .register_tool(BrowserScrollTool::new(session.clone()))
        .register_tool(BrowserHoverTool::new(session.clone()))
        .register_tool(BrowserSelectTool::new(session.clone()))
        .register_tool(BrowserPressKeyTool::new(session.clone()))
        .register_tool(BrowserScreenshotTool::new(session.clone()))
        .register_tool(BrowserSnapshotTool::new(session.clone()))
        .register_tool(BrowserEvaluateTool::new(session.clone()))
        .register_tool(BrowserExtractTool::new(session.clone()))
        .register_tool(BrowserGetMarkdownTool::new(session.clone()))
        .register_tool(BrowserReadLinksTool::new(session.clone()))
        .register_tool(BrowserGoBackTool::new(session.clone()))
        .register_tool(BrowserGoForwardTool::new(session.clone()))
        .register_tool(BrowserNewTabTool::new(session.clone()))
        .register_tool(BrowserCloseTabTool::new(session.clone()))
        .register_tool(BrowserSwitchTabTool::new(session.clone()))
        .register_tool(BrowserTabListTool::new(session.clone()))
        .register_tool(BrowserWaitTool::new(session.clone()))
        .register_tool(BrowserCloseTool::new(session.clone()))
        .register_tool(BrowserRestartTool::new(session.clone()))
}
