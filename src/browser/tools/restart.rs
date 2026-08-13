use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Content, Tool, ToolContext};
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

    fn description(&self) -> &'static str {
        "Restart the browser (relaunch Chrome with the original launch options). \
         Use this when the browser is unresponsive, crashed, or when browser tool calls start \
         failing with connection errors (e.g. \"closed connection\", \"No session with given id\"). \
         This is the safe reset — do NOT kill Chrome via shell commands. \
         After restart the browser starts fresh (previous tabs are gone); re-navigate as needed."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn call(&self, _args: &Value, _ctx: &ToolContext) -> AgentResult<Vec<Content>> {
        let session = self.session.clone();

        tokio::task::spawn_blocking(move || {
            let mut session = session.lock().unwrap_or_else(|e| e.into_inner());
            match session.restart() {
                Ok(_) => Ok(vec![Content::text("Browser restarted successfully. It starts fresh — previous tabs are gone, re-navigate as needed.".to_string())]),
                Err(e) => Ok(vec![Content::text(format!("Failed to restart browser: {}", e))]),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_restart failed: {}", e)))?
    }
}
