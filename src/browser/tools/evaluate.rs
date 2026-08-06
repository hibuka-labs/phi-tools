use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Tool, ToolContext, ToolControlFlow, ToolOutput};
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

    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "browser_evaluate",
                "description": "Execute JavaScript code in the browser context and return the result. Use for extracting data or performing custom page interactions.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "script": {
                            "type": "string",
                            "description": "JavaScript code to execute. The return value will be serialized to JSON."
                        }
                    },
                    "required": ["script"]
                }
            }
        })
    }

    async fn call(&self, args: &Value, _ctx: &ToolContext) -> AgentResult<ToolOutput> {
        let session = self.session.clone();
        let script = args["script"].as_str().unwrap_or("").to_string();

        tokio::task::spawn_blocking(move || {
            let session = session.lock().unwrap_or_else(|e| e.into_inner());
            match session.evaluate(&script) {
                Ok(result) => Ok(ToolOutput {
                    summary: format!("JavaScript result: {}", result),
                    raw: Some(json!({"result": result, "success": true})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
                Err(e) => Ok(ToolOutput {
                    summary: format!("JavaScript evaluation failed: {}", e),
                    raw: Some(json!({"success": false, "error": e})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_evaluate failed: {}", e)))?
    }
}
