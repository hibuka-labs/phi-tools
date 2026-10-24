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

/// How this browser was created, so `restart()` can recreate it from scratch.
#[derive(Clone)]
enum SessionConfig {
    Launch(LaunchOptions),
    Connect(ConnectionOptions),
}

/// Whether a Chrome is attached right now.
///
/// Three states on purpose. `Idle` is a session that has not paid for Chrome
/// yet and spawns one on first use — a process that merely *has* browser tools
/// must not drag a Chrome into every run. `Down` is the opposite: torn down on
/// purpose, so only `restart` brings it back and the error can say so instead
/// of silently respawning what the agent just closed.
enum Live {
    Idle,
    Running(Browser),
    Down,
}

/// Browser session wrapping a Chrome/Chromium instance via CDP.
pub struct BrowserSession {
    live: Live,
    config: SessionConfig,
}

impl BrowserSession {
    // ── Lifecycle ──

    /// Launch a new browser instance.
    pub fn launch(options: LaunchOptions) -> Result<Self, String> {
        let browser = Self::spawn(&options)?;
        Ok(Self {
            live: Live::Running(browser),
            config: SessionConfig::Launch(options),
        })
    }

    /// Prepare a session that spawns Chrome on first use, not now.
    ///
    /// Every tool call goes through [`Self::browser`], which starts the
    /// instance here; until then the session costs no process.
    pub fn lazy(options: LaunchOptions) -> Self {
        Self {
            live: Live::Idle,
            config: SessionConfig::Launch(options),
        }
    }

    /// Connect to an existing browser instance via WebSocket.
    pub fn connect(options: ConnectionOptions) -> Result<Self, String> {
        let browser = Browser::connect(options.ws_url.clone())
            .map_err(|e| format!("Failed to connect to browser: {}", e))?;
        Ok(Self {
            live: Live::Running(browser),
            config: SessionConfig::Connect(options),
        })
    }

    /// Launch a fresh Chrome instance from stored options and create an initial tab.
    fn spawn(options: &LaunchOptions) -> Result<Browser, String> {
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

        if let Some(path) = options.chrome_path.clone() {
            launch_opts.path = Some(path);
        }
        if let Some(dir) = options.user_data_dir.clone() {
            launch_opts.user_data_dir = Some(dir);
        }
        launch_opts.sandbox = options.sandbox;

        let browser =
            Browser::new(launch_opts).map_err(|e| format!("Failed to launch browser: {}", e))?;

        // Create an initial tab
        browser
            .new_tab()
            .map_err(|e| format!("Failed to create initial tab: {}", e))?;

        Ok(browser)
    }

    /// Restart the browser: tear down the current instance (if any) and relaunch
    /// with the same options that created it. This is the safe way for the agent to
    /// recover when Chrome is unresponsive, crashed, or was killed externally — it
    /// replaces the internal `Browser`, so all shared tools see the new instance.
    pub fn restart(&mut self) -> Result<(), String> {
        let _ = self.close();

        self.live = Live::Running(self.spawn_attached()?);
        Ok(())
    }

    /// Spawn or reconnect from whatever config this session was built with.
    fn spawn_attached(&self) -> Result<Browser, String> {
        match &self.config {
            SessionConfig::Launch(opts) => Self::spawn(opts),
            SessionConfig::Connect(opts) => Browser::connect(opts.ws_url.clone())
                .map_err(|e| format!("Failed to connect to browser: {}", e)),
        }
    }

    /// Access the live browser, starting one if this session has never had
    /// one. Errors only when it was closed or killed on purpose.
    fn browser(&mut self) -> Result<&Browser, String> {
        if matches!(self.live, Live::Idle) {
            self.live = Live::Running(self.spawn_attached()?);
        }
        match &self.live {
            Live::Running(browser) => Ok(browser),
            Live::Idle | Live::Down => {
                Err("Browser is not running (it was closed or killed). Use browser_restart to relaunch."
                    .to_string())
            }
        }
    }

    /// Close the browser. Tolerates a dead/unresponsive connection.
    pub fn close(&mut self) -> Result<(), String> {
        if let Live::Running(browser) = &self.live {
            // Close all tabs — the browser process exits when all tabs are closed.
            if let Ok(tabs) = browser.get_tabs().lock().map(|t| t.clone()) {
                for tab in tabs {
                    let _ = tab.close(false);
                }
            }
        }
        // Deliberate teardown: `browser` must not respawn behind the agent's
        // back, so this lands in `Down` rather than back in `Idle`.
        self.live = Live::Down;
        Ok(())
    }

