//! The seam of the "New workspace" tests: a picker a test answers with, and the
//! temp paths and waits the two workspace flows share.
//!
//! The flows take a [`FolderPicker`], so nothing here opens a window: the fake
//! records every title it was asked for and answers the folder it was given, or
//! a cancel. The config file of each test lives in its own temp directory, so
//! the user's config and home are never read.
#![allow(dead_code)] // each test binary uses one part of it

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::config::{ConfigSnapshot, ConfigWriter};
use gentle_desk_lib::picker::FolderPicker;

/// Every wait of these tests: a save of one small file, never a hang.
pub const WAIT: Duration = Duration::from_secs(10);

/// How long a file or a folder is watched after the action that should not have
/// touched it.
pub const SETTLE: Duration = Duration::from_millis(150);

/// A picker a test answers with: the folder it was told, or a cancel.
pub struct FakePicker {
    answer: Option<PathBuf>,
    titles: Mutex<Vec<String>>,
}

impl FakePicker {
    /// A picker that answers `path`.
    pub fn picking(path: &str) -> Self {
        Self {
            answer: Some(PathBuf::from(path)),
            titles: Mutex::new(Vec::new()),
        }
    }

    /// A picker whose dialog the user cancelled.
    pub fn cancelled() -> Self {
        Self {
            answer: None,
            titles: Mutex::new(Vec::new()),
        }
    }

    /// How many dialogs the flows asked this picker to open.
    pub fn calls(&self) -> usize {
        lock(&self.titles).len()
    }
}

impl FolderPicker for FakePicker {
    fn pick_folder(&self, title: &str) -> Option<PathBuf> {
        lock(&self.titles).push(title.to_owned());
        self.answer.clone()
    }
}

/// A holder whose config file is `path`, with the one writer of the file armed.
pub fn holder_at(path: &Path) -> AppStateHolder {
    let holder = AppStateHolder::with_config(ConfigSnapshot::at(Some(path)));
    holder.arm_config_saves(ConfigWriter::spawn(Some(path.to_path_buf())));
    holder
}

/// A folder of `dir`, created, as the text the contract carries.
pub fn folder(dir: &Path, name: &str) -> String {
    let path = dir.join(name);
    std::fs::create_dir_all(&path).unwrap();
    text(&path)
}

/// A path as the text the contract carries it.
pub fn text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// The entries of a folder, sorted, so a test can say nothing was created.
pub fn entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// Waits for what a bounded wait is about, failing the test instead of hanging.
pub fn wait_for<T>(what: &str, mut probe: impl FnMut() -> Option<T>) -> T {
    let deadline = Instant::now() + WAIT;
    loop {
        if let Some(value) = probe() {
            return value;
        }
        assert!(
            Instant::now() < deadline,
            "waited {WAIT:?} for {what} and it never came"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// A lock that survives a poisoned mutex.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}
