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

/// The 22 browser tools as a list.
///
/// Builders that wrap `agent_base::AgentBuilder` rather than being one (the
/// product shells) can register these with their own `register_tool_arc`;
/// [`register_browser_tools`] is the same set for the plain base builder.
#[cfg(feature = "browser")]
pub fn browser_tools(browser: &BrowserToolset) -> Vec<std::sync::Arc<dyn agent_base::Tool>> {
    use std::sync::Arc;

    let session = browser.session();
    vec![
        Arc::new(BrowserNavigateTool::new(session.clone())),
        Arc::new(BrowserClickTool::new(session.clone())),
        Arc::new(BrowserInputTool::new(session.clone())),
        Arc::new(BrowserScrollTool::new(session.clone())),
        Arc::new(BrowserHoverTool::new(session.clone())),
        Arc::new(BrowserSelectTool::new(session.clone())),
        Arc::new(BrowserPressKeyTool::new(session.clone())),
        Arc::new(BrowserScreenshotTool::new(session.clone())),
        Arc::new(BrowserSnapshotTool::new(session.clone())),
        Arc::new(BrowserEvaluateTool::new(session.clone())),
        Arc::new(BrowserExtractTool::new(session.clone())),
        Arc::new(BrowserGetMarkdownTool::new(session.clone())),
        Arc::new(BrowserReadLinksTool::new(session.clone())),
        Arc::new(BrowserGoBackTool::new(session.clone())),
        Arc::new(BrowserGoForwardTool::new(session.clone())),
        Arc::new(BrowserNewTabTool::new(session.clone())),
        Arc::new(BrowserCloseTabTool::new(session.clone())),
        Arc::new(BrowserSwitchTabTool::new(session.clone())),
        Arc::new(BrowserTabListTool::new(session.clone())),
        Arc::new(BrowserWaitTool::new(session.clone())),
        Arc::new(BrowserCloseTool::new(session.clone())),
        Arc::new(BrowserRestartTool::new(session.clone())),
    ]
}

/// Register all 22 browser-automation tools on the builder.
#[cfg(feature = "browser")]
pub fn register_browser_tools(
    builder: agent_base::AgentBuilder,
    browser: &BrowserToolset,
) -> agent_base::AgentBuilder {
    browser_tools(browser)
        .into_iter()
        .fold(builder, |builder, tool| builder.register_tool_arc(tool))
}
