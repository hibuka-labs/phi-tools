use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Tool, ToolContext, ToolControlFlow, ToolOutput};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;

pub struct BrowserCloseTabTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserCloseTabTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserCloseTabTool {
    fn name(&self) -> &'static str {
        "browser_close_tab"
    }

    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "browser_close_tab",
                "description": "Close the current active tab. If it's the last tab, the browser session will end.",
                "parameters": {
                    "type": "object",
                    "properties": {}
                }
            }
        })
    }

    async fn call(&self, _args: &Value, _ctx: &ToolContext) -> AgentResult<ToolOutput> {
        let session = self.session.clone();

        tokio::task::spawn_blocking(move || {
            let mut session = session.lock().unwrap();
            match session.close_active_tab() {
                Ok(_) => Ok(ToolOutput {
                    summary: "Tab closed.".to_string(),
                    raw: Some(json!({"success": true})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
                Err(e) => Ok(ToolOutput {
                    summary: format!("Failed to close tab: {}", e),
                    raw: Some(json!({"success": false, "error": e})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_close_tab panic: {}", e)))?
    }
}
