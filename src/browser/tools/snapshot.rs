use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Tool, ToolContext, ToolControlFlow, ToolOutput};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;
use crate::browser::tools::utils::{render_aria_tree, RenderMode};

pub struct BrowserSnapshotTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserSnapshotTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserSnapshotTool {
    fn name(&self) -> &'static str {
        "browser_snapshot"
    }

    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "browser_snapshot",
                "description": "Get an accessibility snapshot of the current page, with numbered interactive elements. Use this to see what's on the page and find element indices for interaction tools.",
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
            match session.extract_dom() {
                Ok(dom) => {
                    let count = dom.count_interactive();
                    let snapshot = render_aria_tree(&dom.root, RenderMode::Ai, None);
                    Ok(ToolOutput {
                        summary: format!(
                            "Page snapshot ({} interactive elements):\n{}",
                            count, snapshot
                        ),
                        raw: Some(json!({"interactive_count": count})),
                        control_flow: ToolControlFlow::Continue,
                        truncation: None,
                    })
                }
                Err(e) => Ok(ToolOutput {
                    summary: format!("Failed to capture snapshot: {}", e),
                    raw: None,
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_snapshot panic: {}", e)))?
    }
}
