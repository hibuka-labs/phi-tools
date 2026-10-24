use std::sync::{Arc, Mutex};

use agent_base::{AgentResult, Content, Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{Value, json};

use crate::browser::session::BrowserSession;

pub struct BrowserScreenshotTool {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserScreenshotTool {
    pub fn new(session: Arc<Mutex<BrowserSession>>) -> Self {
        Self { session }
    }
}

#[async_trait]
impl Tool for BrowserScreenshotTool {
    fn name(&self) -> &'static str {
        "browser_screenshot"
    }

    fn description(&self) -> &'static str {
        "Capture a screenshot of the current page as a base64-encoded PNG image. Use this to visually understand the page layout."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "save_to": {
                    "type": "string",
                    "description": "Also write the PNG bytes to this path. For handing the image to \
                        something that reads files rather than tool output — a canvas reference image, \
                        say. The image is still returned either way."
                }
            }
        })
    }

    async fn call(&self, args: &Value, _ctx: &ToolContext) -> AgentResult<Vec<Content>> {
        let session = self.session.clone();
        let save_to = args
            .get("save_to")
            .and_then(Value::as_str)
            .map(str::to_string);

        tokio::task::spawn_blocking(move || {
            let mut session = session.lock().unwrap_or_else(|e| e.into_inner());
            match session.screenshot() {
                Ok(data) => {
                    let mut out = Vec::new();
                    if let Some(path) = save_to {
                        out.push(Content::text(save_png(&data, &path)));
                    }
                    out.push(Content::image(data, "image/png"));
                    Ok(out)
                }
                Err(e) => Ok(vec![Content::text(format!("Screenshot failed: {}", e))]),
            }
        })
        .await
        .map_err(|e| {
            agent_base::AgentError::Internal(format!("browser_screenshot failed: {}", e))
        })?
    }
}

/// Decode the base64 PNG and write it out. Never fails the capture: the image
/// is already in hand, and a note is more useful than an error.
fn save_png(base64_png: &str, path: &str) -> String {
    use base64::Engine as _;

    let bytes = match base64::engine::general_purpose::STANDARD.decode(base64_png) {
        Ok(bytes) => bytes,
        Err(err) => return format!("save_to: could not decode the PNG: {err}"),
    };
    match std::fs::write(path, &bytes) {
        Ok(()) => format!("saved: {path}"),
        Err(err) => format!("save_to: could not write {path}: {err}"),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::save_png;

    /// A path under the system temp dir with room to clean up. No `tempfile`
    /// dev-dependency: the suite must build from the manifest as committed.
    fn scratch(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("phi-tools-save-png-{name}"))
    }

    #[test]
    fn save_png_writes_the_decoded_bytes() {
        let path = scratch("decoded.png");
        let note = save_png("aGVsbG8=", path.to_str().unwrap());
        assert!(note.contains("saved:"), "{note}");
        assert_eq!(std::fs::read(&path).unwrap(), b"hello");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn save_png_never_fails_the_capture() {
        // Bad base64 or a path that cannot be written: a note comes back, the
        // image the caller already holds is unaffected.
        let note = save_png("not base64!", "/tmp/whatever.png");
        assert!(note.starts_with("save_to: could not decode"), "{note}");

        let note = save_png("aGVsbG8=", "/no/such/dir/ref.png");
        assert!(note.starts_with("save_to: could not write"), "{note}");
    }
}
