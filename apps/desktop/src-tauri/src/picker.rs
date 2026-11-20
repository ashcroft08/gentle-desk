//! The two dialog seams: the folder picker (Q1, W3c) and the file picker the
//! composer's clip asks (design/composer-dock T4, S3).
//!
//! Asking a human for a folder or a file can take minutes, so it is never done on
//! the window's thread and never under a lock: every caller runs it on a thread
//! of its own (the two workspace commands and `pick_files` spawn one through
//! `tauri::async_runtime::spawn_blocking`).
//!
//! [`FolderPicker`] and [`FilePicker`] are the seams: the workspace flows and the
//! clip take them by `&dyn`, so a test answers at once instead of opening a
//! window, and the two real implementations are the `rfd` dialogs. Q1 option A,
//! decided by the user: the Tauri dialog plugin was measured and dropped, because
//! its `common-controls-v6` link cost cannot be kept out of a test binary (see
//! `odd/tasks/workspaces-design.md`, "Q4 answer" and "W3c decisions").

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use tauri::Wry;

/// The window a dialog is owned by, shared by the folder picker and the file
/// picker: only the setup step has a window, and it installs it here once for
/// both, so the two dialogs are modal to the app the same way.
#[derive(Clone, Default)]
pub struct DialogParent(Arc<Mutex<Option<tauri::WebviewWindow<Wry>>>>);

impl DialogParent {
    /// The window a dialog is owned by right now, `None` before the setup step.
    fn get(&self) -> Option<tauri::WebviewWindow<Wry>> {
        lock(&self.0).clone()
    }

    /// Makes every later dialog owned by `window`.
    pub fn set(&self, window: tauri::WebviewWindow<Wry>) {
        *lock(&self.0) = Some(window);
    }
}

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
    parent: DialogParent,
}

impl NativeFolderPicker {
    /// A picker whose dialogs are not owned by any window yet.
    pub fn new() -> Self {
        Self {
            parent: DialogParent::default(),
        }
    }

    /// The window this picker's dialogs are owned by, as the file picker takes
    /// it: one install at startup owns both dialogs.
    pub fn parent(&self) -> DialogParent {
        self.parent.clone()
    }

    /// Makes every later dialog owned by `window`. Called once, from the setup
    /// step, which is the only place with a window to give.
    pub fn set_parent(&self, window: tauri::WebviewWindow<Wry>) {
        self.parent.set(window);
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
        let parent = self.parent.get();
        let dialog = rfd::FileDialog::new().set_title(title.to_owned());
        match parent {
            // rfd sets the dialog's owner from the window handle, which is what
            // makes it modal to the window rather than to the desktop.
            Some(window) => dialog.set_parent(&window).pick_folder(),
            None => dialog.pick_folder(),
        }
    }
}

/// What asks the human for files. Implemented by the real dialog
/// ([`NativeFilePicker`]) and by the fake the attachment tests inject.
pub trait FilePicker: Send + Sync {
    /// The files the human picked, oldest first; empty when they cancelled the
    /// dialog.
    fn pick_files(&self, title: &str) -> Vec<PathBuf>;
}

/// The real file picker: rfd's Win32 open dialog, several files at a time, owned
/// by the same window as the folder picker.
pub struct NativeFilePicker {
    parent: DialogParent,
}

impl NativeFilePicker {
    /// A picker whose dialogs are not owned by any window yet.
    pub fn new() -> Self {
        Self {
            parent: DialogParent::default(),
        }
    }

    /// A picker that shares `parent` with the folder picker, so one install at
    /// startup owns both dialogs.
    pub fn with_parent(parent: DialogParent) -> Self {
        Self { parent }
    }
}

impl Default for NativeFilePicker {
    fn default() -> Self {
        Self::new()
    }
}

impl FilePicker for NativeFilePicker {
    fn pick_files(&self, title: &str) -> Vec<PathBuf> {
        // Same as the folder dialog: the window comes off the lock before the
        // dialog opens, and a cancelled pick is the empty list.
        let parent = self.parent.get();
        let dialog = rfd::FileDialog::new().set_title(title.to_owned());
        match parent {
            Some(window) => dialog.set_parent(&window).pick_files(),
            None => dialog.pick_files(),
        }
        .unwrap_or_default()
    }
}

/// A lock that survives a poisoned mutex: nothing here may panic because a
/// thread died holding one.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}
