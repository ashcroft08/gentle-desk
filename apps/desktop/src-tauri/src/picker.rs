//! The folder picker seam (Q1, W3c).
//!
//! Asking a human for a folder can take minutes, so it is never done on the
//! window's thread and never under a lock: every caller runs it on a thread of
//! its own (the two workspace commands spawn one through
//! `tauri::async_runtime::spawn_blocking`).
//!
//! [`FolderPicker`] is the seam: the workspace flows take it by `&dyn`, so a
//! test answers at once instead of opening a window, and the one real
//! implementation is the all-in-one `rfd` dialog. Q1 option A, decided by the
//! user: the Tauri dialog plugin was measured and dropped, because its
//! `common-controls-v6` link cost cannot be kept out of a test binary (see
//! `odd/tasks/workspaces-design.md`, "Q4 answer" and "W3c decisions").

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use tauri::Wry;

/// What asks the human for one folder. Implemented by the real dialog
/// ([`NativeFolderPicker`]) and by the fake each workspace test injects.
pub trait FolderPicker: Send + Sync {
    /// The folder the human picked, `None` when they cancelled the dialog.
    fn pick_folder(&self, title: &str) -> Option<PathBuf>;
}

/// The real picker: rfd's Win32 folder dialog, the shell's `IFileDialog`.
///
/// rfd is taken with `default-features = false`: its defaults are the Linux
/// backends, and its `common-controls-v6` feature is what imports comctl32 v6's
/// `TaskDialogIndirect`, which no test binary carries a manifest for.
pub struct NativeFolderPicker {
    /// The window the dialog is owned by, so it is modal to the app. Installed
    /// once at startup, when the window exists; `None` (a pick before the setup
    /// step, or a run with no window) leaves the dialog unowned, which still
    /// opens.
    parent: Mutex<Option<tauri::WebviewWindow<Wry>>>,
}

impl NativeFolderPicker {
    /// A picker whose dialogs are not owned by any window yet.
    pub fn new() -> Self {
        Self {
            parent: Mutex::new(None),
        }
    }

    /// Makes every later dialog owned by `window`. Called once, from the setup
    /// step, which is the only place with a window to give.
    pub fn set_parent(&self, window: tauri::WebviewWindow<Wry>) {
        *lock(&self.parent) = Some(window);
    }
}

impl Default for NativeFolderPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl FolderPicker for NativeFolderPicker {
    fn pick_folder(&self, title: &str) -> Option<PathBuf> {
        // The window is taken off the lock BEFORE the dialog opens: a lock held
        // for the minutes a dialog can stay open would freeze every later call.
        let parent = lock(&self.parent).clone();
        let dialog = rfd::FileDialog::new().set_title(title.to_owned());
        match parent {
            // rfd sets the dialog's owner from the window handle, which is what
            // makes it modal to the window rather than to the desktop.
            Some(window) => dialog.set_parent(&window).pick_folder(),
            None => dialog.pick_folder(),
        }
    }
}

/// A lock that survives a poisoned mutex: nothing here may panic because a
/// thread died holding one.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}
