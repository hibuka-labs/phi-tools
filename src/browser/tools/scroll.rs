use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Content, Tool, ToolContext};
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

    fn description(&self) -> &'static str {
        "Scroll the page. Use 'down' to scroll down, 'up' to scroll up."
    }

    fn schema(&self) -> Value {
        json!({
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
        })
    }

    async fn call(&self, args: &Value, _ctx: &ToolContext) -> AgentResult<Vec<Content>> {
        let session = self.session.clone();
        let direction = args["direction"].as_str().unwrap_or("down").to_string();
        let amount = args["amount"].as_u64().unwrap_or(500) as u32;

        tokio::task::spawn_blocking(move || {
            let session = session.lock().unwrap_or_else(|e| e.into_inner());
            match session.scroll(&direction, amount) {
                Ok(_) => Ok(vec![Content::text(format!(
                    "Scrolled {} by {}px.",
                    direction, amount
                ))]),
                Err(e) => Ok(vec![Content::text(format!("Scroll failed: {}", e))]),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_scroll failed: {}", e)))?
    }
}
