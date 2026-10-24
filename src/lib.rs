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
    groups::{BrowserToolGroup, browser_tool_names},
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

/// The 22 browser tools as a list — the whole suite, in registration order.
///
/// Builders that wrap `agent_base::AgentBuilder` rather than being one (the
/// product shells) can register these with their own `register_tool_arc`;
/// [`register_browser_tools`] is the same set for the plain base builder.
/// Scenarios that only need part of the suite should register a subset —
/// see [`browser_tools_subset`] / [`browser_tools_groups`]; every tool schema
/// registered is paid for on every LLM request.
#[cfg(feature = "browser")]
pub fn browser_tools(browser: &BrowserToolset) -> Vec<std::sync::Arc<dyn agent_base::Tool>> {
    browser::groups::construct_groups(browser.session(), &[browser::groups::BrowserToolGroup::All])
}

/// The browser tools of one usage scenario ([`BrowserToolGroup::All`] is the
/// full suite — same list as [`browser_tools`]).
#[cfg(feature = "browser")]
pub fn browser_tools_subset(
    browser: &BrowserToolset,
    group: BrowserToolGroup,
) -> Vec<std::sync::Arc<dyn agent_base::Tool>> {
    browser_tools_groups(browser, &[group])
}

/// The browser tools of several usage scenarios, deduped, in registration
/// order. An empty slice yields no tools; a slice containing
/// [`BrowserToolGroup::All`] yields the full suite.
#[cfg(feature = "browser")]
pub fn browser_tools_groups(
    browser: &BrowserToolset,
    groups: &[BrowserToolGroup],
) -> Vec<std::sync::Arc<dyn agent_base::Tool>> {
    browser::groups::construct_groups(browser.session(), groups)
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
