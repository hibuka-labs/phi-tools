//! URL normalization and ARIA tree rendering utilities.

use crate::browser::dom::{AriaChild, AriaNode};

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
    if let Some(max) = max_depth {
        if depth > max {
            return;
        }
    }

    match mode {
        RenderMode::Ai => render_node_ai(node, buf, depth, max_depth),
    }
}

fn render_node_ai(node: &AriaNode, buf: &mut String, depth: usize, max_depth: Option<usize>) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser::dom::AriaNode;

    #[test]
    fn test_normalize_url() {
        assert_eq!(normalize_url("example.com"), "https://example.com");
        assert_eq!(
            normalize_url("https://example.com"),
            "https://example.com"
        );
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
}
