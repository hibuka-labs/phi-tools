use std::sync::{Arc, Mutex};
use std::time::Duration;

use agent_base::{AgentResult, Content, Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;

pub struct BrowserWaitTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserWaitTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserWaitTool {
    fn name(&self) -> &'static str {
        "browser_wait"
    }

    fn description(&self) -> &'static str {
        "Wait for a specified duration (in milliseconds). Useful for waiting for page animations or dynamic content to load."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "timeout_ms": {
                    "type": "integer",
                    "description": "Time to wait in milliseconds (default: 2000)"
                }
            }
        })
    }

    async fn call(&self, args: &Value, _ctx: &ToolContext) -> AgentResult<Vec<Content>> {
        let timeout = args["timeout_ms"].as_u64().unwrap_or(2000);
        let session = self.session.clone();

        tokio::task::spawn_blocking(move || {
            let _session = session.lock().unwrap_or_else(|e| e.into_inner());
            std::thread::sleep(Duration::from_millis(timeout));
            Ok(vec![Content::text(format!("Waited {}ms.", timeout))])
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_wait failed: {}", e)))?
    }
}
