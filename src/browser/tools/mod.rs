//! Browser automation tools — re-exports.
//!
//! Each tool implements `agent_base::Tool` and holds an `Arc<Mutex<BrowserSession>>`.
//! Tools are registered individually by consumers via `AgentBuilder::register_tool`.

mod utils;

mod click;
mod close;
mod close_tab;
mod evaluate;
mod extract;
mod go_back;
mod go_forward;
mod hover;
mod input;
mod markdown;
mod navigate;
mod new_tab;
mod press_key;
mod read_links;
mod restart;
mod screenshot;
mod scroll;
mod select;
mod snapshot;
mod switch_tab;
mod tab_list;
mod wait;

pub use click::BrowserClickTool;
pub use close::BrowserCloseTool;
pub use close_tab::BrowserCloseTabTool;
pub use evaluate::BrowserEvaluateTool;
pub use extract::BrowserExtractTool;
pub use go_back::BrowserGoBackTool;
pub use go_forward::BrowserGoForwardTool;
pub use hover::BrowserHoverTool;
pub use input::BrowserInputTool;
pub use markdown::BrowserGetMarkdownTool;
pub use navigate::BrowserNavigateTool;
pub use new_tab::BrowserNewTabTool;
pub use press_key::BrowserPressKeyTool;
pub use read_links::BrowserReadLinksTool;
pub use restart::BrowserRestartTool;
pub use screenshot::BrowserScreenshotTool;
pub use scroll::BrowserScrollTool;
pub use select::BrowserSelectTool;
pub use snapshot::BrowserSnapshotTool;
pub use switch_tab::BrowserSwitchTabTool;
pub use tab_list::BrowserTabListTool;
pub use wait::BrowserWaitTool;
