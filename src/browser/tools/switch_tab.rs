use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Tool, ToolContext, ToolControlFlow, ToolOutput};
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

    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "browser_switch_tab",
                "description": "Switch to a specific tab by its index (from browser_tab_list).",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "index": {
                            "type": "integer",
                            "description": "Tab index from browser_tab_list"
                        }
                    },
                    "required": ["index"]
                }
            }
        })
    }

    async fn call(&self, args: &Value, _ctx: &ToolContext) -> AgentResult<ToolOutput> {
        let session = self.session.clone();
        let index = args["index"].as_u64().map(|i| i as usize).unwrap_or(0);

        tokio::task::spawn_blocking(move || {
            let session = session.lock().unwrap_or_else(|e| e.into_inner());
            let tabs = match session.get_tabs() {
                Ok(t) => t,
                Err(e) => {
                    return Ok(ToolOutput {
                        summary: format!("Failed to get tabs: {}", e),
                        raw: None,
                        control_flow: ToolControlFlow::Continue,
                        truncation: None,
                    });
                }
            };

            if index >= tabs.len() {
                return Ok(ToolOutput {
                    summary: format!(
                        "Tab index {} out of range ({} tabs).",
                        index,
                        tabs.len()
                    ),
                    raw: None,
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                });
            }

            let tab = &tabs[index];
            match tab.activate() {
                Ok(_) => {
                    let title = tab.get_title().unwrap_or_default();
                    Ok(ToolOutput {
                        summary: format!("Switched to tab [{}]: {}", index, title),
                        raw: Some(json!({"index": index, "title": title, "success": true})),
                        control_flow: ToolControlFlow::Continue,
                        truncation: None,
                    })
                }
                Err(e) => Ok(ToolOutput {
                    summary: format!("Failed to switch to tab {}: {}", index, e),
                    raw: Some(json!({"success": false, "error": e.to_string()})),
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_switch_tab failed: {}", e)))?
    }
}
