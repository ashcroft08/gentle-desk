//! Windowless tests of the shell home choice (shell-choice S1-S3, S5): what the
//! app found at start (the folder of the user's pi), the welcome question it
//! asks only when there is something to choose, and the choice kept in the app
//! config, where a restart finds it.
//!
//! A temp directory holds the config file, so the user's own config is never
//! read; a save runs on the writer's thread, so a test that reads the file back
//! waits for it, bounded.

use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use gentle_backend::home::HomeChoice;
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::config::{ConfigSnapshot, ConfigWriter};
use gentle_desk_lib::contract::{HomeChoiceView, UiIntent};
use gentle_runtime::app_config::{self, AppConfig};

const WAIT: Duration = Duration::from_secs(10);
const SETTLE: Duration = Duration::from_millis(100);

const PI: &str = "C:/Users/someone/.pi/agent";

#[test]
fn a_run_that_found_no_pi_asks_nothing_and_has_no_choice() {
    let holder = AppStateHolder::default();

    let shell = holder.view().shell;

    assert_eq!(shell.pi_path, None);
    assert_eq!(shell.choice, None);
    assert!(!shell.ask, "with no pi there is nothing to choose between");
}

#[test]
fn a_run_that_found_pi_and_has_no_choice_asks() {
    let holder = AppStateHolder::default();
    holder.set_pi_path(Some(PI.to_owned()));

    let shell = holder.view().shell;

    assert_eq!(shell.pi_path.as_deref(), Some(PI));
    assert_eq!(shell.choice, None);
    assert!(shell.ask);
}

#[test]
fn choosing_reaches_the_state_the_file_and_stops_the_question() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);
    holder.set_pi_path(Some(PI.to_owned()));
    let before = holder.view();

    let after = holder.dispatch(UiIntent::SetShellHome {
        choice: HomeChoiceView::Link,
    });

    assert_eq!(after.shell.choice, Some(HomeChoiceView::Link));
    assert!(!after.shell.ask, "a chosen home is never asked again");
    assert_ne!(after.sessions_revision, before.sessions_revision);
    let loaded = wait_for("the choice to reach the config file", || {
        let loaded = app_config::load(&path);
        (loaded.config.shell_home == Some(HomeChoice::Link)).then_some(loaded)
    });
    assert_eq!(loaded.notice, None, "the file was written, not damaged");
}

#[test]
fn choosing_the_same_home_again_changes_nothing_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);
    holder.dispatch(UiIntent::SetShellHome {
        choice: HomeChoiceView::Isolated,
    });
    wait_for("the first choice to reach the file", || {
        (app_config::load(&path).config.shell_home == Some(HomeChoice::Isolated)).then_some(())
    });
    let before = holder.view();
    let written = fs::read(&path).unwrap();

    let after = holder.dispatch(UiIntent::SetShellHome {
        choice: HomeChoiceView::Isolated,
    });
    std::thread::sleep(SETTLE);

    assert_eq!(after, before);
    assert_eq!(fs::read(&path).unwrap(), written);
}

#[test]
fn a_stored_choice_is_there_at_the_next_start_and_nothing_is_asked() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let mut saved = AppConfig::default();
    assert!(saved.set_shell_home(HomeChoice::Link));
    app_config::save(&path, &saved).unwrap();
    let holder = holder_at(&path);
    holder.set_pi_path(Some(PI.to_owned()));

    let shell = holder.view().shell;

    assert_eq!(shell.choice, Some(HomeChoiceView::Link));
    assert!(!shell.ask);
}

#[test]
fn a_choice_and_a_theme_change_do_not_overwrite_each_other() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let holder = holder_at(&path);

    holder.dispatch(UiIntent::SetTheme {
        theme: gentle_desk_lib::contract::ThemeView::Light,
    });
    holder.dispatch(UiIntent::SetShellHome {
        choice: HomeChoiceView::Link,
    });

    wait_for("both edits to reach the file", || {
        let config = app_config::load(&path).config;
        (config.shell_home == Some(HomeChoice::Link)
            && config.theme == gentle_runtime::app_config::Theme::Light)
            .then_some(())
    });
}

fn holder_at(path: &Path) -> AppStateHolder {
    let holder = AppStateHolder::with_config(ConfigSnapshot::at(Some(path)));
    holder.arm_config_saves(ConfigWriter::spawn(Some(path.to_path_buf())));
    holder
}

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
