//! Windowless tests of the theme setting (the v1 Appearance setting): the value
//! the web side sends back lives in the app config, so it survives a restart,
//! and the window's own chrome follows it through the seam
//! ([`WindowTheme`]) instead of through a Tauri type.
//!
//! A temp directory holds the config file, so the user's own config is never
//! read. A save runs on the writer's own thread, so a test that reads the file
//! back waits for it, bounded.

use std::fs;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::config::{ConfigSnapshot, ConfigWriter};
use gentle_desk_lib::contract::{ThemeView, UiIntent};
use gentle_desk_lib::theme::WindowTheme;
use gentle_runtime::app_config::{self, AppConfig, Theme};

/// Every wait of these tests: a save of one small file, never a hang.
const WAIT: Duration = Duration::from_secs(10);

/// How long a file is watched after the change that should not have touched it.
const SETTLE: Duration = Duration::from_millis(100);

/// The window chrome a test injects: it remembers every theme it was asked to
/// apply, and touches no window at all.
#[derive(Default)]
struct FakeWindowTheme {
    applied: Mutex<Vec<Theme>>,
    /// Whether every application fails, which is what a window that is already
    /// gone does.
    failing: bool,
}

impl FakeWindowTheme {
    fn failing() -> Self {
        Self {
            applied: Mutex::new(Vec::new()),
            failing: true,
        }
    }

    fn applied(&self) -> Vec<Theme> {
        lock(&self.applied).clone()
    }
}

impl WindowTheme for FakeWindowTheme {
    fn set(&self, theme: Theme) -> Result<(), String> {
        lock(&self.applied).push(theme);
        if self.failing {
            return Err("the window is gone".to_owned());
        }
        Ok(())
    }
}

#[test]
fn a_run_with_no_chosen_theme_starts_on_the_system_theme() {
    let holder = AppStateHolder::default();

    assert_eq!(holder.view().theme, ThemeView::System);
}

#[test]
fn setting_a_theme_reaches_the_state_the_file_and_the_revision() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);
    let before = holder.view();

    let after = holder.dispatch(UiIntent::SetTheme {
        theme: ThemeView::Gentle,
    });

    assert_eq!(
        after.theme,
        ThemeView::Gentle,
        "the push carries the choice"
    );
    assert_ne!(
        after.sessions_revision, before.sessions_revision,
        "the config moved, so the revision says so"
    );
    let loaded = wait_for("the theme to reach the config file", || {
        let loaded = app_config::load(&path);
        (loaded.config.theme == Theme::Gentle).then_some(loaded)
    });
    assert_eq!(loaded.notice, None, "the file was written, not damaged");
    assert_eq!(loaded.config.theme, Theme::Gentle);
}

#[test]
fn setting_the_same_theme_again_changes_nothing_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);
    holder.dispatch(UiIntent::SetTheme {
        theme: ThemeView::Gentle,
    });
    wait_for("the first theme to reach the file", || {
        (app_config::load(&path).config.theme == Theme::Gentle).then_some(())
    });

    let before = holder.view();
    let written = fs::read(&path).unwrap();
    let after = holder.dispatch(UiIntent::SetTheme {
        theme: ThemeView::Gentle,
    });
    std::thread::sleep(SETTLE);

    assert_eq!(
        after, before,
        "the same theme changes no slice of the state"
    );
    assert_eq!(
        fs::read(&path).unwrap(),
        written,
        "and asks for no save at all"
    );
}

#[test]
fn two_quick_theme_changes_leave_the_last_one_in_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    // The file starts on another theme, so the last change of the test is a
    // visible one: nothing here passes because the file never moved.
    let mut saved = AppConfig::default();
    assert!(saved.set_theme(Theme::Dark));
    app_config::save(&path, &saved).unwrap();
    let holder = holder_at(&path);

    holder.dispatch(UiIntent::SetTheme {
        theme: ThemeView::Gentle,
    });
    holder.dispatch(UiIntent::SetTheme {
        theme: ThemeView::Light,
    });

    // One writer, one order: whatever it wrote in between, the last value is
    // what the file holds, and it stays there.
    wait_for("the last theme to reach the file", || {
        (app_config::load(&path).config.theme == Theme::Light).then_some(())
    });
    std::thread::sleep(SETTLE);
    assert_eq!(app_config::load(&path).config.theme, Theme::Light);
}

