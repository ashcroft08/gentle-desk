//! Link-level proof of the clipboard and the browser opener (T5a).
//!
//! The Tauri clipboard/browser PLUGINS were avoided: the folder picker's history
//! (Q4, W3c) is that a plugin's link cost cannot be kept out of a test binary -
//! the dialog plugin imported comctl32 v6's `TaskDialogIndirect` and the test
//! exe failed to load with `STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139)`.
//!
//! This test binary is the check for the two crates chosen instead: it names
//! the real `arboard` clipboard and the real `opener` browser, so both are
//! linked into it, and it must simply START and pass.
//!
//! Neither one is CALLED: a real write would overwrite the clipboard of the
//! machine running the tests and a real open would start a browser. What is
//! proven is that this binary loads with both linked.

use std::sync::{Arc, Mutex, MutexGuard};

use gentle_desk_lib::clipboard::{Clipboard, SystemClipboard};
use gentle_desk_lib::link_opener::{LinkOpener, SystemLinkOpener};

/// A clipboard that is never the machine's.
#[derive(Default)]
struct FakeClipboard(Mutex<Vec<String>>);

impl Clipboard for FakeClipboard {
    fn copy_text(&self, text: &str) -> Result<(), String> {
        lock(&self.0).push(text.to_owned());
        Ok(())
    }
}

/// An opener that never starts a browser.
#[derive(Default)]
struct FakeOpener(Mutex<Vec<String>>);

impl LinkOpener for FakeOpener {
    fn open_url(&self, url: &str) -> Result<(), String> {
        lock(&self.0).push(url.to_owned());
        Ok(())
    }
}

/// The seam a production caller takes: a trait object, whatever built it.
fn clipboard_seam(clipboard: Arc<dyn Clipboard>) -> Arc<dyn Clipboard> {
    clipboard
}

fn opener_seam(opener: Arc<dyn LinkOpener>) -> Arc<dyn LinkOpener> {
    opener
}

#[test]
fn the_real_clipboard_and_browser_link_into_a_test_binary() {
    // The real ones, built and handed over as the seams the bridge holds: their
    // implementations, and with them arboard and opener, are linked here.
    let real_clipboard = clipboard_seam(Arc::new(SystemClipboard::new()));
    let real_opener = opener_seam(Arc::new(SystemLinkOpener::new()));
    assert!(Arc::strong_count(&real_clipboard) >= 1);
    assert!(Arc::strong_count(&real_opener) >= 1);

    // The seams answer a fake for a test, exactly as the bridge hands them over.
    let fake_clipboard = clipboard_seam(Arc::new(FakeClipboard::default()));
    assert_eq!(fake_clipboard.copy_text("hello"), Ok(()));
    let fake_opener = opener_seam(Arc::new(FakeOpener::default()));
    assert_eq!(fake_opener.open_url("https://example.com/"), Ok(()));
}

/// A lock that survives a poisoned mutex.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}
