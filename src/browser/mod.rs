//! Browser automation module.
//!
//! Provides `BrowserToolset` (lifecycle manager) and 21 tools implementing
//! `agent_base::Tool` for CDP-based browser automation.
//!
//! Gated behind the `browser` Cargo feature.

pub mod config;
pub mod dom;
pub mod session;
pub mod tools;

use std::sync::{Arc, Mutex};

use self::config::{ConnectionOptions, LaunchOptions};
use self::session::BrowserSession;

/// Owns a `BrowserSession` and provides shared access to tools.
///
/// ```ignore
/// use phi_tools::browser::BrowserToolset;
///
/// let browser = BrowserToolset::launch(Default::default())?;
/// let session = browser.session(); // Arc<Mutex<BrowserSession>>
/// ```
pub struct BrowserToolset {
    session: Arc<Mutex<BrowserSession>>,
}

impl BrowserToolset {
    /// Launch a new Chrome/Chromium instance.
    pub fn launch(options: LaunchOptions) -> Result<Self, String> {
        let session = BrowserSession::launch(options)?;
        Ok(Self {
            session: Arc::new(Mutex::new(session)),
        })
    }

    /// Connect to an existing browser via WebSocket.
    pub fn connect(options: ConnectionOptions) -> Result<Self, String> {
        let session = BrowserSession::connect(options)?;
        Ok(Self {
            session: Arc::new(Mutex::new(session)),
        })
    }

    /// Get a shared reference to the browser session.
    pub fn session(&self) -> Arc<Mutex<BrowserSession>> {
        self.session.clone()
    }

    /// Check if the browser is still alive.
    pub fn is_alive(&self) -> bool {
        self.session.lock().map(|s| s.is_alive()).unwrap_or(false)
    }
}
