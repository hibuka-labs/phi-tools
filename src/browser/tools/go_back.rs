use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Tool, ToolContext, ToolControlFlow, ToolOutput};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;
use crate::browser::tools::utils::{RenderMode, render_aria_tree};

pub struct BrowserGoBackTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserGoBackTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserGoBackTool {
    fn name(&self) -> &'static str {
        "browser_go_back"
    }

    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "browser_go_back",
                "description": "Navigate back in browser history. Returns a page snapshot after going back.",
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
            let session = session.lock().unwrap_or_else(|e| e.into_inner());
            match session.go_back() {
                Ok(_) => {
                    let snapshot = session.extract_dom().map_or_else(
                        |e| format!("(snapshot failed: {})", e),
                        |dom| render_aria_tree(&dom.root, RenderMode::Ai, None),
                    );
                    Ok(ToolOutput {
                        summary: format!("Went back.\n\nPage snapshot:\n{}", snapshot),
                        raw: Some(json!({"success": true})),
                        control_flow: ToolControlFlow::Continue,
                        truncation: None,
                    })
                }
                Err(e) => Ok(ToolOutput {
                    summary: format!("Go back failed: {}", e),
                    raw: Some(json!({"success": false, "error": e})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_go_back failed: {}", e)))?
    }
}
