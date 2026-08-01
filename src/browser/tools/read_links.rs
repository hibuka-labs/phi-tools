use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Tool, ToolContext, ToolControlFlow, ToolOutput};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;

pub struct BrowserReadLinksTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserReadLinksTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserReadLinksTool {
    fn name(&self) -> &'static str {
        "browser_read_links"
    }

    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "browser_read_links",
                "description": "Extract all links from the current page with their text and URLs. Useful for discovering navigation options or scraping link lists.",
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
            let session = session.lock().unwrap();
            match session.read_links() {
                Ok(links) => Ok(ToolOutput {
                    summary: format!("Page links:\n{}", links),
                    raw: Some(json!({"success": true})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
                Err(e) => Ok(ToolOutput {
                    summary: format!("Failed to read links: {}", e),
                    raw: None,
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_read_links panic: {}", e)))?
    }
}
