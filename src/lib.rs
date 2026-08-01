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
    config::{ConnectionOptions, LaunchOptions},
    tools::{
        BrowserClickTool, BrowserCloseTabTool, BrowserCloseTool, BrowserEvaluateTool,
        BrowserExtractTool, BrowserGetMarkdownTool, BrowserGoBackTool, BrowserGoForwardTool,
        BrowserHoverTool, BrowserInputTool, BrowserNavigateTool, BrowserNewTabTool,
        BrowserPressKeyTool, BrowserReadLinksTool, BrowserScreenshotTool, BrowserScrollTool,
        BrowserSelectTool, BrowserSnapshotTool, BrowserSwitchTabTool, BrowserTabListTool,
        BrowserWaitTool,
    },
    BrowserToolset,
};
