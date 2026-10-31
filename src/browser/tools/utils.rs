//! URL normalization, ARIA tree rendering, and shared tool utilities.

use crate::browser::dom::{AriaChild, AriaNode};
use crate::browser::session::BrowserSession;

/// Normalize a URL: add https:// prefix if missing.
pub fn normalize_url(url: &str) -> String {
    let url = url.trim();
    if url.is_empty() {
        return url.to_string();
    }
    if url.starts_with("http://") || url.starts_with("https://") {
        return url.to_string();
    }
    if url.starts_with("about:") || url.starts_with("data:") || url.starts_with("file:") {
        return url.to_string();
    }
    format!("https://{}", url)
}

// ── ARIA Tree Rendering ──

/// Rendering mode for the ARIA tree snapshot.
pub enum RenderMode {
    /// Optimized for AI consumption: compact, includes indices, hides non-interactive.
    Ai,
}

/// Render an ARIA tree as a compact text snapshot for AI consumption.
///
/// Format example:
/// ```text
/// - generic [active] [index=0]:
///   - heading "Page Title" [level=1] [index=1]:
///     text: Page Title
///   - button "Submit" [index=2]:
///   - textbox "Search" [index=3]:
/// ```
pub fn render_aria_tree(root: &AriaNode, mode: RenderMode, max_depth: Option<usize>) -> String {
    let mut buf = String::new();
    render_node(root, &mut buf, 0, &mode, max_depth);
    buf
}

fn render_node(
    node: &AriaNode,
    buf: &mut String,
    depth: usize,
    mode: &RenderMode,
    max_depth: Option<usize>,
) {
    if let Some(max) = max_depth
        && depth > max
    {
        return;
    }

    match mode {
        RenderMode::Ai => render_node_ai(node, buf, depth, max_depth),
    }
}

fn render_node_ai(node: &AriaNode, buf: &mut String, depth: usize, max_depth: Option<usize>) {
    // The cutoff has to live here too: this function recurses into itself, so
    // the check in `render_node` only ever fires for the root and `max_depth`
    // would otherwise truncate nothing.
    if let Some(max) = max_depth
        && depth > max
    {
        return;
    }
    let indent = "  ".repeat(depth);

    // Skip empty non-interactive non-container nodes in AI mode
    if !node.is_container()
        && !node.is_interactive()
        && node.name.is_empty()
        && node.children.is_empty()
        && node.role != "textbox"
    {
        // Still recurse into children
        for child in &node.children {
            if let AriaChild::Node(child_node) = child {
                render_node_ai(child_node, buf, depth, max_depth);
            }
        }
        return;
    }

    // Build node line
    buf.push_str(&indent);
    buf.push_str("- ");
    buf.push_str(&node.role);

    if let Some(idx) = node.index {
        buf.push_str(&format!(" [{}]", idx));
    }

    if !node.name.is_empty() {
        buf.push_str(&format!(" \"{}\"", node.name));
    }

    if node.active == Some(true) {
        buf.push_str(" [active]");
    }
    if node.disabled == Some(true) {
        buf.push_str(" [disabled]");
    }
    if node.checked.is_some() {
        let checked_str = match &node.checked {
            Some(crate::browser::dom::AriaChecked::Bool(true)) => "checked",
            Some(crate::browser::dom::AriaChecked::Bool(false)) => "unchecked",
            Some(crate::browser::dom::AriaChecked::Mixed(_)) => "mixed",
            _ => "",
        };
        if !checked_str.is_empty() {
            buf.push_str(&format!(" [{}]", checked_str));
        }
    }
    if let Some(level) = node.level {
        buf.push_str(&format!(" [level={}]", level));
    }
    if node.has_pointer_cursor() {
        buf.push_str(" [pointer]");
    }

    // Special props
    if let Some(url) = node.props.get("url") {
        buf.push_str(&format!(" -> {}", url));
    }
    if let Some(placeholder) = node.props.get("placeholder") {
        buf.push_str(&format!(" placeholder=\"{}\"", placeholder));
    }

    buf.push('\n');

    // Render text children inline
    for child in &node.children {
        match child {
            AriaChild::Text(text) => {
                let trimmed = text.trim();
                if !trimmed.is_empty() && trimmed != node.name {
                    buf.push_str(&format!("{}  text: {}\n", indent, trimmed));
                }
            }
            AriaChild::Node(child_node) => {
                render_node_ai(child_node, buf, depth + 1, max_depth);
            }
        }
    }
}

