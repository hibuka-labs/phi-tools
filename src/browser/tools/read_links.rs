use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Content, Tool, ToolContext};
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

    fn description(&self) -> &'static str {
        "Extract all links from the current page with their text and URLs. Useful for discovering navigation options or scraping link lists."
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
            match session.read_links() {
                Ok(links) => Ok(vec![Content::text(format!("Page links:\n{}", links))]),
                Err(e) => Ok(vec![Content::text(format!("Failed to read links: {}", e))]),
            }
        })
        .await
        .map_err(|e| {
            agent_base::AgentError::Internal(format!("browser_read_links failed: {}", e))
        })?
    }
}
