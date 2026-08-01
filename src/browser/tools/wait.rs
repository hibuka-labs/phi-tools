use std::sync::{Arc, Mutex};
use std::time::Duration;

use agent_base::{AgentResult, Tool, ToolContext, ToolControlFlow, ToolOutput};
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

    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "browser_wait",
                "description": "Wait for a specified duration (in milliseconds). Useful for waiting for page animations or dynamic content to load.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "timeout_ms": {
                            "type": "integer",
                            "description": "Time to wait in milliseconds (default: 2000)"
                        }
                    }
                }
            }
        })
    }

    async fn call(&self, args: &Value, _ctx: &ToolContext) -> AgentResult<ToolOutput> {
        let timeout = args["timeout_ms"].as_u64().unwrap_or(2000);
        let session = self.session.clone();

        tokio::task::spawn_blocking(move || {
            let _session = session.lock().unwrap();
            std::thread::sleep(Duration::from_millis(timeout));
            Ok(ToolOutput {
                summary: format!("Waited {}ms.", timeout),
                raw: Some(json!({"timeout_ms": timeout})),
                control_flow: ToolControlFlow::Continue,
                truncation: None,
            })
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_wait panic: {}", e)))?
    }
}
