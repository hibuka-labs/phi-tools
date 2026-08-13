use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Content, Tool, ToolContext};
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

    fn description(&self) -> &'static str {
        "Press a keyboard key. Common keys: Enter, Escape, Tab, ArrowDown, ArrowUp, ArrowLeft, ArrowRight, Backspace, Delete, PageDown, PageUp, Home, End."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "key": {
                    "type": "string",
                    "description": "The key to press (e.g., 'Enter', 'Escape', 'Tab', 'ArrowDown')"
                }
            },
            "required": ["key"]
        })
    }

    async fn call(&self, args: &Value, _ctx: &ToolContext) -> AgentResult<Vec<Content>> {
        let session = self.session.clone();
        let key = args["key"].as_str().unwrap_or("").to_string();

        tokio::task::spawn_blocking(move || {
            let session = session.lock().unwrap_or_else(|e| e.into_inner());
            match session.press_key(&key) {
                Ok(_) => Ok(vec![Content::text(format!("Pressed key: {}", key))]),
                Err(e) => Ok(vec![Content::text(format!("Press key failed: {}", e))]),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_press_key failed: {}", e)))?
    }
}
