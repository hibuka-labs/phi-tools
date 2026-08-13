use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Content, Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;
use crate::browser::tools::utils::normalize_url;

pub struct BrowserNewTabTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserNewTabTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserNewTabTool {
    fn name(&self) -> &'static str {
        "browser_new_tab"
    }

    fn description(&self) -> &'static str {
        "Open a new browser tab and navigate to the specified URL."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "URL to open in the new tab"
                }
            },
            "required": ["url"]
        })
    }

    async fn call(&self, args: &Value, _ctx: &ToolContext) -> AgentResult<Vec<Content>> {
        let session = self.session.clone();
        let url = normalize_url(args["url"].as_str().unwrap_or(""));

        tokio::task::spawn_blocking(move || {
            let mut session = session.lock().unwrap_or_else(|e| e.into_inner());

            match session.new_tab() {
                Ok(_tab) => match session.navigate(&url) {
                    Ok(_) => {
                        let _ = session.wait_for_navigation();
                        Ok(vec![Content::text(format!(
                            "Opened new tab and navigated to {}.",
                            url
                        ))])
                    }
                    Err(e) => Ok(vec![Content::text(format!(
                        "New tab opened but navigation failed: {}",
                        e
                    ))]),
                },
                Err(e) => Ok(vec![Content::text(format!(
                    "Failed to open new tab: {}",
                    e
                ))]),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_new_tab failed: {}", e)))?
    }
}
