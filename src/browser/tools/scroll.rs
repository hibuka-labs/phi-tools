use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Tool, ToolContext, ToolControlFlow, ToolOutput};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;

pub struct BrowserScrollTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserScrollTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserScrollTool {
    fn name(&self) -> &'static str {
        "browser_scroll"
    }

    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "browser_scroll",
                "description": "Scroll the page. Use 'down' to scroll down, 'up' to scroll up.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "direction": {
                            "type": "string",
                            "description": "Scroll direction: 'down', 'up', 'left', or 'right' (default: 'down')"
                        },
                        "amount": {
                            "type": "integer",
                            "description": "Scroll amount in pixels (default: 500)"
                        }
                    }
                }
            }
        })
    }

    async fn call(&self, args: &Value, _ctx: &ToolContext) -> AgentResult<ToolOutput> {
        let session = self.session.clone();
        let direction = args["direction"]
            .as_str()
            .unwrap_or("down")
            .to_string();
        let amount = args["amount"].as_u64().unwrap_or(500) as u32;

        tokio::task::spawn_blocking(move || {
            let session = session.lock().unwrap();
            match session.scroll(&direction, amount) {
                Ok(_) => Ok(ToolOutput {
                    summary: format!("Scrolled {} by {}px.", direction, amount),
                    raw: Some(json!({"direction": direction, "amount": amount, "success": true})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
                Err(e) => Ok(ToolOutput {
                    summary: format!("Scroll failed: {}", e),
                    raw: Some(json!({"success": false, "error": e})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_scroll panic: {}", e)))?
    }
}
