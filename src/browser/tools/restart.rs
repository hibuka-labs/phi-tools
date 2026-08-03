use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Tool, ToolContext, ToolControlFlow, ToolOutput};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;

pub struct BrowserRestartTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserRestartTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserRestartTool {
    fn name(&self) -> &'static str {
        "browser_restart"
    }

    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "browser_restart",
                "description": "Restart the browser (relaunch Chrome with the original launch options). \
                 Use this when the browser is unresponsive, crashed, or when browser tool calls start \
                 failing with connection errors (e.g. \"closed connection\", \"No session with given id\"). \
                 This is the safe reset — do NOT kill Chrome via shell commands. \
                 After restart the browser starts fresh (previous tabs are gone); re-navigate as needed.",
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
            let mut session = session.lock().unwrap_or_else(|e| e.into_inner());
            match session.restart() {
                Ok(_) => Ok(ToolOutput {
                    summary: "Browser restarted successfully. It starts fresh — previous tabs are gone, re-navigate as needed.".to_string(),
                    raw: Some(json!({"success": true})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
                Err(e) => Ok(ToolOutput {
                    summary: format!("Failed to restart browser: {}", e),
                    raw: Some(json!({"success": false, "error": e})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_restart failed: {}", e)))?
    }
}
