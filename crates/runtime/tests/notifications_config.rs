//! The persisted notifications setting (notifications S4, write side): ONE
//! additive field of version 1, written only ONCE the user turns the alert off.
//! A file of a user who never touched the setting stays byte-for-byte what older
//! builds wrote - the same rule the language preference and the chosen home
//! follow while they are unset - and an absent key still reads as the default.
//!
//! This is the crate's own test binary for the document's BYTES: the lib tests
//! pin the helpers, and a write shape that only shows up in the file
//! (`{"version":1,"theme":"dark","workspaces":[]}`) has to fail here, in the
//! binary a `cargo test -p gentle-runtime` runs but the filtered lib run does
//! not.
//!
//! Every file lives in a temporary directory outside the repository, so no
//! user's own config is read or written.

use std::fs;
use std::path::{Path, PathBuf};

use gentle_core::i18n::Language;
use gentle_runtime::app_config::{self, AppConfig, Notifications};

/// A config path inside a fresh temporary directory.
fn path_in(dir: &Path) -> PathBuf {
    dir.join("config.json")
}

/// The parsed JSON of a file, as the file really holds it.
fn document(path: &Path) -> serde_json::Value {
    let data = fs::read(path).expect("the file was written");
    serde_json::from_slice(&data).expect("the file is JSON")
}

#[test]
fn an_unset_mode_writes_exactly_the_document_older_builds_wrote() {
    let dir = tempfile::tempdir().unwrap();
    let path = path_in(dir.path());
    let config = AppConfig {
        theme: app_config::Theme::Dark,
        ..AppConfig::default()
    };

    app_config::save(&path, &config).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        r#"{"version":1,"theme":"dark","workspaces":[]}"#,
        "a user who never touched the setting gets the file older builds wrote"
    );
    assert_eq!(
        app_config::load(&path).config.notifications,
        Notifications::WhenUnfocused,
        "and the absent key still reads as the default"
    );
}

#[test]
fn off_is_written_under_the_name_the_file_stores() {
    let dir = tempfile::tempdir().unwrap();
    let path = path_in(dir.path());
    let config = AppConfig {
        notifications: Notifications::Off,
        ..AppConfig::default()
    };

    app_config::save(&path, &config).unwrap();

    assert_eq!(
        document(&path)["notifications"],
        serde_json::json!("off"),
        "the file stores the mode under its own name"
    );
    let loaded = app_config::load(&path);
    assert_eq!(loaded.notice, None, "a stored mode is not a damaged file");
    assert_eq!(loaded.config, config, "the whole document came back");
}

#[test]
fn choosing_the_default_again_takes_the_key_back_out_of_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = path_in(dir.path());
    let untouched = r#"{"version":1,"theme":"dark","workspaces":[]}"#;
    fs::write(&path, untouched).unwrap();
    let mut config = app_config::load(&path).config;

    assert!(config.set_notifications(Notifications::Off));
    app_config::save(&path, &config).unwrap();
    assert_eq!(document(&path)["notifications"], serde_json::json!("off"));

    assert!(
        config.set_notifications(Notifications::WhenUnfocused),
        "back to the default is a change, so it is saved"
    );
    app_config::save(&path, &config).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        untouched,
        "the key is gone again: back at the default the file is the older one"
    );
    assert_eq!(
        app_config::load(&path).config.notifications,
        Notifications::WhenUnfocused
    );
}

#[test]
fn every_mode_round_trips_through_the_file() {
    for mode in Notifications::ALL {
        let dir = tempfile::tempdir().unwrap();
        let path = path_in(dir.path());
        let mut config = AppConfig::default();
        assert!(config.add_workspace(r"C:\dev\p"));
        config.set_notifications(mode);

        app_config::save(&path, &config).unwrap();

        let loaded = app_config::load(&path);
        assert_eq!(loaded.notice, None, "{mode:?} is not a damaged file");
        assert_eq!(loaded.config, config, "{mode:?} did not survive the file");
        assert_eq!(loaded.config.notifications, mode);
    }
}

#[test]
fn an_unknown_mode_reads_as_the_default_and_keeps_the_rest_of_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = path_in(dir.path());
    fs::write(
        &path,
        br#"{"version":1,"theme":"Gentle","language":"es","notifications":"always","workspaces":[{"path":"C:/dev/p","collapsed":true}]}"#,
    )
    .unwrap();

    let loaded = app_config::load(&path);

    assert_eq!(loaded.notice, None, "an unknown mode is not a damaged file");
    assert_eq!(
        loaded.config.notifications,
        Notifications::WhenUnfocused,
        "a value this version does not know reads as the default"
    );
    assert_eq!(loaded.config.theme, app_config::Theme::Gentle);
    assert_eq!(loaded.config.language, Some(Language::Es));
    assert_eq!(
        loaded.config.workspaces.len(),
        1,
        "the rest of the file survives the unknown mode"
    );
    assert!(loaded.config.workspaces[0].collapsed);
    assert!(path.exists(), "the file is not moved aside");
}

#[test]
fn a_stored_mode_survives_an_unrelated_edit() {
    let dir = tempfile::tempdir().unwrap();
    let path = path_in(dir.path());
    fs::write(
        &path,
        r#"{"version":1,"theme":"dark","notifications":"off","workspaces":[{"path":"C:/dev/p","collapsed":false}]}"#,
    )
    .unwrap();

    let mut config = app_config::load(&path).config;
    assert!(config.set_theme(app_config::Theme::Light));
    app_config::save(&path, &config).unwrap();

    let reloaded = app_config::load(&path);
    assert_eq!(reloaded.notice, None, "the saved file is read, not damaged");
    assert_eq!(
        reloaded.config.notifications,
        Notifications::Off,
        "an unrelated edit kept the stored mode"
    );
    assert_eq!(reloaded.config.theme, app_config::Theme::Light);
}
