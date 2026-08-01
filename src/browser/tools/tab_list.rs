use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Tool, ToolContext, ToolControlFlow, ToolOutput};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;
use crate::browser::tools::utils::get_page_url;

pub struct BrowserTabListTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserTabListTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserTabListTool {
    fn name(&self) -> &'static str {
        "browser_tab_list"
    }

    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "browser_tab_list",
                "description": "List all open browser tabs with their titles, URLs, and indices.",
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
            match session.get_tabs() {
                Ok(tabs) => {
                    let mut output = String::new();
                    let mut tab_data = Vec::new();

                    for (i, tab) in tabs.iter().enumerate() {
                        let url = get_page_url(tab);
                        let title = tab
                            .get_title()
                            .unwrap_or_else(|_| "unknown".to_string());
                        let active = url == get_active_url(&session);
                        let marker = if active { " [active]" } else { "" };

                        output.push_str(&format!(
                            "[{}]{} {}\n    {}\n",
                            i, marker, title, url
                        ));
                        tab_data.push(json!({
                            "index": i,
                            "title": title,
                            "url": url,
                            "active": active
                        }));
                    }

                    Ok(ToolOutput {
                        summary: format!("{} tab(s):\n{}", tabs.len(), output),
                        raw: Some(json!({"tabs": tab_data, "count": tabs.len()})),
                        control_flow: ToolControlFlow::Continue,
                        truncation: None,
                    })
                }
                Err(e) => Ok(ToolOutput {
                    summary: format!("Failed to list tabs: {}", e),
                    raw: None,
                    control_flow: ToolControlFlow::Continue,
                    truncation: None,
                }),
            }
        })
        .await
        .map_err(|e| agent_base::AgentError::Internal(format!("browser_tab_list failed: {}", e)))?
    }
}

fn get_active_url(session: &BrowserSession) -> String {
    session
        .tab()
        .ok()
        .and_then(|tab| {
            tab.evaluate("window.location.href", false)
                .ok()
                .and_then(|r| r.value)
                .and_then(|v| v.as_str().map(String::from))
        })
        .unwrap_or_default()
}
