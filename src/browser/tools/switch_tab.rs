use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Content, Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;

pub struct BrowserSwitchTabTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserSwitchTabTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserSwitchTabTool {
    fn name(&self) -> &'static str {
        "browser_switch_tab"
    }

    fn description(&self) -> &'static str {
        "Switch to a specific tab by its index (from browser_tab_list)."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "index": {
                    "type": "integer",
                    "description": "Tab index from browser_tab_list"
                }
            },
            "required": ["index"]
        })
    }

    async fn call(&self, args: &Value, _ctx: &ToolContext) -> AgentResult<Vec<Content>> {
        let session = self.session.clone();
        let index = args["index"].as_u64().map(|i| i as usize).unwrap_or(0);

        tokio::task::spawn_blocking(move || {
            let session = session.lock().unwrap_or_else(|e| e.into_inner());
            let tabs = match session.get_tabs() {
                Ok(t) => t,
                Err(e) => {
                    return Ok(vec![Content::text(format!("Failed to get tabs: {}", e))]);
                }
            };

            if index >= tabs.len() {
                return Ok(vec![Content::text(format!(
                    "Tab index {} out of range ({} tabs).",
                    index,
                    tabs.len()
                ))]);
            }

            let tab = &tabs[index];
            match tab.activate() {
                Ok(_) => {
                    let title = tab.get_title().unwrap_or_default();
                    Ok(vec![Content::text(format!(
                        "Switched to tab [{}]: {}",
                        index, title
                    ))])
                }
                Err(e) => Ok(vec![Content::text(format!(
                    "Failed to switch to tab {}: {}",
                    index, e
                ))]),
            }
        })
        .await
        .map_err(|e| {
            agent_base::AgentError::Internal(format!("browser_switch_tab failed: {}", e))
        })?
    }
}
