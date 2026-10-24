//! Usage-scenario grouping for the 22 CDP browser tools (tool-face loading).
//!
//! Grouping is by *use case*, not by parameter count, so a product shell can
//! compose the tool face a scenario needs without knowing tool details:
//! the design-canvas self-check only ever needs [`BrowserToolGroup::Canvas`]
//! (`navigate` / `screenshot` / `click` / `snapshot` / `close`), while general
//! browser automation takes [`BrowserToolGroup::All`].
//!
//! [`REGISTRY`] is the single source of truth: one row per tool carrying its
//! group, its wire name, and its constructor. Names, group membership, and
//! construction all derive from the same table, so a new tool cannot appear in
//! one place and drift from another.

use std::sync::{Arc, Mutex};

use agent_base::Tool;

use super::session::BrowserSession;
use super::tools::*;

/// Which usage scenario a browser tool belongs to.
///
/// Product shells compose groups per scenario (`browser_tools_groups`);
/// [`BrowserToolGroup::All`] is the whole suite and the historical default of
/// [`crate::browser_tools`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum BrowserToolGroup {
    /// Canvas self-check minimum: open the page, look at it, click an option,
    /// close. `browser_navigate` / `browser_screenshot` / `browser_click` /
    /// `browser_snapshot` / `browser_close`.
    Canvas,
    /// Tab management: `browser_new_tab` / `browser_switch_tab` /
    /// `browser_close_tab` / `browser_tab_list`.
    Tabs,
    /// Form filling and page interaction: `browser_input_fill` /
    /// `browser_select` / `browser_hover` / `browser_press_key` /
    /// `browser_scroll`.
    Forms,
    /// Content extraction: `browser_extract_content` / `browser_get_markdown`
    /// / `browser_read_links` / `browser_evaluate`.
    Extract,
    /// Session flow control: `browser_go_back` / `browser_go_forward` /
    /// `browser_wait` / `browser_restart`.
    Session,
    /// Every browser tool (= the union of the groups above).
    All,
}

/// A tool row: scenario group, wire name, and constructor.
type BrowserToolRow = (
    BrowserToolGroup,
    &'static str,
    fn(Arc<Mutex<BrowserSession>>) -> Arc<dyn Tool>,
);

