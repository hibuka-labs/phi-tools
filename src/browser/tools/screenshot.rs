use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Content, Tool, ToolContext};
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

    fn description(&self) -> &'static str {
        "Capture a screenshot of the current page as a base64-encoded PNG image. Use this to visually understand the page layout."
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
            let session = session.lock().unwrap_or_else(|e| e.into_inner());
            match session.screenshot() {
                Ok(data) => Ok(vec![Content::image(data, "image/png")]),
                Err(e) => Ok(vec![Content::text(format!("Screenshot failed: {}", e))]),
            }
        })
        .await
        .map_err(|e| {
            agent_base::AgentError::Internal(format!("browser_screenshot failed: {}", e))
        })?
    }
}
