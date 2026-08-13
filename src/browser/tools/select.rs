use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Content, Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;
use crate::browser::tools::utils::resolve_selector;

pub struct BrowserSelectTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserSelectTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserSelectTool {
    fn name(&self) -> &'static str {
        "browser_select"
    }

    fn description(&self) -> &'static str {
        "Select an option in a dropdown (select) element. Use the index from the page snapshot or a CSS selector."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "index": {
                    "type": "integer",
                    "description": "Element index from the page snapshot. Use either index or selector."
                },
                "selector": {
                    "type": "string",
                    "description": "CSS selector for the select element. Use either index or selector."
                },
                "value": {
                    "type": "string",
                    "description": "The option value to select"
                }
            },
            "required": ["value"]
        })
    }

    async fn call(&self, args: &Value, _ctx: &ToolContext) -> AgentResult<Vec<Content>> {
        let session = self.session.clone();
        let value = args["value"].as_str().unwrap_or("").to_string();
        let index = args["index"].as_u64().map(|i| i as usize);
        let selector = args["selector"].as_str().map(String::from);

        tokio::task::spawn_blocking(move || {
            let session = session.lock().unwrap_or_else(|e| e.into_inner());

            let css = match resolve_selector(&session, index, selector, "browser_select") {
                Ok(s) => s,
                Err(msg) => return Ok(vec![Content::text(msg)]),
            };

            match session.select_option(&css, &value) {
                Ok(_) => Ok(vec![Content::text(format!(
                    "Selected '{}' in: {}",
                    value, css
                ))]),
                Err(e) => Ok(vec![Content::text(format!(
                    "Select failed on '{}': {}",
                    css, e
                ))]),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_select failed: {}", e)))?
    }
}