/// The canonical 22-tool registry, in registration order.
///
/// This is the only place a browser tool is listed. Everything else —
/// [`crate::browser_tools`], [`browser_tool_names`], subset construction — is
/// derived from this table, and [`tests::registry_matches_constructed_suite`]
/// pins the constructed suite to it.
fn registry() -> [BrowserToolRow; 22] {
    [
        (BrowserToolGroup::Canvas, "browser_navigate", |s| {
            Arc::new(BrowserNavigateTool::new(s))
        }),
        (BrowserToolGroup::Canvas, "browser_click", |s| {
            Arc::new(BrowserClickTool::new(s))
        }),
        (BrowserToolGroup::Forms, "browser_input_fill", |s| {
            Arc::new(BrowserInputTool::new(s))
        }),
        (BrowserToolGroup::Forms, "browser_scroll", |s| {
            Arc::new(BrowserScrollTool::new(s))
        }),
        (BrowserToolGroup::Forms, "browser_hover", |s| {
            Arc::new(BrowserHoverTool::new(s))
        }),
        (BrowserToolGroup::Forms, "browser_select", |s| {
            Arc::new(BrowserSelectTool::new(s))
        }),
        (BrowserToolGroup::Forms, "browser_press_key", |s| {
            Arc::new(BrowserPressKeyTool::new(s))
        }),
        (BrowserToolGroup::Canvas, "browser_screenshot", |s| {
            Arc::new(BrowserScreenshotTool::new(s))
        }),
        (BrowserToolGroup::Canvas, "browser_snapshot", |s| {
            Arc::new(BrowserSnapshotTool::new(s))
        }),
        (BrowserToolGroup::Extract, "browser_evaluate", |s| {
            Arc::new(BrowserEvaluateTool::new(s))
        }),
        (BrowserToolGroup::Extract, "browser_extract_content", |s| {
            Arc::new(BrowserExtractTool::new(s))
        }),
        (BrowserToolGroup::Extract, "browser_get_markdown", |s| {
            Arc::new(BrowserGetMarkdownTool::new(s))
        }),
        (BrowserToolGroup::Extract, "browser_read_links", |s| {
            Arc::new(BrowserReadLinksTool::new(s))
        }),
        (BrowserToolGroup::Session, "browser_go_back", |s| {
            Arc::new(BrowserGoBackTool::new(s))
        }),
        (BrowserToolGroup::Session, "browser_go_forward", |s| {
            Arc::new(BrowserGoForwardTool::new(s))
        }),
        (BrowserToolGroup::Tabs, "browser_new_tab", |s| {
            Arc::new(BrowserNewTabTool::new(s))
        }),
        (BrowserToolGroup::Tabs, "browser_close_tab", |s| {
            Arc::new(BrowserCloseTabTool::new(s))
        }),
        (BrowserToolGroup::Tabs, "browser_switch_tab", |s| {
            Arc::new(BrowserSwitchTabTool::new(s))
        }),
        (BrowserToolGroup::Tabs, "browser_tab_list", |s| {
            Arc::new(BrowserTabListTool::new(s))
        }),
        (BrowserToolGroup::Session, "browser_wait", |s| {
            Arc::new(BrowserWaitTool::new(s))
        }),
        (BrowserToolGroup::Canvas, "browser_close", |s| {
            Arc::new(BrowserCloseTool::new(s))
        }),
        (BrowserToolGroup::Session, "browser_restart", |s| {
            Arc::new(BrowserRestartTool::new(s))
        }),
    ]
}

impl BrowserToolGroup {
    /// Tool names in this group, in registration order.
    ///
    /// [`BrowserToolGroup::All`] is the whole registry — the same names
    /// [`crate::browser_tools`] constructs.
    pub fn tool_names(self) -> Vec<&'static str> {
        registry()
            .iter()
            .filter(|(g, _, _)| self == BrowserToolGroup::All || *g == self)
            .map(|(_, name, _)| *name)
            .collect()
    }

    /// Whether `name` belongs to this group (`All` contains every browser
    /// tool; anything not in the registry belongs to no group).
    pub fn contains(self, name: &str) -> bool {
        registry()
            .iter()
            .any(|(g, n, _)| *n == name && (self == BrowserToolGroup::All || *g == self))
    }
}

/// Names of the tools in `groups`, deduped, in registration order.
///
/// Passing overlapping groups is fine — each name appears once. A slice
/// containing [`BrowserToolGroup::All`] is the whole suite.
pub fn browser_tool_names(groups: &[BrowserToolGroup]) -> Vec<&'static str> {
    let all = groups.contains(&BrowserToolGroup::All);
    registry()
        .iter()
        .filter(|(g, _, _)| all || groups.contains(g))
        .map(|(_, name, _)| *name)
        .collect()
}

