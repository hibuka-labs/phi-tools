use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Content, Tool, ToolContext};
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

    fn description(&self) -> &'static str {
        "Navigate back in browser history. Returns a page snapshot after going back."
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
            match session.go_back() {
                Ok(_) => {
                    let snapshot = session.extract_dom().map_or_else(
                        |e| format!("(snapshot failed: {})", e),
                        |dom| render_aria_tree(&dom.root, RenderMode::Ai, None),
                    );
                    Ok(vec![Content::text(format!(
                        "Went back.\n\nPage snapshot:\n{}",
                        snapshot
                    ))])
                }
                Err(e) => Ok(vec![Content::text(format!("Go back failed: {}", e))]),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_go_back failed: {}", e)))?
    }
}
