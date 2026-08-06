use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Tool, ToolContext, ToolControlFlow, ToolOutput};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;
use crate::browser::tools::utils::resolve_selector;

pub struct BrowserSelectTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserSelectTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserSelectTool {
    fn name(&self) -> &'static str {
        "browser_select"
    }

    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "browser_select",
                "description": "Select an option in a dropdown (select) element. Use the index from the page snapshot or a CSS selector.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "index": {
                            "type": "integer",
                            "description": "Element index from the page snapshot. Use either index or selector."
                        },
                        "selector": {
                            "type": "string",
                            "description": "CSS selector for the select element. Use either index or selector."
                        },
                        "value": {
                            "type": "string",
                            "description": "The option value to select"
                        }
                    },
                    "required": ["value"]
                }
            }
        })
    }

    async fn call(&self, args: &Value, _ctx: &ToolContext) -> AgentResult<ToolOutput> {
        let session = self.session.clone();
        let value = args["value"].as_str().unwrap_or("").to_string();
        let index = args["index"].as_u64().map(|i| i as usize);
        let selector = args["selector"].as_str().map(String::from);

        tokio::task::spawn_blocking(move || {
            let session = session.lock().unwrap_or_else(|e| e.into_inner());

            let css = match resolve_selector(&session, index, selector, "browser_select") {
                Ok(s) => s,
                Err(msg) => return Ok(msg),
            };

            match session.select_option(&css, &value) {
                Ok(_) => Ok(ToolOutput {
                    summary: format!("Selected '{}' in: {}", value, css),
                    raw: Some(json!({"selector": css, "value": value, "success": true})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
                Err(e) => Ok(ToolOutput {
                    summary: format!("Select failed on '{}': {}", css, e),
                    raw: Some(json!({"selector": css, "success": false, "error": e})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_select failed: {}", e)))?
    }
}