/// Continuation hint for messages that render a page view (snapshots,
/// markdown, links, HTML): the surgical re-read tool reaches the tail.
pub const CONTINUATION_PAGE: &str = "use browser_evaluate to read specific content";

/// Continuation hint for `browser_evaluate` results.
pub const CONTINUATION_SCRIPT: &str = "re-run with a narrower script";

/// Fit a page-derived message into the per-call output budget
/// (`ToolContext::max_output_chars`) so the engine's hard reject (agent-base
/// §6.5 — over-budget output fails the whole call) never fires for a tool
/// rendering unbounded page content. Same contract as the kernel tools'
/// `read_file`: whole lines kept, a continuation hint appended.
///
/// `None` (no budget configured) and text already within budget come back
/// unchanged. The budget counts **chars** — the engine measures
/// `content_text(..).chars().count()`, so CJK page text must not be cut at a
/// byte count. `continuation` names the way to see the missing tail.
pub fn fit_to_budget(text: &str, max_chars: Option<usize>, continuation: &str) -> String {
    let Some(max_chars) = max_chars else {
        return text.to_string();
    };
    let total = text.chars().count();
    if total <= max_chars {
        return text.to_string();
    }

    // The marker and its separating newline are part of the budget, so the
    // message the engine measures stays <= max_chars with the notice attached.
    let marker = format!("...(truncated, {total} chars total; {continuation})");
    let reserve = marker.chars().count() + 1;
    let budget = max_chars.saturating_sub(reserve);

    let mut out = String::new();
    let mut used = 0usize;
    for line in text.split_inclusive('\n') {
        let line_chars = line.chars().count();
        if used + line_chars > budget {
            break;
        }
        out.push_str(line);
        used += line_chars;
    }
    if used == 0 {
        // Not even the first line fits (one giant line — e.g. minified HTML —
        // or a tiny budget): hard-cut at a char boundary so the remainder is
        // still valid text.
        out.extend(text.chars().take(budget));
    }
    while out.ends_with('\n') {
        out.pop();
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(&marker);

    // Ultimate guard: a budget smaller than the marker itself must still not
    // exceed what the engine measures.
    if out.chars().count() > max_chars {
        out = out.chars().take(max_chars).collect();
    }
    out
}

/// Get the current page URL from a tab.
pub fn get_page_url(tab: &headless_chrome::Tab) -> String {
    let result = tab.evaluate("window.location.href", false);
    match result {
        Ok(obj) => obj
            .value
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "unknown".to_string()),
        Err(_) => "unknown".to_string(),
    }
}

