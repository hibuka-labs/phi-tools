use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Tool, ToolContext, ToolControlFlow, ToolOutput};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;

pub struct BrowserPressKeyTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserPressKeyTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserPressKeyTool {
    fn name(&self) -> &'static str {
        "browser_press_key"
    }

    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "browser_press_key",
                "description": "Press a keyboard key. Common keys: Enter, Escape, Tab, ArrowDown, ArrowUp, ArrowLeft, ArrowRight, Backspace, Delete, PageDown, PageUp, Home, End.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "key": {
                            "type": "string",
                            "description": "The key to press (e.g., 'Enter', 'Escape', 'Tab', 'ArrowDown')"
                        }
                    },
                    "required": ["key"]
                }
            }
        })
    }

    async fn call(&self, args: &Value, _ctx: &ToolContext) -> AgentResult<ToolOutput> {
        let session = self.session.clone();
        let key = args["key"].as_str().unwrap_or("").to_string();

        tokio::task::spawn_blocking(move || {
            let session = session.lock().unwrap_or_else(|e| e.into_inner());
            match session.press_key(&key) {
                Ok(_) => Ok(ToolOutput {
                    summary: format!("Pressed key: {}", key),
                    raw: Some(json!({"key": key, "success": true})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
                Err(e) => Ok(ToolOutput {
                    summary: format!("Press key failed: {}", e),
                    raw: Some(json!({"key": key, "success": false, "error": e})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_press_key failed: {}", e)))?
    }
}
