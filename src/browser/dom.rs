//! DOM extraction and ARIA tree representation.
//!
//! Ported from browser-use-rs. Extracts page structure with indexed interactive
//! elements for AI-friendly targeting (based on Playwright's ARIA snapshot approach).

use std::collections::HashMap;

use headless_chrome::Tab;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ── ARIA Node types ──

/// Represents an ARIA node in the accessibility tree.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AriaNode {
    /// ARIA role (e.g., "button", "link", "textbox", "generic", "iframe", "fragment").
    pub role: String,
    /// Accessible name of the element.
    pub name: String,
    /// Index of the element in the interactive elements array.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,
    /// Child nodes (can be AriaNode or text strings).
    #[serde(default)]
    pub children: Vec<AriaChild>,
    /// ARIA properties specific to this element (e.g., url, placeholder).
    #[serde(default)]
    pub props: HashMap<String, String>,
    /// Box information (visibility, cursor).
    #[serde(default)]
    pub box_info: BoxInfo,
    /// Whether element is checked (for checkboxes, radios, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked: Option<AriaChecked>,
    /// Whether element is disabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    /// Whether element is expanded (for expandable elements).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expanded: Option<bool>,
    /// Heading/list level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u32>,
    /// Whether button is pressed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pressed: Option<AriaPressed>,
    /// Whether element is selected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
    /// Whether element is currently active/focused.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

/// Child of an AriaNode — either another AriaNode or a text string.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum AriaChild {
    Text(String),
    Node(Box<AriaNode>),
}

/// ARIA checked state (true, false, or mixed).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum AriaChecked {
    Bool(bool),
    Mixed(String),
}

/// ARIA pressed state (true, false, or mixed).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum AriaPressed {
    Bool(bool),
    Mixed(String),
}

/// Box/visibility information for an element.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoxInfo {
    /// Whether the element is visible (non-zero bounding box).
    #[serde(default)]
    pub visible: bool,
    /// CSS cursor value (e.g., "pointer", "default").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl Default for BoxInfo {
    fn default() -> Self {
        Self {
            visible: false,
            cursor: None,
        }
    }
}

// Legacy compatibility
pub type ElementNode = AriaNode;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl BoundingBox {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }

    pub fn area(&self) -> f64 {
        self.width * self.height
    }
}

// ── AriaNode methods ──

impl AriaNode {
    pub fn new(role: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            name: name.into(),
            index: None,
            children: Vec::new(),
            props: HashMap::new(),
            box_info: BoxInfo::default(),
            checked: None,
            disabled: None,
            expanded: None,
            level: None,
            pressed: None,
            selected: None,
            active: None,
        }
    }

    pub fn fragment() -> Self {
        Self::new("fragment", "")
    }

    pub fn with_index(mut self, index: usize) -> Self {
        self.index = Some(index);
        self
    }

    pub fn with_child(mut self, child: AriaChild) -> Self {
        self.children.push(child);
        self
    }

    pub fn with_children(mut self, children: Vec<AriaChild>) -> Self {
        self.children = children;
        self
    }

    pub fn with_prop(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.props.insert(key.into(), value.into());
        self
    }

    pub fn with_box(mut self, visible: bool, cursor: Option<String>) -> Self {
        self.box_info = BoxInfo { visible, cursor };
        self
    }

    pub fn with_checked(mut self, checked: bool) -> Self {
        self.checked = Some(AriaChecked::Bool(checked));
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }

    pub fn with_expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }

    pub fn with_level(mut self, level: u32) -> Self {
        self.level = Some(level);
        self
    }

    pub fn is_interactive(&self) -> bool {
        self.index.is_some() && self.box_info.visible
    }

    pub fn has_pointer_cursor(&self) -> bool {
        self.box_info
            .cursor
            .as_ref()
            .map_or(false, |c| c == "pointer")
    }

    pub fn is_container(&self) -> bool {
        self.role == "fragment" || self.role == "iframe"
    }

    pub fn get_text_content(&self) -> String {
        let mut result = String::new();
        self.collect_text(&mut result);
        result.trim().to_string()
    }

    fn collect_text(&self, buffer: &mut String) {
        for child in &self.children {
            match child {
                AriaChild::Text(text) => {
                    buffer.push_str(text);
                    buffer.push(' ');
                }
                AriaChild::Node(node) => {
                    node.collect_text(buffer);
                }
            }
        }
    }

    pub fn count_nodes(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(|c| match c {
                AriaChild::Text(_) => 0,
                AriaChild::Node(n) => n.count_nodes(),
            })
            .sum::<usize>()
    }

    pub fn find_by_index(&self, index: usize) -> Option<&AriaNode> {
        if self.index == Some(index) {
            return Some(self);
        }
        for child in &self.children {
            if let AriaChild::Node(node) = child {
                if let Some(found) = node.find_by_index(index) {
                    return Some(found);
                }
            }
        }
        None
    }

    pub fn find_by_index_mut(&mut self, index: usize) -> Option<&mut AriaNode> {
        if self.index == Some(index) {
            return Some(self);
        }
        for child in &mut self.children {
            if let AriaChild::Node(node) = child {
                if let Some(found) = node.find_by_index_mut(index) {
                    return Some(found);
                }
            }
        }
        None
    }

    pub fn count_interactive(&self) -> usize {
        let mut count = 0;
        self.count_interactive_recursive(&mut count);
        count
    }

    fn count_interactive_recursive(&self, count: &mut usize) {
        if self.index.is_some() {
            *count += 1;
        }
        for child in &self.children {
            if let AriaChild::Node(node) = child {
                node.count_interactive_recursive(count);
            }
        }
    }

    pub fn aria_equals(&self, other: &AriaNode) -> bool {
        if self.role != other.role || self.name != other.name {
            return false;
        }
        if self.checked != other.checked
            || self.disabled != other.disabled
            || self.expanded != other.expanded
            || self.level != other.level
            || self.pressed != other.pressed
            || self.selected != other.selected
        {
            return false;
        }
        if self.has_pointer_cursor() != other.has_pointer_cursor() {
            return false;
        }
        if self.props.len() != other.props.len() {
            return false;
        }
        for (k, v) in &self.props {
            if other.props.get(k) != Some(v) {
                return false;
            }
        }
        true
    }
}

