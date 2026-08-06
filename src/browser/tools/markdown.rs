use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Tool, ToolContext, ToolControlFlow, ToolOutput};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;

pub struct BrowserGetMarkdownTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserGetMarkdownTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserGetMarkdownTool {
    fn name(&self) -> &'static str {
        "browser_get_markdown"
    }

    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "browser_get_markdown",
                "description": "Get the current page content as Markdown. Useful for extracting readable content from articles or documentation pages.",
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
            match session.get_markdown() {
                Ok(md) => Ok(ToolOutput {
                    summary: md,
                    raw: Some(json!({"success": true})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
                Err(e) => Ok(ToolOutput {
                    summary: format!("Failed to get markdown: {}", e),
                    raw: None,
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
            }
        })
        .await
        .map_err(|e| {
            agent_base::AgentError::Internal(format!("browser_get_markdown failed: {}", e))
        })?
    }
}
