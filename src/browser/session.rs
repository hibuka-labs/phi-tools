//! Browser session management.
//!
//! Wraps `headless_chrome::Browser` to provide CDP-based browser control:
//! lifecycle (launch/connect/close), tab management, navigation, DOM
//! extraction, element interaction, and screenshot capture.
//!
//! Ported from browser-use-rs, adapted for phi-tools: removed ToolRegistry.

use std::ffi::OsStr;
use std::sync::Arc;
use std::time::Duration;

use headless_chrome::{Browser, LaunchOptions as ChromeLaunchOptions, Tab};
use log;

use super::config::{ConnectionOptions, LaunchOptions};
use super::dom::DomTree;

/// Browser session wrapping a Chrome/Chromium instance via CDP.
pub struct BrowserSession {
    browser: Browser,
}

impl BrowserSession {
    // ── Lifecycle ──

    /// Launch a new browser instance.
    pub fn launch(options: LaunchOptions) -> Result<Self, String> {
        let mut launch_opts = ChromeLaunchOptions::default();

        // Anti-bot detection mitigation
        launch_opts
            .ignore_default_args
            .push(OsStr::new("--enable-automation"));
        launch_opts
            .args
            .push(OsStr::new("--disable-blink-features=AutomationControlled"));

        // Idle timeout: 1 hour (default is 30s)
        launch_opts.idle_browser_timeout = Duration::from_secs(60 * 60);

        launch_opts.headless = options.headless;
        launch_opts.window_size = Some((options.window_width, options.window_height));

        if let Some(path) = options.chrome_path {
            launch_opts.path = Some(path);
        }
        if let Some(dir) = options.user_data_dir {
            launch_opts.user_data_dir = Some(dir);
        }
        launch_opts.sandbox = options.sandbox;

        let browser = Browser::new(launch_opts)
            .map_err(|e| format!("Failed to launch browser: {}", e))?;

        // Create an initial tab
        browser
            .new_tab()
            .map_err(|e| format!("Failed to create initial tab: {}", e))?;

        Ok(Self {
            browser,
        })
    }

    /// Connect to an existing browser instance via WebSocket.
    pub fn connect(options: ConnectionOptions) -> Result<Self, String> {
        let browser = Browser::connect(options.ws_url)
            .map_err(|e| format!("Failed to connect to browser: {}", e))?;

        Ok(Self {
            browser,
        })
    }

    /// Close the browser.
    pub fn close(&mut self) -> Result<(), String> {
        // Close all tabs — the browser process exits when all tabs are closed.
        let tabs = self.get_tabs()?;
        for tab in tabs {
            let _ = tab.close(false);
        }
        Ok(())
    }

    // ── Tab management ──

    /// Get the active tab.
    pub fn tab(&self) -> Result<Arc<Tab>, String> {
        self.get_active_tab()
    }

    /// Create a new tab.
    pub fn new_tab(&mut self) -> Result<Arc<Tab>, String> {
        self.browser
            .new_tab()
            .map_err(|e| format!("Failed to create tab: {}", e))
    }

    /// Get all open tabs.
    pub fn get_tabs(&self) -> Result<Vec<Arc<Tab>>, String> {
        self.browser
            .get_tabs()
            .lock()
            .map_err(|e| format!("Failed to get tabs: {}", e))
            .map(|tabs| tabs.clone())
    }

    /// Get the currently active tab by checking visibility and focus state.
    pub fn get_active_tab(&self) -> Result<Arc<Tab>, String> {
        let tabs = self.get_tabs()?;

        // First pass: check visibility + focus (strongest signal)
        for tab in &tabs {
            let result = tab.evaluate(
                "document.visibilityState === 'visible' && document.hasFocus()",
                false,
            );
            if let Ok(obj) = result {
                if let Some(value) = obj.value {
                    if value.as_bool().unwrap_or(false) {
                        return Ok(tab.clone());
                    }
                }
            }
        }

        // Second pass: check just visibility
        for tab in &tabs {
            let result = tab.evaluate("document.visibilityState === 'visible'", false);
            if let Ok(obj) = result {
                if let Some(value) = obj.value {
                    if value.as_bool().unwrap_or(false) {
                        return Ok(tab.clone());
                    }
                }
            }
        }

        // Fallback: return first tab
        tabs.into_iter()
            .next()
            .ok_or_else(|| "No tabs available".to_string())
    }

    /// Close the active tab.
    pub fn close_active_tab(&mut self) -> Result<(), String> {
        self.tab()?
            .close(true)
            .map(|_| ())
            .map_err(|e| format!("Failed to close tab: {}", e))
    }

    // ── Navigation ──

