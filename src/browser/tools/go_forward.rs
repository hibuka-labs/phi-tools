use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Tool, ToolContext, ToolControlFlow, ToolOutput};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;
use crate::browser::tools::utils::{render_aria_tree, RenderMode};

pub struct BrowserGoForwardTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserGoForwardTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserGoForwardTool {
    fn name(&self) -> &'static str {
        "browser_go_forward"
    }

    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "browser_go_forward",
                "description": "Navigate forward in browser history. Returns a page snapshot.",
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
            match session.go_forward() {
                Ok(_) => {
                    let snapshot = session.extract_dom().map_or_else(
                        |e| format!("(snapshot failed: {})", e),
                        |dom| render_aria_tree(&dom.root, RenderMode::Ai, None),
                    );
                    Ok(ToolOutput {
                        summary: format!("Went forward.\n\nPage snapshot:\n{}", snapshot),
                        raw: Some(json!({"success": true})),
                        control_flow: ToolControlFlow::Continue,
                        truncation: None,
                    })
                }
                Err(e) => Ok(ToolOutput {
                    summary: format!("Go forward failed: {}", e),
                    raw: Some(json!({"success": false, "error": e})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_go_forward panic: {}", e)))?
    }
}
