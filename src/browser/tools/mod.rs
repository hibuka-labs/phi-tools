//! Browser automation tools — re-exports.
//!
//! Each tool implements `agent_base::Tool` and holds an `Arc<Mutex<BrowserSession>>`.
//! Tools are registered individually by consumers via `AgentBuilder::register_tool`.

mod utils;

mod navigate;
mod go_back;
mod go_forward;
mod wait;
mod click;
mod hover;
mod input;
mod select;
mod press_key;
mod scroll;
mod screenshot;
mod snapshot;
mod evaluate;
mod extract;
mod markdown;
mod read_links;
mod new_tab;
mod tab_list;
mod switch_tab;
mod close_tab;
mod close;

pub use navigate::BrowserNavigateTool;
pub use go_back::BrowserGoBackTool;
pub use go_forward::BrowserGoForwardTool;
pub use wait::BrowserWaitTool;
pub use click::BrowserClickTool;
pub use hover::BrowserHoverTool;
pub use input::BrowserInputTool;
pub use select::BrowserSelectTool;
pub use press_key::BrowserPressKeyTool;
pub use scroll::BrowserScrollTool;
pub use screenshot::BrowserScreenshotTool;
pub use snapshot::BrowserSnapshotTool;
pub use evaluate::BrowserEvaluateTool;
pub use extract::BrowserExtractTool;
pub use markdown::BrowserGetMarkdownTool;
pub use read_links::BrowserReadLinksTool;
pub use new_tab::BrowserNewTabTool;
pub use tab_list::BrowserTabListTool;
pub use switch_tab::BrowserSwitchTabTool;
pub use close_tab::BrowserCloseTabTool;
pub use close::BrowserCloseTool;
