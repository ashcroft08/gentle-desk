//! Windowless tests of the notifications setting (notifications S4): one
//! persisted mode, read leniently, written by the same ONE config step the
//! theme and the language take, and re-emitted as the slice the web side draws.
//!
//! A temp directory holds the config file, so the user's own config is never
//! read. A save runs on the writer's own thread, so a test that reads the file
//! back waits for it, bounded.

use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gentle_core::i18n::Language;
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::config::{ConfigSnapshot, ConfigWriter};
use gentle_desk_lib::contract::{LanguageView, NotificationsMode, ThemeView, UiIntent};
use gentle_runtime::app_config::{self, Notifications, Theme};

/// Every wait of these tests: a save of one small file, never a hang.
const WAIT: Duration = Duration::from_secs(10);

/// How long a file is watched after the change that should not have touched it.
const SETTLE: Duration = Duration::from_millis(100);

#[test]
fn a_run_with_no_chosen_mode_notifies_when_the_window_is_not_in_front() {
    let holder = AppStateHolder::default();

    // The maintainer's proposal (L3): on, and only while the window is not in
    // front, until they choose otherwise.
    assert_eq!(
        holder.view().notifications,
        NotificationsMode::WhenUnfocused
    );
}

#[test]
fn setting_the_notifications_mode_reaches_the_state_the_file_and_the_revision() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);
    let before = holder.view();

    let after = holder.dispatch(UiIntent::SetNotifications {
        mode: NotificationsMode::Off,
    });

    assert_eq!(
        after.notifications,
        NotificationsMode::Off,
        "the push carries the choice"
    );
    assert_ne!(
        after.sessions_revision, before.sessions_revision,
        "the config moved, so the revision says so"
    );
    assert_eq!(after.config_notice, None, "not a config problem");
    let loaded = wait_for("the mode to reach the config file", || {
        let loaded = app_config::load(&path);
        (loaded.config.notifications == Notifications::Off).then_some(loaded)
    });
    assert_eq!(loaded.notice, None, "the file was written, not damaged");
    assert_eq!(loaded.config.notifications, Notifications::Off);
}

#[test]
fn setting_the_same_mode_again_changes_nothing_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);
    holder.dispatch(UiIntent::SetNotifications {
        mode: NotificationsMode::Off,
    });
    wait_for("the first mode to reach the file", || {
        (app_config::load(&path).config.notifications == Notifications::Off).then_some(())
    });

    let before = holder.view();
    let written = fs::read(&path).unwrap();
    let after = holder.dispatch(UiIntent::SetNotifications {
        mode: NotificationsMode::Off,
    });
    std::thread::sleep(SETTLE);

    assert_eq!(after, before, "the same mode changes no slice of the state");
    assert_eq!(
        fs::read(&path).unwrap(),
        written,
        "and asks for no save at all"
    );
}

#[test]
fn an_unknown_stored_mode_loads_as_the_default_and_keeps_the_rest_of_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    // A document another build wrote: a mode this version does not know, and
    // every other setting beside it.
    fs::write(
        &path,
        br#"{"version":1,"theme":"Gentle","language":"es","notifications":"always","workspaces":[{"path":"C:/dev/p","collapsed":false}]}"#,
    )
    .unwrap();

    let holder = holder_at(&path);
    let view = holder.view();

    assert_eq!(
        view.notifications,
        NotificationsMode::WhenUnfocused,
        "a mode this version does not know reads as the default"
    );
    assert_eq!(
        view.theme,
        ThemeView::Gentle,
        "the rest of the file is read, not discarded"
    );
    assert_eq!(view.language, Some(LanguageView::Es));
    assert_eq!(view.config_notice, None, "not a damaged config");
    assert!(path.exists(), "the file is not moved aside");
}

#[test]
fn a_save_that_fails_keeps_the_choice_and_sets_the_notice() {
    let dir = tempfile::tempdir().unwrap();
    // The config's parent is a FILE, so no config file can be created there.
    let blocked = dir.path().join("blocked");
    fs::write(&blocked, b"a file, not a folder").unwrap();
    let path = blocked.join("config.json");

    let writer = ConfigWriter::spawn(Some(path.clone()));
    let holder = Arc::new(AppStateHolder::default());
    holder.arm_config_saves(writer.clone());
    // The bridge's wiring, in one line: the failure becomes the config notice.
    let holder_for_failure = Arc::clone(&holder);
    writer.on_failure(Arc::new(move |message: String| {
        holder_for_failure.note_config_problem(message);
    }));

    let after = holder.dispatch(UiIntent::SetNotifications {
        mode: NotificationsMode::Off,
    });

    assert_eq!(
        after.notifications,
        NotificationsMode::Off,
        "the choice is the user's and the run keeps it in memory"
    );
    let notice = wait_for("the failed save to be reported", || {
        holder.view().config_notice.clone()
    });
    assert!(
        notice.contains("could not be saved"),
        "the reason names the save, got {notice:?}"
    );
    assert!(!path.exists(), "a failed save creates no file");
    assert_eq!(
        holder.view().notifications,
        NotificationsMode::Off,
        "the pushed slice still carries the choice"
    );
}

#[test]
fn the_theme_the_language_and_the_mode_all_round_trip_together() {
    // PRESERVE: the settings that already shared this write path keep working
    // with the new field beside them, in one document.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);

    holder.dispatch(UiIntent::SetTheme {
        theme: ThemeView::GentlemanSexy,
    });
    holder.dispatch(UiIntent::SetLanguage {
        language: LanguageView::Es,
    });
    holder.dispatch(UiIntent::SetNotifications {
        mode: NotificationsMode::Off,
    });

    let loaded = wait_for("all three settings to reach the file", || {
        let loaded = app_config::load(&path);
        let held = loaded.config.theme == Theme::GentlemanSexy
            && loaded.config.language == Some(Language::Es)
            && loaded.config.notifications == Notifications::Off;
        held.then_some(loaded)
    });
    assert_eq!(loaded.notice, None);

    // The next start reads all three back from the same file.
    let next = AppStateHolder::with_config(ConfigSnapshot::at(Some(&path)));
    let view = next.view();
    assert_eq!(view.theme, ThemeView::GentlemanSexy);
    assert_eq!(view.language, Some(LanguageView::Es));
    assert_eq!(view.notifications, NotificationsMode::Off);
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