// ── DomTree ──

/// Snapshot extraction response from JavaScript.
#[derive(Debug, serde::Deserialize)]
struct SnapshotResponse {
    root: AriaNode,
    selectors: Vec<String>,
    #[serde(rename = "iframeIndices")]
    iframe_indices: Vec<usize>,
}

/// Represents the ARIA snapshot of a web page.
#[derive(Debug, Clone)]
pub struct DomTree {
    /// Root AriaNode (usually a fragment).
    pub root: AriaNode,
    /// Array of CSS selectors indexed by element index.
    pub selectors: Vec<String>,
    /// List of iframe indices (for multi-frame snapshots).
    pub iframe_indices: Vec<usize>,
}

impl DomTree {
    pub fn new(root: AriaNode) -> Self {
        let mut tree = Self {
            root,
            selectors: Vec::new(),
            iframe_indices: Vec::new(),
        };
        tree.rebuild_maps();
        tree
    }

    /// Build DOM tree from a browser tab.
    pub fn from_tab(tab: &Arc<Tab>) -> Result<Self, String> {
        Self::from_tab_with_prefix(tab, "")
    }

    /// Build DOM tree from a browser tab with a ref prefix (for iframe handling).
    pub fn from_tab_with_prefix(tab: &Arc<Tab>, _ref_prefix: &str) -> Result<Self, String> {
        let js_code = include_str!("extract_dom.js");

        let result = tab
            .evaluate(js_code, false)
            .map_err(|e| format!("Failed to execute DOM extraction script: {}", e))?;

        let json_value = result
            .value
            .ok_or_else(|| "No value returned from DOM extraction".to_string())?;

        let json_str: String = serde_json::from_value(json_value)
            .map_err(|e| format!("Failed to get JSON string: {}", e))?;

        let response: SnapshotResponse = serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse snapshot JSON: {}", e))?;

        Ok(Self {
            root: response.root,
            selectors: response.selectors,
            iframe_indices: response.iframe_indices,
        })
    }

    fn rebuild_maps(&mut self) {
        self.iframe_indices.clear();
        let max_index = self.find_max_index(&self.root.clone());
        if let Some(max_idx) = max_index {
            if self.selectors.len() <= max_idx {
                self.selectors.resize(max_idx + 1, String::new());
            }
        }
        let root = self.root.clone();
        self.collect_iframe_indices(&root);
    }

    fn find_max_index(&self, node: &AriaNode) -> Option<usize> {
        let mut max = node.index;
        for child in &node.children {
            if let AriaChild::Node(child_node) = child {
                if let Some(child_max) = self.find_max_index(child_node) {
                    max = match max {
                        Some(current) => Some(current.max(child_max)),
                        None => Some(child_max),
                    };
                }
            }
        }
        max
    }

    fn collect_iframe_indices(&mut self, node: &AriaNode) {
        if let Some(index) = node.index {
            if node.role == "iframe" {
                self.iframe_indices.push(index);
            }
        }
        for child in &node.children {
            if let AriaChild::Node(child_node) = child {
                self.collect_iframe_indices(child_node);
            }
        }
    }

    /// Get CSS selector for a given index.
    pub fn get_selector(&self, index: usize) -> Option<&String> {
        self.selectors.get(index).filter(|s| !s.is_empty())
    }

