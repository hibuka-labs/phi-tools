use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Content, Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;

pub struct BrowserEvaluateTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserEvaluateTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserEvaluateTool {
    fn name(&self) -> &'static str {
        "browser_evaluate"
    }

    fn description(&self) -> &'static str {
        "Execute JavaScript code in the browser context and return the result. Use for extracting data or performing custom page interactions."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "script": {
                    "type": "string",
                    "description": "JavaScript code to execute. The return value will be serialized to JSON."
                }
            },
            "required": ["script"]
        })
    }

    async fn call(&self, args: &Value, _ctx: &ToolContext) -> AgentResult<Vec<Content>> {
        let session = self.session.clone();
        let script = args["script"].as_str().unwrap_or("").to_string();

        tokio::task::spawn_blocking(move || {
            let session = session.lock().unwrap_or_else(|e| e.into_inner());
            match session.evaluate(&script) {
                Ok(result) => Ok(vec![Content::text(format!(
                    "JavaScript result: {}",
                    result
                ))]),
                Err(e) => Ok(vec![Content::text(format!(
                    "JavaScript evaluation failed: {}",
                    e
                ))]),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_evaluate failed: {}", e)))?
    }
}