#[test]
fn the_window_chrome_follows_the_theme_at_startup_and_on_every_change() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let mut saved = AppConfig::default();
    assert!(saved.set_theme(Theme::Gentle));
    app_config::save(&path, &saved).unwrap();
    let holder = holder_at(&path);
    let window = std::sync::Arc::new(FakeWindowTheme::default());
    holder.arm_window_theme(window.clone());

    // Startup: the theme of this run reaches the window before any push.
    holder.apply_window_theme();
    assert_eq!(window.applied(), [Theme::Gentle]);

    for (intent, expected) in [
        (ThemeView::System, Theme::System),
        (ThemeView::Dark, Theme::Dark),
        (ThemeView::Gentle, Theme::Gentle),
        (ThemeView::GentlemanCute, Theme::GentlemanCute),
        (ThemeView::GentlemanSexy, Theme::GentlemanSexy),
        (ThemeView::Light, Theme::Light),
    ] {
        holder.dispatch(UiIntent::SetTheme { theme: intent });
        assert_eq!(
            window.applied().last(),
            Some(&expected),
            "{intent:?} is the theme the window was asked for"
        );
    }
    assert_eq!(
        window.applied(),
        [
            Theme::Gentle,
            Theme::System,
            Theme::Dark,
            Theme::Gentle,
            Theme::GentlemanCute,
            Theme::GentlemanSexy,
            Theme::Light,
        ],
        "the seam is asked once per change, in order"
    );
}

#[test]
fn asking_for_the_same_theme_again_does_not_touch_the_window() {
    let holder = AppStateHolder::default();
    let window = std::sync::Arc::new(FakeWindowTheme::default());
    holder.arm_window_theme(window.clone());

    holder.dispatch(UiIntent::SetTheme {
        theme: ThemeView::Light,
    });
    holder.dispatch(UiIntent::SetTheme {
        theme: ThemeView::Light,
    });

    assert_eq!(window.applied(), [Theme::Light]);
}

#[test]
fn a_window_that_refuses_the_theme_keeps_the_choice_in_memory() {
    // A window that is already gone fails the call; the setting is the user's
    // and the run keeps it, so the next push still carries it.
    let holder = AppStateHolder::default();
    holder.arm_window_theme(std::sync::Arc::new(FakeWindowTheme::failing()));

    let after = holder.dispatch(UiIntent::SetTheme {
        theme: ThemeView::GentlemanSexy,
    });

    assert_eq!(after.theme, ThemeView::GentlemanSexy);
    assert_eq!(after.config_notice, None, "not a config problem");
}

#[test]
fn a_fold_and_a_theme_change_do_not_overwrite_each_other() {
    let dir = tempfile::tempdir().unwrap();
    let repo = folder(dir.path(), "repo");
    let path = dir.path().join("config.json");
    let mut saved = AppConfig::default();
    assert!(saved.add_workspace(&repo));
    app_config::save(&path, &saved).unwrap();
    let holder = holder_at(&path);

    holder.dispatch(UiIntent::SetTheme {
        theme: ThemeView::GentlemanCute,
    });
    let after_fold = holder.dispatch(UiIntent::SetWorkspaceCollapsed {
        path: repo.clone(),
        collapsed: true,
    });

    assert_eq!(
        after_fold.theme,
        ThemeView::GentlemanCute,
        "the fold kept the chosen theme"
    );
    let loaded = wait_for("both edits to reach the file", || {
        let loaded = app_config::load(&path);
        let folded = loaded
            .config
            .workspace(&repo)
            .is_some_and(|entry| entry.collapsed);
        (folded && loaded.config.theme == Theme::GentlemanCute).then_some(loaded)
    });
    assert_eq!(loaded.config.theme, Theme::GentlemanCute);

    // And the other way round: a theme change kept the fold.
    let after_theme = holder.dispatch(UiIntent::SetTheme {
        theme: ThemeView::Dark,
    });
    let page = holder.sessions_window(0, 8);
    assert!(
        matches!(
            page.rows.first(),
            Some(gentle_desk_lib::contract::SessionTreeRow::Workspace(row)) if row.collapsed
        ),
        "the theme change kept the fold, got {:?}",
        page.rows
    );
    assert_eq!(after_theme.theme, ThemeView::Dark);
}

#[test]
fn the_chosen_theme_survives_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);
    holder.dispatch(UiIntent::SetTheme {
        theme: ThemeView::GentlemanSexy,
    });
    wait_for("the theme to reach the config file", || {
        (app_config::load(&path).config.theme == Theme::GentlemanSexy).then_some(())
    });

    // The next start: the same file, read before the holder exists.
    let next = AppStateHolder::with_config(ConfigSnapshot::at(Some(&path)));

    assert_eq!(next.view().theme, ThemeView::GentlemanSexy);
}

/// A holder over the config file at `path`, with the one writer armed.
fn holder_at(path: &Path) -> AppStateHolder {
    let holder = AppStateHolder::with_config(ConfigSnapshot::at(Some(path)));
    holder.arm_config_saves(ConfigWriter::spawn(Some(path.to_path_buf())));
    holder
}

/// A folder of `dir`, created, as the text the config stores.
fn folder(dir: &Path, name: &str) -> String {
    let path = dir.join(name);
    fs::create_dir_all(&path).unwrap();
    path.to_string_lossy().into_owned()
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
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
