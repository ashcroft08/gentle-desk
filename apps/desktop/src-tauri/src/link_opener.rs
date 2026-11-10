//! Opening a link in the system browser (T5a), behind a seam.
//!
//! [`LinkOpener`] is the seam: the bridge holds one `Arc<dyn LinkOpener>`, the
//! `open_link` command takes it by `&dyn` on a blocking thread, and a test
//! injects a fake that records the urls instead of starting a browser.
//!
//! [`SystemLinkOpener`] is the one real implementation: the `opener` crate. The
//! Tauri shell/opener plugin is not used, for the folder picker's reason
//! (`odd/tasks/workspaces-design.md`, "Q4 answer"): a plugin's link cost cannot
//! be kept out of a test binary.
//!
//! [`open`] is the whole rule: the url is checked ([`crate::link`]) BEFORE the
//! opener is called, so a refused url never reaches the browser.

use crate::link;

/// Opening one url in whatever the system uses for it. Implemented by the real
/// opener ([`SystemLinkOpener`]) and by the fake each link test injects.
pub trait LinkOpener: Send + Sync {
    /// Hands `url` to the system's default program for it.
    fn open_url(&self, url: &str) -> Result<(), String>;
}

/// The real opener: `opener`, which asks the system for the default program of
/// the url's scheme.
pub struct SystemLinkOpener;

impl SystemLinkOpener {
    /// The one implementation of the seam.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SystemLinkOpener {
    fn default() -> Self {
        Self::new()
    }
}

impl LinkOpener for SystemLinkOpener {
    fn open_url(&self, url: &str) -> Result<(), String> {
        opener::open(url)
            .map_err(|error| format!("the link could not be opened by the system: {error}"))
    }
}

/// Checks the url and then hands it to `opener`: the one path the `open_link`
/// command runs, so what a test covers is exactly what the app runs.
///
/// A failed open is an error the web side receives, never a panic and never a
/// state change: the thread renders nothing different either way.
pub fn open(opener: &dyn LinkOpener, url: &str) -> Result<(), String> {
    link::check(url)?;
    opener.open_url(url)
}
