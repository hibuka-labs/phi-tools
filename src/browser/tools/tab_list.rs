use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Content, Tool, ToolContext};
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

    fn description(&self) -> &'static str {
        "List all open browser tabs with their titles, URLs, and indices."
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
            match session.get_tabs() {
                Ok(tabs) => {
                    let mut output = String::new();

                    for (i, tab) in tabs.iter().enumerate() {
                        let url = get_page_url(tab);
                        let title = tab.get_title().unwrap_or_else(|_| "unknown".to_string());
                        let active = url == get_active_url(&session);
                        let marker = if active { " [active]" } else { "" };

                        output.push_str(&format!("[{}]{} {}\n    {}\n", i, marker, title, url));
                    }

                    Ok(vec![Content::text(format!(
                        "{} tab(s):\n{}",
                        tabs.len(),
                        output
                    ))])
                }
                Err(e) => Ok(vec![Content::text(format!("Failed to list tabs: {}", e))]),
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