/// Construct the tools belonging to `groups`, in registration order.
pub(crate) fn construct_groups(
    session: Arc<Mutex<BrowserSession>>,
    groups: &[BrowserToolGroup],
) -> Vec<Arc<dyn Tool>> {
    let all = groups.contains(&BrowserToolGroup::All);
    registry()
        .into_iter()
        .filter(|(g, _, _)| all || groups.contains(g))
        .map(|(_, _, ctor)| ctor(session.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lazy_session() -> Arc<Mutex<BrowserSession>> {
        crate::BrowserToolset::lazy(crate::LaunchOptions::default()).session()
    }

    /// The suite `crate::browser_tools` constructs must equal the registry
    /// rows exactly — same tools, same names, same order. This is what makes
    /// the name tables safe to trust without listing tool names anywhere else.
    #[test]
    fn registry_matches_constructed_suite() {
        let session = lazy_session();
        let constructed =
            crate::browser_tools(&crate::BrowserToolset::lazy(crate::LaunchOptions::default()));
        let rows = registry();
        assert_eq!(
            constructed.len(),
            rows.len(),
            "one row per constructed tool"
        );
        for (tool, (group, name, _)) in constructed.iter().zip(rows.iter()) {
            assert_eq!(tool.name(), *name, "registry name must match the tool");
            assert!(
                group.contains(tool.name()),
                "registry group {group:?} must contain {}",
                tool.name()
            );
            // The constructor in the row is what `construct_groups` runs —
            // a same-name check keeps both construction paths in step.
            let via_row = construct_groups(session.clone(), &[*group])
                .into_iter()
                .find(|t| t.name() == *name)
                .expect("row constructor must yield its own tool");
            assert_eq!(via_row.name(), *name);
        }
    }

    #[test]
    fn canvas_group_is_the_five_tool_minimum() {
        let names = BrowserToolGroup::Canvas.tool_names();
        assert_eq!(
            names,
            vec![
                "browser_navigate",
                "browser_click",
                "browser_screenshot",
                "browser_snapshot",
                "browser_close",
            ],
            "canvas self-check minimum — see the design note's §4 table"
        );
    }

    #[test]
    fn specific_groups_partition_the_suite() {
        let groups = [
            BrowserToolGroup::Canvas,
            BrowserToolGroup::Tabs,
            BrowserToolGroup::Forms,
            BrowserToolGroup::Extract,
            BrowserToolGroup::Session,
        ];
        let mut seen: Vec<&str> = groups.iter().flat_map(|g| g.tool_names()).collect();
        let total = seen.len();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(
            seen.len(),
            total,
            "groups must not overlap (each tool in exactly one group)"
        );
        let mut all = BrowserToolGroup::All.tool_names();
        all.sort_unstable();
        assert_eq!(seen, all, "the five groups must cover the whole suite");
    }

    #[test]
    fn all_equals_everything() {
        let all = BrowserToolGroup::All.tool_names();
        assert_eq!(all.len(), 22);
        for name in &all {
            assert!(BrowserToolGroup::All.contains(name));
        }
        assert!(!BrowserToolGroup::All.contains("browser_nonexistent"));
    }

    #[test]
    fn groups_slice_composes_and_dedupes() {
        let union = browser_tool_names(&[BrowserToolGroup::Canvas, BrowserToolGroup::Tabs]);
        assert_eq!(union.len(), 9);
        assert!(union.contains(&"browser_navigate"));
        assert!(union.contains(&"browser_tab_list"));
        assert!(!union.contains(&"browser_evaluate"));

        // Overlap and All both collapse to one list each.
        let dup = browser_tool_names(&[BrowserToolGroup::Canvas, BrowserToolGroup::Canvas]);
        assert_eq!(dup, BrowserToolGroup::Canvas.tool_names());
        let all = browser_tool_names(&[BrowserToolGroup::Canvas, BrowserToolGroup::All]);
        assert_eq!(all, BrowserToolGroup::All.tool_names());

        // Empty selection yields nothing — not everything.
        assert!(browser_tool_names(&[]).is_empty());
    }

    #[test]
    fn subset_construction_filters_by_group() {
        let browser = crate::BrowserToolset::lazy(crate::LaunchOptions::default());
        let canvas = crate::browser_tools_subset(&browser, BrowserToolGroup::Canvas);
        let names: Vec<_> = canvas.iter().map(|t| t.name()).collect();
        assert_eq!(names, BrowserToolGroup::Canvas.tool_names());
        assert_eq!(canvas.len(), 5);

        let empty = crate::browser_tools_groups(&browser, &[]);
        assert!(empty.is_empty());

        let mixed = crate::browser_tools_groups(
            &browser,
            &[BrowserToolGroup::Extract, BrowserToolGroup::Session],
        );
        assert_eq!(mixed.len(), 8);
    }
}