    // ── Tab management ──

    /// Get the active tab.
    pub fn tab(&mut self) -> Result<Arc<Tab>, String> {
        self.get_active_tab()
    }

    /// Create a new tab.
    pub fn new_tab(&mut self) -> Result<Arc<Tab>, String> {
        self.browser()?
            .new_tab()
            .map_err(|e| format!("Failed to create tab: {}", e))
    }

    /// Get all open tabs.
    pub fn get_tabs(&mut self) -> Result<Vec<Arc<Tab>>, String> {
        self.browser()?
            .get_tabs()
            .lock()
            .map_err(|e| format!("Failed to get tabs: {}", e))
            .map(|tabs| tabs.clone())
    }

    /// Get the currently active tab by checking visibility and focus state.
    pub fn get_active_tab(&mut self) -> Result<Arc<Tab>, String> {
        let tabs = self.get_tabs()?;

        // First pass: check visibility + focus (strongest signal)
        for tab in &tabs {
            let result = tab.evaluate(
                "document.visibilityState === 'visible' && document.hasFocus()",
                false,
            );
            if let Ok(obj) = result
                && let Some(value) = obj.value
                && value.as_bool().unwrap_or(false)
            {
                return Ok(tab.clone());
            }
        }

        // Second pass: check just visibility
        for tab in &tabs {
            let result = tab.evaluate("document.visibilityState === 'visible'", false);
            if let Ok(obj) = result
                && let Some(value) = obj.value
                && value.as_bool().unwrap_or(false)
            {
                return Ok(tab.clone());
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
    pub fn navigate(&mut self, url: &str) -> Result<(), String> {
        self.tab()?
            .navigate_to(url)
            .map_err(|e| format!("Failed to navigate to {}: {}", url, e))?;
        Ok(())
    }

    /// Wait for navigation to complete.
    pub fn wait_for_navigation(&mut self) -> Result<(), String> {
        self.tab()?
            .wait_until_navigated()
            .map(|_| ())
            .map_err(|e| format!("Navigation timeout: {}", e))
    }

    /// Navigate back in browser history.
    pub fn go_back(&mut self) -> Result<(), String> {
        let js = "(function() { window.history.back(); return true; })()";
        self.tab()?
            .evaluate(js, false)
            .map_err(|e| format!("Failed to go back: {}", e))?;
        std::thread::sleep(Duration::from_millis(300));
        Ok(())
    }

    /// Navigate forward in browser history.
    pub fn go_forward(&mut self) -> Result<(), String> {
        let js = "(function() { window.history.forward(); return true; })()";
        self.tab()?
            .evaluate(js, false)
            .map_err(|e| format!("Failed to go forward: {}", e))?;
        std::thread::sleep(Duration::from_millis(300));
        Ok(())
    }

    // ── DOM extraction ──

    /// Extract the DOM tree from the active tab.
    pub fn extract_dom(&mut self) -> Result<DomTree, String> {
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
    pub fn get_selector_for_index(&mut self, index: usize) -> Result<String, String> {
        let dom = self.extract_dom()?;
        dom.get_selector(index)
            .cloned()
            .ok_or_else(|| format!("No element with index {}", index))
    }

    /// Click an element by CSS selector.
    pub fn click_element(&mut self, selector: &str) -> Result<(), String> {
        let tab = self.tab()?;
        let element = self.find_element(&tab, selector)?;
        element
            .click()
            .map(|_| ())
            .map_err(|e| format!("Click failed: {}", e))
    }

    /// Type text into an input element.
    pub fn type_text(&mut self, selector: &str, text: &str) -> Result<(), String> {
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
    pub fn hover_element(&mut self, selector: &str) -> Result<(), String> {
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
    pub fn select_option(&mut self, selector: &str, value: &str) -> Result<(), String> {
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
    pub fn press_key(&mut self, key: &str) -> Result<(), String> {
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
    pub fn scroll(&mut self, direction: &str, amount: u32) -> Result<(), String> {
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
    pub fn screenshot(&mut self) -> Result<String, String> {
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
    pub fn evaluate(&mut self, script: &str) -> Result<serde_json::Value, String> {
        let tab = self.tab()?;
        let result = tab
            .evaluate(script, false)
            .map_err(|e| format!("JavaScript evaluation failed: {}", e))?;

        let value = result.value.unwrap_or(serde_json::Value::Null);
        Ok(value)
    }

    // ── Content extraction ──

    /// Get the page content as Markdown.
    pub fn get_markdown(&mut self) -> Result<String, String> {
        let tab = self.tab()?;
        let html = tab
            .get_content()
            .map_err(|e| format!("Failed to get page content: {}", e))?;
        Ok(html2md::parse_html(&html))
    }

    /// Get the page's full HTML.
    pub fn get_html(&mut self) -> Result<String, String> {
        let tab = self.tab()?;
        tab.get_content()
            .map_err(|e| format!("Failed to get HTML: {}", e))
    }

    /// Extract all links from the page.
    pub fn read_links(&mut self) -> Result<String, String> {
        let js = r#"(() => {
            const links = Array.from(document.querySelectorAll('a[href]'));
            return JSON.stringify(links.map((a, i) => ({
                index: i,
                text: (a.textContent || '').trim().substring(0, 200),
                href: a.href
            })));
        })()"#;

        let result = self
            .tab()?
            .evaluate(js, false)
            .map_err(|e| format!("Failed to read links: {}", e))?;

        let value = result.value.unwrap_or(serde_json::Value::Null);
        let links: Vec<serde_json::Value> = serde_json::from_value(value).unwrap_or_default();

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

    /// A session that was never started must not resurrect Chrome after a
    /// deliberate `close()` — only `browser_restart` may. Without this split,
    /// closing would silently relaunch what the agent just tore down.
    #[test]
    fn closed_session_stays_down_instead_of_respawning() {
        let mut session = BrowserSession::lazy(LaunchOptions::default());
        session.close().expect("close");
        let err = match session.browser() {
            Ok(_) => panic!("a closed session must stay down"),
            Err(err) => err,
        };
        assert!(err.contains("browser_restart"), "{err}");
    }

    /// Building a lazy session must not need a browser at all.
    #[test]
    fn lazy_session_constructs_without_chrome() {
        let _session = BrowserSession::lazy(LaunchOptions::default());
    }

    #[test]
    #[ignore] // Requires Chrome installed
    fn test_launch_and_navigate() {
        let mut session =
            BrowserSession::launch(LaunchOptions::default()).expect("Failed to launch browser");
        assert!(session.get_tabs().is_ok());

        session.navigate("about:blank").expect("Navigation failed");
        session
            .wait_for_navigation()
            .expect("Wait for navigation failed");
    }

    #[test]
    #[ignore] // Requires Chrome installed
    fn test_restart_replaces_browser() {
        let mut session =
            BrowserSession::launch(LaunchOptions::default()).expect("Failed to launch browser");
        let first_tab_count = session.get_tabs().expect("tabs").len();
        assert!(first_tab_count >= 1);

        // Restart should tear down the old instance and bring up a fresh one.
        session.restart().expect("Failed to restart browser");
        let new_tabs = session.get_tabs().expect("tabs after restart");
        assert!(!new_tabs.is_empty(), "restart must leave a usable browser");

        // The fresh browser can navigate.
        session
            .navigate("about:blank")
            .expect("Navigation failed after restart");
        session.wait_for_navigation().expect("Wait after restart");
    }

    #[test]
    #[ignore]
    fn test_extract_dom() {
        let mut session =
            BrowserSession::launch(LaunchOptions::default()).expect("Failed to launch browser");
        session.navigate("about:blank").expect("Navigation failed");
        session
            .wait_for_navigation()
            .expect("Wait for navigation failed");

        let dom = session.extract_dom().expect("DOM extraction failed");
        assert!(dom.count_nodes() > 0);
    }

    #[test]
    #[ignore]
    fn test_screenshot() {
        let mut session =
            BrowserSession::launch(LaunchOptions::default()).expect("Failed to launch browser");
        session.navigate("about:blank").expect("Navigation failed");

        let data = session.screenshot().expect("Screenshot failed");
        assert!(!data.is_empty());
    }
}
