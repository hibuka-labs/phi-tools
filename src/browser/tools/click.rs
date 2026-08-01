use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Tool, ToolContext, ToolControlFlow, ToolOutput};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;

pub struct BrowserClickTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserClickTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserClickTool {
    fn name(&self) -> &'static str {
        "browser_click"
    }

    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "browser_click",
                "description": "Click on an element. Use the index from the page snapshot (preferred) or a CSS selector.",
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
                        }
                    }
                }
            }
        })
    }

    async fn call(&self, args: &Value, _ctx: &ToolContext) -> AgentResult<ToolOutput> {
        let session = self.session.clone();
        let index = args["index"].as_u64().map(|i| i as usize);
        let selector = args["selector"].as_str().map(String::from);

        tokio::task::spawn_blocking(move || {
            let session = session.lock().unwrap();

            let css = match (index, selector) {
                (Some(_), Some(_)) => {
                    return Ok(ToolOutput {
                        summary: "Cannot specify both 'index' and 'selector'. Use one or the other."
                            .to_string(),
                        raw: None,
                        control_flow: ToolControlFlow::Continue,
                        truncation: None,
                    });
                }
                (None, None) => {
                    return Ok(ToolOutput {
                        summary: "Must specify either 'index' or 'selector'.".to_string(),
                        raw: None,
                        control_flow: ToolControlFlow::Continue,
                        truncation: None,
                    });
                }
                (Some(idx), None) => match session.get_selector_for_index(idx) {
                    Ok(s) => s,
                    Err(e) => {
                        return Ok(ToolOutput {
                            summary: format!("No element at index {}: {}", idx, e),
                            raw: None,
                            control_flow: ToolControlFlow::Continue,
                            truncation: None,
                        });
                    }
                },
                (None, Some(s)) => s,
            };

            match session.click_element(&css) {
                Ok(_) => Ok(ToolOutput {
                    summary: format!("Clicked element: {}", css),
                    raw: Some(json!({"selector": css, "success": true})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
                Err(e) => Ok(ToolOutput {
                    summary: format!("Click failed on '{}': {}", css, e),
                    raw: Some(json!({"selector": css, "success": false, "error": e})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_click panic: {}", e)))?
    }
}
