//! Windowless tests of "skip this version" (0.8.0 settings-polish S1, S2): the
//! version the user refused lives in the app config, so it survives a restart,
//! and it reaches the slice the web side draws as `UpdatesView::skipped` beside
//! the rows the check read.
//!
//! A temp directory holds the config file, so the user's own config is never
//! read. A save runs on the writer's own thread, so a test that reads the file
//! back waits for it, bounded.

use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::config::{ConfigSnapshot, ConfigWriter};
use gentle_desk_lib::contract::UiIntent;
use gentle_runtime::app_config;

/// Every wait of these tests: a save of one small file, never a hang.
const WAIT: Duration = Duration::from_secs(10);

/// How long a file is watched after a change that should not have touched it.
const SETTLE: Duration = Duration::from_millis(100);

#[test]
fn skipping_a_version_reaches_the_slice_and_the_config_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);

    let after = holder.dispatch(UiIntent::SkipUpdate {
        version: "2.0.5".to_owned(),
    });

    assert_eq!(
        after.updates.skipped.as_deref(),
        Some("2.0.5"),
        "the push carries the skip, which is what the pane and the gear read"
    );
    let loaded = wait_for("the skip to reach the config file", || {
        let loaded = app_config::load(&path);
        (loaded.config.skipped_update.is_some()).then_some(loaded)
    });
    assert_eq!(loaded.notice, None, "the file was written, not damaged");
    assert_eq!(loaded.config.skipped_update.as_deref(), Some("2.0.5"));
}

#[test]
fn the_skip_survives_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);
    holder.dispatch(UiIntent::SkipUpdate {
        version: "2.0.5".to_owned(),
    });
    wait_for("the skip to reach the config file", || {
        (app_config::load(&path).config.skipped_update.is_some()).then_some(())
    });

    // The next start: the same file, read before the holder exists.
    let next = AppStateHolder::with_config(ConfigSnapshot::at(Some(&path)));

    assert_eq!(next.view().updates.skipped.as_deref(), Some("2.0.5"));
    assert_eq!(next.skipped_update().as_deref(), Some("2.0.5"));
}

#[test]
fn stopping_the_skip_clears_it_from_the_slice_and_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let mut saved = app_config::AppConfig::default();
    assert!(saved.set_skipped_update("2.0.5"));
    app_config::save(&path, &saved).unwrap();
    let holder = holder_at(&path);
    assert_eq!(
        holder.view().updates.skipped.as_deref(),
        Some("2.0.5"),
        "the skip this run holds comes from the config"
    );

    let after = holder.dispatch(UiIntent::UnskipUpdate);

    assert_eq!(after.updates.skipped, None);
    wait_for("the cleared skip to reach the config file", || {
        (app_config::load(&path).config.skipped_update.is_none()).then_some(())
    });
}

#[test]
fn skipping_a_value_this_app_cannot_read_stores_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);

    let after = holder.dispatch(UiIntent::SkipUpdate {
        version: "latest".to_owned(),
    });
    std::thread::sleep(SETTLE);

    assert_eq!(
        after.updates.skipped, None,
        "a value that is not a version is no skip at all, so it hides nothing"
    );
    assert!(
        !path.exists(),
        "and nothing was written: the config is exactly as it was"
    );
}

#[test]
fn skipping_the_same_version_again_changes_nothing_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);
    holder.dispatch(UiIntent::SkipUpdate {
        version: "2.0.5".to_owned(),
    });
    wait_for("the first skip to reach the file", || {
        (app_config::load(&path).config.skipped_update.is_some()).then_some(())
    });

    let before = holder.view();
    let written = fs::read(&path).unwrap();
    let after = holder.dispatch(UiIntent::SkipUpdate {
        version: "2.0.5".to_owned(),
    });
    std::thread::sleep(SETTLE);

    assert_eq!(after, before, "the same skip changes no slice of the state");
    assert_eq!(
        fs::read(&path).unwrap(),
        written,
        "and asks for no save at all"
    );
}

/// A holder over the config file at `path`, with the one writer armed.
fn holder_at(path: &Path) -> AppStateHolder {
    let holder = AppStateHolder::with_config(ConfigSnapshot::at(Some(path)));
    holder.arm_config_saves(ConfigWriter::spawn(Some(path.to_path_buf())));
    holder
}

/// Waits for what a bounded wait is about, failing the test instead of hanging.
fn wait_for<T>(what: &str, mut probe: impl FnMut() -> Option<T>) -> T {
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
