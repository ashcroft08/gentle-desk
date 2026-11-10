//! The message Copy action (T5a): writing one message's text to the system
//! clipboard, off the caller's thread.
//!
//! [`Clipboard`] is the seam: the bridge holds one `Arc<dyn Clipboard>`, the
//! command sink that runs `Command::CopyToClipboard` takes it by `&dyn`, and a
//! test injects a fake that records what it was asked to write instead of
//! overwriting the clipboard of the machine running the tests.
//!
//! [`SystemClipboard`] is the one real implementation: the `arboard` crate in
//! text mode, with its default features off. The Tauri clipboard plugin is not
//! used: the folder picker's history (`odd/tasks/workspaces-design.md`, "Q4
//! answer") is that a plugin's link cost cannot be kept out of a test binary.
//!
//! A copy never blocks the thread that asked for it and never holds a lock: the
//! write happens on a thread of its own, and a failure is one line on stderr -
//! the notices component (T7) is where the user is told about such things.

use std::sync::Arc;

/// Writing text to the system clipboard. Implemented by the real clipboard
/// ([`SystemClipboard`]) and by the fake each copy test injects.
pub trait Clipboard: Send + Sync {
    /// Writes `text` to the clipboard, replacing what it held.
    fn copy_text(&self, text: &str) -> Result<(), String>;
}

/// The real clipboard: `arboard`, created per write.
///
/// Nothing of it is kept between writes, so a clipboard held by another program
/// when the first copy fails is not a broken clipboard for every later copy.
pub struct SystemClipboard;

impl SystemClipboard {
    /// The one implementation of the seam.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SystemClipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Clipboard for SystemClipboard {
    fn copy_text(&self, text: &str) -> Result<(), String> {
        let mut board = arboard::Clipboard::new()
            .map_err(|error| format!("the system clipboard is unavailable: {error}"))?;
        board
            .set_text(text.to_owned())
            .map_err(|error| format!("the text could not be written to the clipboard: {error}"))
    }
}

/// Writes `text` on a thread of its own, so neither the window's thread nor the
/// thread that folded the intent waits on the clipboard, and no lock is held
/// while it is written.
///
/// A failure is logged and nothing else: it changes no state the view renders,
/// so a push claiming otherwise would be a lie the user cannot act on.
pub fn copy_off_thread(clipboard: Arc<dyn Clipboard>, text: String) {
    let spawned = std::thread::Builder::new()
        .name("clipboard-copy".to_owned())
        .spawn(move || {
            if let Err(error) = clipboard.copy_text(&text) {
                eprintln!("a message could not be copied: {error}");
            }
        });
    if let Err(error) = spawned {
        eprintln!("the clipboard thread is unavailable: {error}");
    }
}
