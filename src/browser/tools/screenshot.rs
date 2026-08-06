use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Tool, ToolContext, ToolControlFlow, ToolOutput};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;

pub struct BrowserScreenshotTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserScreenshotTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserScreenshotTool {
    fn name(&self) -> &'static str {
        "browser_screenshot"
    }

    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "browser_screenshot",
                "description": "Capture a screenshot of the current page as a base64-encoded PNG image. Use this to visually understand the page layout.",
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
            let session = session.lock().unwrap_or_else(|e| e.into_inner());
            match session.screenshot() {
                Ok(data) => Ok(ToolOutput {
                    summary: format!("Screenshot captured ({} bytes base64 PNG).", data.len()),
                    raw: Some(json!({"screenshot": data, "format": "png", "encoding": "base64"})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
                Err(e) => Ok(ToolOutput {
                    summary: format!("Screenshot failed: {}", e),
                    raw: None,
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
            }
        })
        .await
        .map_err(|e| {
            agent_base::AgentError::Internal(format!("browser_screenshot failed: {}", e))
        })?
    }
}
