use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Content, Tool, ToolContext};
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

    fn description(&self) -> &'static str {
        "Get the current page content as Markdown. Useful for extracting readable content from articles or documentation pages."
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
            match session.get_markdown() {
                Ok(md) => Ok(vec![Content::text(md)]),
                Err(e) => Ok(vec![Content::text(format!(
                    "Failed to get markdown: {}",
                    e
                ))]),
            }
        })
        .await
        .map_err(|e| {
            agent_base::AgentError::Internal(format!("browser_get_markdown failed: {}", e))
        })?
    }
}
