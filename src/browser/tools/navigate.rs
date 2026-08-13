use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Content, Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;
use crate::browser::tools::utils::{RenderMode, normalize_url, render_aria_tree};

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

    async fn call(&self, args: &Value, _ctx: &ToolContext) -> AgentResult<Vec<Content>> {
        let session = self.session.clone();
        let url = normalize_url(args["url"].as_str().unwrap_or(""));

        tokio::task::spawn_blocking(move || {
            let session = session.lock().unwrap_or_else(|e| e.into_inner());

            match session.navigate(&url) {
                Ok(_) => {
                    let _ = session.wait_for_navigation();
                    match session.extract_dom() {
                        Ok(dom) => {
                            let snapshot = render_aria_tree(&dom.root, RenderMode::Ai, None);
                            Ok(vec![Content::text(format!(
                                "Navigated to {}\n\nPage snapshot:\n{}",
                                url, snapshot
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
