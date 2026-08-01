use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Tool, ToolContext, ToolControlFlow, ToolOutput};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;
use crate::browser::tools::utils::{normalize_url, render_aria_tree, RenderMode};

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

    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "browser_navigate",
                "description": "Navigate to a URL in the browser. Returns a page snapshot with numbered interactive elements. Use this to open web pages.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to navigate to (e.g., 'https://example.com' or 'example.com')"
                        }
                    },
                    "required": ["url"]
                }
            }
        })
    }

    async fn call(&self, args: &Value, _ctx: &ToolContext) -> AgentResult<ToolOutput> {
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
                            Ok(ToolOutput {
                                summary: format!(
                                    "Navigated to {}\n\nPage snapshot:\n{}",
                                    url, snapshot
                                ),
                                raw: Some(json!({"url": url, "success": true})),
                                control_flow: ToolControlFlow::Continue,
                                truncation: None,
                            })
                        }
                        Err(e) => Ok(ToolOutput {
                            summary: format!(
                                "Navigated to {} but failed to extract page content: {}",
                                url, e
                            ),
                            raw: Some(json!({"url": url, "success": true, "warning": e})),
                            control_flow: ToolControlFlow::Continue,
                            truncation: None,
                        }),
                    }
                }
                Err(e) => Ok(ToolOutput {
                    summary: format!("Navigation failed: {}", e),
                    raw: Some(json!({"url": url, "success": false, "error": e})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_navigate failed: {}", e)))?
    }
}
