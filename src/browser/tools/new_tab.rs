use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Tool, ToolContext, ToolControlFlow, ToolOutput};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;
use crate::browser::tools::utils::normalize_url;

pub struct BrowserNewTabTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserNewTabTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserNewTabTool {
    fn name(&self) -> &'static str {
        "browser_new_tab"
    }

    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "browser_new_tab",
                "description": "Open a new browser tab and navigate to the specified URL.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "URL to open in the new tab"
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
            let mut session = session.lock().unwrap_or_else(|e| e.into_inner());

            match session.new_tab() {
                Ok(_tab) => {
                    match session.navigate(&url) {
                        Ok(_) => {
                            let _ = session.wait_for_navigation();
                            Ok(ToolOutput {
                                summary: format!("Opened new tab and navigated to {}.", url),
                                raw: Some(json!({"url": url, "success": true})),
                                control_flow: ToolControlFlow::Continue,
                                truncation: None,
                            })
                        }
                        Err(e) => Ok(ToolOutput {
                            summary: format!("New tab opened but navigation failed: {}", e),
                            raw: Some(json!({"success": false, "error": e})),
                            control_flow: ToolControlFlow::Continue,
                            truncation: None,
                        }),
                    }
                }
                Err(e) => Ok(ToolOutput {
                    summary: format!("Failed to open new tab: {}", e),
                    raw: Some(json!({"success": false, "error": e})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_new_tab failed: {}", e)))?
    }
}