    /// Get all interactive element indices.
    pub fn interactive_indices(&self) -> Vec<usize> {
        let mut indices = Vec::new();
        self.collect_indices(&self.root, &mut indices);
        indices.sort();
        indices
    }

    fn collect_indices(&self, node: &AriaNode, indices: &mut Vec<usize>) {
        if let Some(index) = node.index {
            indices.push(index);
        }
        for child in &node.children {
            if let AriaChild::Node(child_node) = child {
                self.collect_indices(child_node, indices);
            }
        }
    }

    pub fn count_nodes(&self) -> usize {
        self.root.count_nodes()
    }

    pub fn count_interactive(&self) -> usize {
        self.root.count_interactive()
    }

    pub fn find_node_by_index(&self, index: usize) -> Option<&AriaNode> {
        self.root.find_by_index(index)
    }

    pub fn find_node_by_index_mut(&mut self, index: usize) -> Option<&mut AriaNode> {
        self.root.find_by_index_mut(index)
    }

    pub fn get_iframe_indices(&self) -> &[usize] {
        &self.iframe_indices
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.root)
            .map_err(|e| format!("Failed to serialize DOM to JSON: {}", e))
    }

    pub fn inject_iframe_content(&mut self, iframe_index: usize, iframe_snapshot: DomTree) {
        if let Some(iframe_node) = self.find_node_by_index_mut(iframe_index) {
            iframe_node.children = iframe_snapshot.root.children;
            let offset = self.selectors.len();
            for selector in iframe_snapshot.selectors {
                if !selector.is_empty() {
                    self.selectors.push(selector);
                }
            }
            for idx in iframe_snapshot.iframe_indices {
                self.iframe_indices.push(idx + offset);
            }
        }
    }

    pub fn assemble_with_iframes<F>(mut self, mut get_iframe_snapshot: F) -> Self
    where
        F: FnMut(usize) -> Option<DomTree>,
    {
        let iframe_indices = self.iframe_indices.clone();
        for iframe_index in iframe_indices {
            if let Some(iframe_snapshot) = get_iframe_snapshot(iframe_index) {
                self.inject_iframe_content(iframe_index, iframe_snapshot);
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tree() -> AriaNode {
        let mut root = AriaNode::fragment();
        root.children.push(AriaChild::Node(Box::new(
            AriaNode::new("button", "Click me")
                .with_index(0)
                .with_box(true, Some("pointer".to_string())),
        )));
        root.children.push(AriaChild::Node(Box::new(
            AriaNode::new("link", "Go to page")
                .with_index(1)
                .with_box(true, None),
        )));
        root.children.push(AriaChild::Node(Box::new(
            AriaNode::new("paragraph", "")
                .with_child(AriaChild::Text("Some text".to_string())),
        )));
        root
    }

    #[test]
    fn test_is_interactive() {
        let interactive = AriaNode::new("button", "Click")
            .with_index(0)
            .with_box(true, None);
        assert!(interactive.is_interactive());

        let not_interactive = AriaNode::new("button", "Click").with_box(false, None);
        assert!(!not_interactive.is_interactive());
    }

    #[test]
    fn test_has_pointer_cursor() {
        let with_pointer = AriaNode::new("button", "")
            .with_box(true, Some("pointer".to_string()));
        assert!(with_pointer.has_pointer_cursor());
    }

    #[test]
    fn test_find_node_by_index() {
        let root = create_test_tree();
        let tree = DomTree::new(root);

        let button = tree.find_node_by_index(0);
        assert!(button.is_some());
        assert_eq!(button.unwrap().role, "button");
        assert_eq!(button.unwrap().name, "Click me");

        let not_found = tree.find_node_by_index(999);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_count_nodes() {
        let root = create_test_tree();
        let tree = DomTree::new(root);
        assert_eq!(tree.count_nodes(), 4);
    }

    #[test]
    fn test_interactive_indices() {
        let root = create_test_tree();
        let tree = DomTree::new(root);
        let indices = tree.interactive_indices();
        assert_eq!(indices.len(), 2);
        assert!(indices.contains(&0));
        assert!(indices.contains(&1));
    }

    #[test]
    fn test_inject_iframe_content() {
        let mut main_tree = AriaNode::fragment();
        main_tree.children.push(AriaChild::Node(Box::new(
            AriaNode::new("iframe", "").with_index(0),
        )));

        let mut iframe_tree = AriaNode::fragment();
        iframe_tree.children.push(AriaChild::Node(Box::new(
            AriaNode::new("button", "Inside iframe").with_index(0),
        )));

        let mut main = DomTree::new(main_tree);
        let iframe = DomTree::new(iframe_tree);

        main.inject_iframe_content(0, iframe);

        let iframe_node = main.find_node_by_index(0).unwrap();
        assert_eq!(iframe_node.children.len(), 1);
    }
}
