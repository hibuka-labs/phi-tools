use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Content, Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;
use crate::browser::tools::utils::{
    CONTINUATION_PAGE, RenderMode, fit_to_budget, normalize_url, render_aria_tree,
};

pub struct BrowserNavigateTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserNavigateTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserNavigateTool {
    fn name(&self) -> &'static str {
        "browser_navigate"
    }

    fn description(&self) -> &'static str {
        "Navigate to a URL in the browser. Returns a page snapshot with numbered interactive elements. Use this to open web pages."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to navigate to (e.g., 'https://example.com' or 'example.com')"
                }
            },
            "required": ["url"]
        })
    }

    async fn call(&self, args: &Value, ctx: &ToolContext) -> AgentResult<Vec<Content>> {
        let session = self.session.clone();
        let url = normalize_url(args["url"].as_str().unwrap_or(""));
        let max_chars = ctx.max_output_chars;

        tokio::task::spawn_blocking(move || {
            let mut session = session.lock().unwrap_or_else(|e| e.into_inner());

            match session.navigate(&url) {
                Ok(_) => {
                    let _ = session.wait_for_navigation();
                    match session.extract_dom() {
                        Ok(dom) => {
                            let snapshot = render_aria_tree(&dom.root, RenderMode::Ai, None);
                            Ok(vec![Content::text(fit_to_budget(
                                &format!("Navigated to {}\n\nPage snapshot:\n{}", url, snapshot),
                                max_chars,
                                CONTINUATION_PAGE,
                            ))])
                        }
                        Err(e) => Ok(vec![Content::text(format!(
                            "Navigated to {} but failed to extract page content: {}",
                            url, e
                        ))]),
                    }
                }
                Err(e) => Ok(vec![Content::text(format!("Navigation failed: {}", e))]),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_navigate failed: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser::BrowserToolset;
    use crate::browser::tools::BrowserSnapshotTool;

    fn text_of(result: &[Content]) -> String {
        assert_eq!(result.len(), 1);
        match &result[0] {
            Content::Text { text } => text.clone(),
            other => panic!("expected text content, got {other:?}"),
        }
    }

    /// A page whose aria tree is far past the output budget: 800 rows of CJK
    /// text and links — the shape of the jobui.com company page whose snapshot
    /// made the engine reject `browser_navigate` wholesale (session
    /// 20261029_e2623969).
    fn big_page() -> String {
        let mut rows = String::new();
        for i in 0..800 {
            rows.push_str(&format!(
                "<div><a href=\"#{i}\">链接{i} 标题文字测试</a><span>补充中文文本第{i}行内容</span></div>\n"
            ));
        }
        format!("<!doctype html><html><head><title>大页面</title></head><body>{rows}</body></html>")
    }

    /// Field → effect sweep: no budget means the full tree comes back (what
    /// the engine used to reject at 16K chars), a budget means the tool
    /// truncates itself — so the engine's hard reject (agent-base §6.5) can
    /// never fire on a big page.
    #[tokio::test]
    async fn oversized_snapshots_self_bound_to_the_output_budget() {
        let dir = tempfile::tempdir().expect("tempdir");
        let page = dir.path().join("big.html");
        std::fs::write(&page, big_page()).expect("fixture writes");
        let url = format!("file://{}", page.display());

        let toolset = BrowserToolset::launch(Default::default()).expect("chrome launches");
        let navigate = BrowserNavigateTool::new(toolset.session());
        let snapshot = BrowserSnapshotTool::new(toolset.session());

        // No budget: the whole tree. This leg is also the fixture-size guard —
        // a fixture that fit would pass the bounded legs vacuously.
        let unbounded_ctx = ToolContext::for_test();
        let unbounded = text_of(
            &navigate
                .call(&json!({ "url": &url }), &unbounded_ctx)
                .await
                .unwrap(),
        );
        assert!(
            unbounded.chars().count() > 16_000,
            "fixture must overflow the budget on its own: {} chars",
            unbounded.chars().count()
        );

        // Budget set: the tool truncates instead of handing the engine
        // something it will reject.
        let mut ctx = ToolContext::for_test();
        ctx.max_output_chars = Some(16_000);
        let bounded = text_of(&navigate.call(&json!({ "url": &url }), &ctx).await.unwrap());
        assert!(
            bounded.chars().count() <= 16_000,
            "{} chars",
            bounded.chars().count()
        );
        assert!(bounded.contains("...(truncated"), "{bounded}");
        assert!(
            bounded.contains("use browser_evaluate to read specific content"),
            "{bounded}"
        );

        // The recovery path must survive the same page: `browser_snapshot` on
        // an oversized page rejected wholesale too, so the model could
        // navigate but never see the page.
        let snap = text_of(&snapshot.call(&json!({}), &ctx).await.unwrap());
        assert!(
            snap.chars().count() <= 16_000,
            "{} chars",
            snap.chars().count()
        );
        assert!(snap.contains("...(truncated"), "{snap}");
    }
}
