//! The window's own chrome: the native title bar follows the chosen theme.
//!
//! The page's colours are the web side's (`apps/desktop/src/theme.css`), and the
//! title bar belongs to the operating system, so it is set from here through
//! Tauri's window theme. `light` is the only theme with a light background, so
//! it asks for a light title bar; the four dark palettes ask for a dark one; and
//! `system` asks for nothing at all, which leaves the window following the
//! reader's own setting exactly as the page does.
//!
//! [`WindowTheme`] is the seam. The real window needs an `AppHandle`, which only
//! a running app has, so the bridge's holder holds an `Arc<dyn WindowTheme>` and
//! a windowless test injects a fake that records what it was asked for
//! (`tests/theme_intent.rs`). [`SystemWindowTheme`] is the one real
//! implementation, built once by the setup step, which is the only place that
//! has the window.

use tauri::Wry;

use gentle_runtime::app_config::Theme;

/// What makes the window's own chrome follow `theme`. Implemented by the real
/// window ([`SystemWindowTheme`]) and by the fake each theme test injects.
pub trait WindowTheme: Send + Sync {
    /// Applies `theme` to the window, or says why it could not: a window that is
    /// already gone refuses, and the run keeps the choice in memory.
    fn set(&self, theme: Theme) -> Result<(), String>;
}

/// The real window: Tauri's own window theme on the main window.
pub struct SystemWindowTheme {
    window: tauri::WebviewWindow<Wry>,
}

impl SystemWindowTheme {
    /// The sink for `window`, the one the app changes.
    pub fn new(window: tauri::WebviewWindow<Wry>) -> Self {
        Self { window }
    }
}

impl WindowTheme for SystemWindowTheme {
    fn set(&self, theme: Theme) -> Result<(), String> {
        self.window
            .set_theme(native_theme(theme))
            .map_err(|error| error.to_string())
    }
}

/// The theme the window itself asks the operating system for. `None` is "follow
/// the reader's own setting", which is what `system` means for the page too.
fn native_theme(theme: Theme) -> Option<tauri::Theme> {
    match theme {
        Theme::System => None,
        Theme::Light => Some(tauri::Theme::Light),
        Theme::Dark | Theme::Gentle | Theme::GentlemanCute | Theme::GentlemanSexy => {
            Some(tauri::Theme::Dark)
        }
    }
}

#[cfg(test)]
mod tests;