    /// Navigate to a URL.
    pub fn navigate(&self, url: &str) -> Result<(), String> {
        self.tab()?
            .navigate_to(url)
            .map_err(|e| format!("Failed to navigate to {}: {}", url, e))?;
        Ok(())
    }

    /// Wait for navigation to complete.
    pub fn wait_for_navigation(&self) -> Result<(), String> {
        self.tab()?
            .wait_until_navigated()
            .map(|_| ())
            .map_err(|e| format!("Navigation timeout: {}", e))
    }

    /// Navigate back in browser history.
    pub fn go_back(&self) -> Result<(), String> {
        let js = "(function() { window.history.back(); return true; })()";
        self.tab()?
            .evaluate(js, false)
            .map_err(|e| format!("Failed to go back: {}", e))?;
        std::thread::sleep(Duration::from_millis(300));
        Ok(())
    }

    /// Navigate forward in browser history.
    pub fn go_forward(&self) -> Result<(), String> {
        let js = "(function() { window.history.forward(); return true; })()";
        self.tab()?
            .evaluate(js, false)
            .map_err(|e| format!("Failed to go forward: {}", e))?;
        std::thread::sleep(Duration::from_millis(300));
        Ok(())
    }

    // ── DOM extraction ──

    /// Extract the DOM tree from the active tab.
    pub fn extract_dom(&self) -> Result<DomTree, String> {
        DomTree::from_tab(&self.tab()?)
    }

    // ── Element interaction ──

