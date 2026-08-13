use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Content, Tool, ToolContext};
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

    fn description(&self) -> &'static str {
        "Close the current active tab. If it's the last tab, the browser session will end."
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
            match session.close_active_tab() {
                Ok(_) => Ok(vec![Content::text("Tab closed.".to_string())]),
                Err(e) => Ok(vec![Content::text(format!("Failed to close tab: {}", e))]),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_close_tab failed: {}", e)))?
    }
}
