use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Content, Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;
use crate::browser::tools::utils::resolve_selector;

pub struct BrowserClickTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserClickTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserClickTool {
    fn name(&self) -> &'static str {
        "browser_click"
    }

    fn description(&self) -> &'static str {
        "Click on an element. Use the index from the page snapshot (preferred) or a CSS selector."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "index": {
                    "type": "integer",
                    "description": "Element index from the page snapshot (preferred). Use either index or selector."
                },
                "selector": {
                    "type": "string",
                    "description": "CSS selector for the element. Use either index or selector."
                }
            }
        })
    }

    async fn call(&self, args: &Value, _ctx: &ToolContext) -> AgentResult<Vec<Content>> {
        let session = self.session.clone();
        let index = args["index"].as_u64().map(|i| i as usize);
        let selector = args["selector"].as_str().map(String::from);

        tokio::task::spawn_blocking(move || {
            let session = session.lock().unwrap_or_else(|e| e.into_inner());

            let css = match resolve_selector(&session, index, selector, "browser_click") {
                Ok(s) => s,
                Err(msg) => return Ok(vec![Content::text(msg)]),
            };

            match session.click_element(&css) {
                Ok(_) => Ok(vec![Content::text(format!("Clicked element: {}", css))]),
                Err(e) => Ok(vec![Content::text(format!(
                    "Click failed on '{}': {}",
                    css, e
                ))]),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_click failed: {}", e)))?
    }
}
