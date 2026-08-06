use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Tool, ToolContext, ToolControlFlow, ToolOutput};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;
use crate::browser::tools::utils::resolve_selector;

pub struct BrowserInputTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserInputTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserInputTool {
    fn name(&self) -> &'static str {
        "browser_input_fill"
    }

    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "browser_input_fill",
                "description": "Type text into an input element. Use the index from the page snapshot (preferred) or a CSS selector.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "index": {
                            "type": "integer",
                            "description": "Element index from the page snapshot (preferred). Use either index or selector."
                        },
                        "selector": {
                            "type": "string",
                            "description": "CSS selector for the element. Use either index or selector."
                        },
                        "text": {
                            "type": "string",
                            "description": "The text to type into the element"
                        }
                    },
                    "required": ["text"]
                }
            }
        })
    }

    async fn call(&self, args: &Value, _ctx: &ToolContext) -> AgentResult<ToolOutput> {
        let session = self.session.clone();
        let text = args["text"].as_str().unwrap_or("").to_string();
        let index = args["index"].as_u64().map(|i| i as usize);
        let selector = args["selector"].as_str().map(String::from);

        tokio::task::spawn_blocking(move || {
            let session = session.lock().unwrap_or_else(|e| e.into_inner());

            let css = match resolve_selector(&session, index, selector, "browser_input_fill") {
                Ok(s) => s,
                Err(msg) => return Ok(msg),
            };

            match session.type_text(&css, &text) {
                Ok(_) => Ok(ToolOutput {
                    summary: format!("Typed '{}' into: {}", text, css),
                    raw: Some(json!({"selector": css, "text": text, "success": true})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
                Err(e) => Ok(ToolOutput {
                    summary: format!("Input failed on '{}': {}", css, e),
                    raw: Some(json!({"selector": css, "success": false, "error": e})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
            }
        })
        .await
        .map_err(|e| {
            agent_base::AgentError::Internal(format!("browser_input_fill failed: {}", e))
        })?
    }
}