    /// Find an element by CSS selector on the given tab.
    pub fn find_element<'a>(
        &self,
        tab: &'a Arc<Tab>,
        css_selector: &str,
    ) -> Result<headless_chrome::Element<'a>, String> {
        tab.find_element(css_selector)
            .map_err(|e| format!("Element '{}' not found: {}", css_selector, e))
    }

    /// Get the CSS selector for an element at the given DOM index.
    pub fn get_selector_for_index(&self, index: usize) -> Result<String, String> {
        let dom = self.extract_dom()?;
        dom.get_selector(index)
            .cloned()
            .ok_or_else(|| format!("No element with index {}", index))
    }

    /// Click an element by CSS selector.
    pub fn click_element(&self, selector: &str) -> Result<(), String> {
        let tab = self.tab()?;
        let element = self.find_element(&tab, selector)?;
        element
            .click()
            .map(|_| ())
            .map_err(|e| format!("Click failed: {}", e))
    }

    /// Type text into an input element.
    pub fn type_text(&self, selector: &str, text: &str) -> Result<(), String> {
        let tab = self.tab()?;
        let element = self.find_element(&tab, selector)?;

        // Click to focus, then type
        element
            .click()
            .map_err(|e| format!("Click to focus failed: {}", e))?;

        // Use JavaScript for typing (more reliable via CDP)
        let escaped = text.replace('\\', "\\\\").replace('\'', "\\'");
        let js = format!(
            r#"(() => {{
                const el = document.querySelector('{}');
                if (!el) return 'element not found';
                el.focus();
                el.value = '{}';
                el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return 'ok';
            }})()"#,
            selector.replace('\'', "\\'"),
            escaped
        );

        tab.evaluate(&js, false)
            .map_err(|e| format!("Type text failed: {}", e))?;
        Ok(())
    }

    /// Hover over an element.
    pub fn hover_element(&self, selector: &str) -> Result<(), String> {
        let js = format!(
            r#"(() => {{
                const el = document.querySelector('{}');
                if (!el) return 'element not found';
                el.dispatchEvent(new MouseEvent('mouseover', {{ bubbles: true }}));
                return 'ok';
            }})()"#,
            selector.replace('\'', "\\'")
        );
        self.tab()?
            .evaluate(&js, false)
            .map_err(|e| format!("Hover failed: {}", e))?;
        Ok(())
    }

    /// Select a dropdown option.
    pub fn select_option(&self, selector: &str, value: &str) -> Result<(), String> {
        let escaped_val = value.replace('\\', "\\\\").replace('\'', "\\'");
        let js = format!(
            r#"(() => {{
                const el = document.querySelector('{}');
                if (!el) return 'element not found';
                el.value = '{}';
                el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return 'ok';
            }})()"#,
            selector.replace('\'', "\\'"),
            escaped_val
        );
        self.tab()?
            .evaluate(&js, false)
            .map_err(|e| format!("Select failed: {}", e))?;
        Ok(())
    }

    /// Press a keyboard key.
    pub fn press_key(&self, key: &str) -> Result<(), String> {
        let js = format!(
            r#"(() => {{
                document.dispatchEvent(new KeyboardEvent('keydown', {{ key: '{}', bubbles: true }}));
                document.dispatchEvent(new KeyboardEvent('keyup', {{ key: '{}', bubbles: true }}));
                return 'ok';
            }})()"#,
            key, key
        );
        self.tab()?
            .evaluate(&js, false)
            .map_err(|e| format!("Press key failed: {}", e))?;
        Ok(())
    }

    /// Scroll the page.
    pub fn scroll(&self, direction: &str, amount: u32) -> Result<(), String> {
        let (x, y) = match direction {
            "down" => (0, amount as i32),
            "up" => (0, -(amount as i32)),
            "right" => (amount as i32, 0),
            "left" => (-(amount as i32), 0),
            _ => (0, amount as i32),
        };

        let js = format!(
            "window.scrollBy({{ left: {}, top: {}, behavior: 'smooth' }}); 'ok'",
            x, y
        );
        self.tab()?
            .evaluate(&js, false)
            .map_err(|e| format!("Scroll failed: {}", e))?;
        Ok(())
    }

    // ── Screenshot ──

    /// Capture a screenshot of the visible viewport as base64-encoded PNG string.
    pub fn screenshot(&self) -> Result<String, String> {
        use base64::Engine;
        use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;

        let tab = self.tab()?;
        let data = tab
            .capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true)
            .map_err(|e| format!("Screenshot failed: {}", e))?;
        Ok(base64::engine::general_purpose::STANDARD.encode(&data))
    }

    // ── JavaScript evaluation ──

    /// Execute JavaScript in the page context and return the result as a JSON value.
    pub fn evaluate(&self, script: &str) -> Result<serde_json::Value, String> {
        let tab = self.tab()?;
        let result = tab
            .evaluate(script, false)
            .map_err(|e| format!("JavaScript evaluation failed: {}", e))?;

        let value = result
            .value
            .unwrap_or(serde_json::Value::Null);
        Ok(value)
    }

    // ── Content extraction ──

    /// Get the page content as Markdown.
    pub fn get_markdown(&self) -> Result<String, String> {
        let tab = self.tab()?;
        let html = tab
            .get_content()
            .map_err(|e| format!("Failed to get page content: {}", e))?;
        Ok(html2md::parse_html(&html))
    }

    /// Get the page's full HTML.
    pub fn get_html(&self) -> Result<String, String> {
        let tab = self.tab()?;
        tab.get_content()
            .map_err(|e| format!("Failed to get HTML: {}", e))
    }

    /// Extract all links from the page.
    pub fn read_links(&self) -> Result<String, String> {
        let js = r#"(() => {
            const links = Array.from(document.querySelectorAll('a[href]'));
            return JSON.stringify(links.map((a, i) => ({
                index: i,
                text: (a.textContent || '').trim().substring(0, 200),
                href: a.href
            })));
        })()"#;

        let result = self.tab()?
            .evaluate(js, false)
            .map_err(|e| format!("Failed to read links: {}", e))?;

        let value = result.value.unwrap_or(serde_json::Value::Null);
        let links: Vec<serde_json::Value> =
            serde_json::from_value(value).unwrap_or_default();

        let mut output = String::new();
        for link in &links {
            let idx = link["index"].as_u64().unwrap_or(0);
            let text = link["text"].as_str().unwrap_or("");
            let href = link["href"].as_str().unwrap_or("");
            output.push_str(&format!("[{}] {} -> {}\n", idx, text, href));
        }

        Ok(output)
    }
}

impl Drop for BrowserSession {
    fn drop(&mut self) {
        log::debug!("BrowserSession dropped, closing browser");
        let _ = self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires Chrome installed
    fn test_launch_and_navigate() {
        let session = BrowserSession::launch(LaunchOptions::default())
            .expect("Failed to launch browser");
        assert!(session.get_tabs().is_ok());

        session.navigate("about:blank").expect("Navigation failed");
        session
            .wait_for_navigation()
            .expect("Wait for navigation failed");
    }

    #[test]
    #[ignore]
    fn test_extract_dom() {
        let session = BrowserSession::launch(LaunchOptions::default())
            .expect("Failed to launch browser");
        session
            .navigate("about:blank")
            .expect("Navigation failed");
        session
            .wait_for_navigation()
            .expect("Wait for navigation failed");

        let dom = session.extract_dom().expect("DOM extraction failed");
        assert!(dom.count_nodes() > 0);
    }

    #[test]
    #[ignore]
    fn test_screenshot() {
        let session = BrowserSession::launch(LaunchOptions::default())
            .expect("Failed to launch browser");
        session
            .navigate("about:blank")
            .expect("Navigation failed");

        let data = session.screenshot().expect("Screenshot failed");
        assert!(!data.is_empty());
    }
}
