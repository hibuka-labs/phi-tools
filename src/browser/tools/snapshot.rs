use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Content, Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;
use crate::browser::tools::utils::{RenderMode, render_aria_tree};

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

    fn description(&self) -> &'static str {
        "Get an accessibility snapshot of the current page, with numbered interactive elements. Use this to see what's on the page and find element indices for interaction tools."
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
            match session.extract_dom() {
                Ok(dom) => {
                    let count = dom.count_interactive();
                    let snapshot = render_aria_tree(&dom.root, RenderMode::Ai, None);
                    Ok(vec![Content::text(format!(
                        "Page snapshot ({} interactive elements):\n{}",
                        count, snapshot
                    ))])
                }
                Err(e) => Ok(vec![Content::text(format!(
                    "Failed to capture snapshot: {}",
                    e
                ))]),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_snapshot failed: {}", e)))?
    }
}
