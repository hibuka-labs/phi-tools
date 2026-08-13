use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Content, Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;

pub struct BrowserExtractTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserExtractTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserExtractTool {
    fn name(&self) -> &'static str {
        "browser_extract_content"
    }

    fn description(&self) -> &'static str {
        "Extract the full HTML content or text of the current page. Use for detailed scraping or analysis."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "include_html": {
                    "type": "boolean",
                    "description": "If true, returns the full HTML source. If false (default), returns plain text only.",
                    "default": false
                }
            }
        })
    }

    async fn call(&self, args: &Value, _ctx: &ToolContext) -> AgentResult<Vec<Content>> {
        let session = self.session.clone();
        let include_html = args["include_html"].as_bool().unwrap_or(false);

        tokio::task::spawn_blocking(move || {
            let session = session.lock().unwrap_or_else(|e| e.into_inner());
            if include_html {
                match session.get_html() {
                    Ok(html) => Ok(vec![Content::text(html)]),
                    Err(e) => Ok(vec![Content::text(format!("Extract failed: {}", e))]),
                }
            } else {
                // Plain text via JS
                let js = "document.body ? document.body.innerText : ''";
                match session.evaluate(js) {
                    Ok(result) => Ok(vec![Content::text(
                        result.as_str().unwrap_or("").to_string(),
                    )]),
                    Err(e) => Ok(vec![Content::text(format!("Extract failed: {}", e))]),
                }
            }
        })
        .await
        .map_err(|e| {
            agent_base::AgentError::Internal(format!("browser_extract_content failed: {}", e))
        })?
    }
}
