use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Tool, ToolContext, ToolControlFlow, ToolOutput};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;

pub struct BrowserExtractTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserExtractTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserExtractTool {
    fn name(&self) -> &'static str {
        "browser_extract_content"
    }

    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "browser_extract_content",
                "description": "Extract the full HTML content or text of the current page. Use for detailed scraping or analysis.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "include_html": {
                            "type": "boolean",
                            "description": "If true, returns the full HTML source. If false (default), returns plain text only.",
                            "default": false
                        }
                    }
                }
            }
        })
    }

    async fn call(&self, args: &Value, _ctx: &ToolContext) -> AgentResult<ToolOutput> {
        let session = self.session.clone();
        let include_html = args["include_html"].as_bool().unwrap_or(false);

        tokio::task::spawn_blocking(move || {
            let session = session.lock().unwrap_or_else(|e| e.into_inner());
            if include_html {
                match session.get_html() {
                    Ok(html) => Ok(ToolOutput {
                        summary: html,
                        raw: Some(json!({"success": true})),
                        control_flow: ToolControlFlow::Continue,
                        truncation: None,
                    }),
                    Err(e) => Ok(ToolOutput {
                        summary: format!("Extract failed: {}", e),
                        raw: None,
                        control_flow: ToolControlFlow::Continue,
                        truncation: None,
                    }),
                }
            } else {
                // Plain text via JS
                let js = "document.body ? document.body.innerText : ''";
                match session.evaluate(js) {
                    Ok(result) => Ok(ToolOutput {
                        summary: result.as_str().unwrap_or("").to_string(),
                        raw: Some(json!({"success": true})),
                        control_flow: ToolControlFlow::Continue,
                        truncation: None,
                    }),
                    Err(e) => Ok(ToolOutput {
                        summary: format!("Extract failed: {}", e),
                        raw: None,
                        control_flow: ToolControlFlow::Continue,
                        truncation: None,
                    }),
                }
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_extract_content failed: {}", e)))?
    }
}