/// Resolve an element's CSS selector from either a snapshot index or an explicit CSS selector.
/// Returns an error message if both are specified or neither.
pub fn resolve_selector(
    session: &mut BrowserSession,
    index: Option<usize>,
    selector: Option<String>,
    tool_name: &str,
) -> Result<String, String> {
    match (index, selector) {
        (Some(_), Some(_)) => Err(format!(
            "{}: cannot specify both 'index' and 'selector'. Use one or the other.",
            tool_name
        )),
        (None, None) => Err(format!(
            "{}: must specify either 'index' or 'selector'.",
            tool_name
        )),
        (Some(idx), None) => session
            .get_selector_for_index(idx)
            .map_err(|e| format!("{}: no element at index {}: {}", tool_name, idx, e)),
        (None, Some(s)) => Ok(s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser::dom::AriaNode;

    #[test]
    fn test_normalize_url() {
        assert_eq!(normalize_url("example.com"), "https://example.com");
        assert_eq!(normalize_url("https://example.com"), "https://example.com");
        assert_eq!(normalize_url("about:blank"), "about:blank");
    }

    #[test]
    fn test_render_aria_tree() {
        let mut root = AriaNode::fragment();
        root.children.push(AriaChild::Node(Box::new(
            AriaNode::new("button", "Click me")
                .with_index(0)
                .with_box(true, Some("pointer".to_string())),
        )));
        root.children.push(AriaChild::Node(Box::new(
            AriaNode::new("heading", "Title")
                .with_index(1)
                .with_level(1)
                .with_box(true, None),
        )));

        let snapshot = render_aria_tree(&root, RenderMode::Ai, None);
        assert!(snapshot.contains("button"));
        assert!(snapshot.contains("Click me"));
        assert!(snapshot.contains("[0]"));
        assert!(snapshot.contains("[pointer]"));
        assert!(snapshot.contains("[level=1]"));
    }

    /// `max_depth` must actually truncate. `render_node_ai` recurses into
    /// itself, so a cutoff that only lives in `render_node` would apply to the
    /// root and nothing below it.
    #[test]
    fn test_max_depth_truncates_nested_nodes() {
        fn button(name: &str, index: usize, child: AriaNode) -> AriaNode {
            let mut node = AriaNode::new("button", name).with_index(index);
            node.children.push(AriaChild::Node(Box::new(child)));
            node
        }

        let deepest = AriaNode::new("button", "Level3").with_index(2);
        let middle = button("Level2", 1, deepest);
        let outer = button("Level1", 0, middle);
        let mut root = AriaNode::fragment();
        root.children.push(AriaChild::Node(Box::new(outer)));

        let unbounded = render_aria_tree(&root, RenderMode::Ai, None);
        assert!(unbounded.contains("Level3"), "{unbounded}");

        // Depth 3 sits past the cutoff; depths 0-2 stay.
        let truncated = render_aria_tree(&root, RenderMode::Ai, Some(2));
        assert!(truncated.contains("Level1"), "{truncated}");
        assert!(truncated.contains("Level2"), "{truncated}");
        assert!(!truncated.contains("Level3"), "{truncated}");
    }

    // ── Output budget (agent-base §6.5: the engine rejects, tools self-bound) ──

    #[test]
    fn fit_to_budget_is_identity_without_a_budget_or_under_it() {
        let text = "short page\n- button [0] \"OK\"\n";
        assert_eq!(fit_to_budget(text, None, CONTINUATION_PAGE), text);
        assert_eq!(
            fit_to_budget(text, Some(text.chars().count()), CONTINUATION_PAGE),
            text,
            "exactly at budget is not truncation"
        );
    }

    #[test]
    fn fit_to_budget_counts_chars_not_bytes() {
        // The engine measures chars (CJK is 3 bytes/char). Five 40-char CJK
        // lines (615 bytes) at a 200-char budget: a byte-counting cut would
        // fit one line, a char-counting one fits two — and the result must
        // stay within 200 *chars* either way.
        let text: String = (0..5).map(|_| format!("{}\n", "汉".repeat(40))).collect();
        let out = fit_to_budget(&text, Some(200), CONTINUATION_PAGE);
        assert!(out.chars().count() <= 200, "{} chars", out.chars().count());
        assert!(out.contains("...(truncated"), "{out}");
        let whole_cjk_lines = out.lines().filter(|l| l.chars().count() == 40).count();
        assert_eq!(
            whole_cjk_lines, 2,
            "kept whole lines counted in chars: {out}"
        );
    }

    #[test]
    fn fit_to_budget_keeps_whole_lines_and_names_the_continuation() {
        // Fixed-width lines so a mid-line cut would be visible.
        let text: String = (0..20)
            .map(|i| format!("line-{i:02} {}\n", "x".repeat(12)))
            .collect();
        let out = fit_to_budget(&text, Some(120), CONTINUATION_PAGE);

        assert!(out.chars().count() <= 120, "{} chars", out.chars().count());
        assert!(out.contains("...(truncated"), "{out}");
        assert!(out.contains(CONTINUATION_PAGE), "{out}");
        assert!(out.contains("line-00"), "{out}");

        let kept: Vec<&str> = out.lines().filter(|l| l.starts_with("line-")).collect();
        assert!(!kept.is_empty(), "{out}");
        assert!(kept.len() < 20, "must actually truncate: {out}");
        assert!(
            kept.iter().all(|l| l.len() == "line-00 ".len() + 12),
            "kept lines are whole, never half-cut: {out}"
        );
        // The kept lines are a prefix of the page, in order.
        for (i, line) in kept.iter().enumerate() {
            assert!(line.starts_with(&format!("line-{i:02}")), "{out}");
        }
    }

    #[test]
    fn fit_to_budget_hard_cuts_a_single_giant_line() {
        // Minified HTML is one line — no line boundary exists to stop at.
        let text = "a".repeat(500);
        let out = fit_to_budget(&text, Some(100), CONTINUATION_SCRIPT);
        assert!(out.chars().count() <= 100, "{} chars", out.chars().count());
        assert!(out.contains("...(truncated"), "{out}");
    }

    #[test]
    fn fit_to_budget_never_exceeds_a_pathological_budget() {
        // A budget smaller than the marker itself still must not overshoot:
        // the engine rejects on measurement alone.
        let text = "b".repeat(500);
        for max in 0..40 {
            let out = fit_to_budget(&text, Some(max), CONTINUATION_PAGE);
            assert!(out.chars().count() <= max, "max={max}: {out}");
        }
    }
}
