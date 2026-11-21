use super::*;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const DAMAGED: &[u8] = b"{ not json";
const DAMAGED_AGAIN: &[u8] = b"still not json";

fn config_file(dir: &Path) -> PathBuf {
    dir.join("gentle-desk").join("config.json")
}

/// The names in `dir`, sorted, so a test can say exactly what is left behind.
fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn write_damaged(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

#[test]
fn a_missing_file_is_an_empty_config_without_a_notice() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());

    let loaded = load(&path);

    assert_eq!(loaded.config, AppConfig::default());
    assert_eq!(loaded.notice, None);
    assert!(names(dir.path()).is_empty(), "nothing is written by a load");
}

#[test]
fn save_then_load_round_trips_the_workspaces_and_the_collapse_state() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    let mut config = AppConfig::default();
    config.add_workspace(r"C:\dev\p");
    config.add_workspace(r"C:\dev\q");
    config.set_collapsed(r"C:\dev\q", true);

    save(&path, &config).unwrap();

    let loaded = load(&path);
    assert_eq!(loaded.config, config);
    assert_eq!(loaded.notice, None);
}

#[test]
fn the_saved_document_carries_the_top_level_version_and_the_workspaces() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    let mut config = AppConfig::default();
    config.add_workspace(r"C:\dev\p");
    config.set_collapsed(r"C:\dev\p", true);

    save(&path, &config).unwrap();

    let document: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(document["version"], serde_json::json!(CONFIG_VERSION));
    assert_eq!(
        document["workspaces"],
        serde_json::json!([{ "path": r"C:\dev\p", "collapsed": true }])
    );
}

#[test]
fn save_creates_the_directory_and_leaves_no_temp_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());

    save(&path, &AppConfig::default()).unwrap();

    assert_eq!(names(path.parent().unwrap()), ["config.json"]);
    assert_eq!(load(&path).config, AppConfig::default());
}

#[test]
fn a_failed_rename_leaves_the_previous_entry_and_no_temp_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    // A directory cannot be replaced by a file rename, so the last step of the
    // write fails with the old entry untouched.
    fs::create_dir_all(&path).unwrap();
    fs::write(path.join("keep.txt"), b"old").unwrap();

    assert!(save(&path, &AppConfig::default()).is_err());

    assert_eq!(fs::read(path.join("keep.txt")).unwrap(), b"old");
    assert_eq!(names(path.parent().unwrap()), ["config.json"]);
}

#[test]
fn a_save_that_cannot_write_keeps_the_previous_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    let mut first = AppConfig::default();
    first.add_workspace(r"C:\dev\p");
    save(&path, &first).unwrap();
    // A directory where the temporary file goes makes the write fail.
    fs::create_dir_all(path.with_file_name("config.json.tmp")).unwrap();
    let mut second = AppConfig::default();
    second.add_workspace(r"C:\dev\q");

    assert!(save(&path, &second).is_err());

    assert_eq!(load(&path).config, first);
}

#[test]
fn a_damaged_file_is_kept_aside_with_its_bytes_and_an_empty_config_comes_back() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    write_damaged(&path, DAMAGED);

    let loaded = load(&path);

    assert_eq!(loaded.config, AppConfig::default());
    let notice = loaded.notice.unwrap();
    assert_eq!(notice.problem, ConfigProblem::Damaged);
    let aside = notice.aside.clone().unwrap();
    assert_ne!(aside, path);
    assert!(!path.exists(), "the damaged file is not left in place");
    assert_eq!(fs::read(&aside).unwrap(), DAMAGED);
    let name = aside.file_name().unwrap().to_string_lossy().into_owned();
    assert!(name.starts_with("config.json.corrupt-"), "{name}");

    let message = notice.message();
    assert!(message.is_ascii(), "{message}");
    assert!(message.contains(&name), "{message}");
}

#[test]
fn a_second_damaged_file_does_not_overwrite_the_first_aside_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    write_damaged(&path, DAMAGED);
    let first = load(&path).notice.unwrap().aside.unwrap();
    write_damaged(&path, DAMAGED_AGAIN);

    let second = load(&path).notice.unwrap().aside.unwrap();

    assert_ne!(first, second);
    assert_eq!(fs::read(&first).unwrap(), DAMAGED);
    assert_eq!(fs::read(&second).unwrap(), DAMAGED_AGAIN);
}

#[test]
fn the_aside_name_counts_up_when_the_second_is_already_taken() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    // Take the plain aside names of the next few seconds, so whichever second
    // the load runs in, the plain name is taken and the name must count up.
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let earlier: Vec<PathBuf> = (0..5)
        .map(|offset| path.with_file_name(format!("config.json.corrupt-{}", seconds + offset)))
        .collect();
    for aside in &earlier {
        fs::write(aside, b"earlier").unwrap();
    }
    write_damaged(&path, DAMAGED);

    let aside = load(&path).notice.unwrap().aside.unwrap();

    let name = aside.file_name().unwrap().to_string_lossy().into_owned();
    assert!(name.starts_with("config.json.corrupt-"), "{name}");
    assert!(name.ends_with("-2"), "{name}");
    assert_eq!(fs::read(&aside).unwrap(), DAMAGED);
    for untouched in &earlier {
        assert_eq!(fs::read(untouched).unwrap(), b"earlier", "{untouched:?}");
    }
}

#[test]
fn a_file_of_a_version_this_app_does_not_write_is_kept_aside() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    let document = br#"{"version":2,"workspaces":[]}"#;
    write_damaged(&path, document);

    let loaded = load(&path);

    assert_eq!(loaded.config, AppConfig::default());
    let notice = loaded.notice.unwrap();
    assert_eq!(notice.problem, ConfigProblem::UnknownVersion { found: 2 });
    let message = notice.message();
    let aside = notice.aside.unwrap();
    assert!(!path.exists());
    assert_eq!(fs::read(&aside).unwrap(), document);
    assert!(message.contains("2"), "{message}");
    assert!(message.is_ascii(), "{message}");
}

/// A v1 document as the version BEFORE the theme field wrote it.
const WITHOUT_THEME: &[u8] =
    br#"{"version":1,"workspaces":[{"path":"C:/dev/p","collapsed":true}]}"#;

#[test]
fn a_config_without_the_theme_field_loads_with_the_system_theme() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    write_damaged(&path, WITHOUT_THEME);

    let loaded = load(&path);

    assert_eq!(loaded.notice, None, "an older file is still this document");
    assert_eq!(loaded.config.theme, crate::app_config::Theme::System);
    assert_eq!(
        loaded.config.workspaces.len(),
        1,
        "the workspaces of the file are kept"
    );
    assert!(loaded.config.workspaces[0].collapsed);
}

#[test]
fn every_theme_round_trips_through_the_file() {
    for theme in crate::app_config::Theme::ALL {
        let dir = tempfile::tempdir().unwrap();
        let path = config_file(dir.path());
        let mut config = AppConfig::default();
        assert!(config.add_workspace(r"C:\dev\p"));
        config.set_theme(theme);

        save(&path, &config).unwrap();

        let loaded = load(&path);
        assert_eq!(loaded.notice, None);
        assert_eq!(loaded.config, config, "{theme:?} did not survive the file");
        assert_eq!(loaded.config.theme, theme);
    }
}

#[test]
fn an_unknown_theme_loads_as_the_system_theme_and_keeps_the_workspaces() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    let document =
        br#"{"version":1,"theme":"Neon","workspaces":[{"path":"C:/dev/p","collapsed":false}]}"#;
    write_damaged(&path, document);

    let loaded = load(&path);

    assert_eq!(
        loaded.notice, None,
        "an unknown theme is not a damaged file"
    );
    assert_eq!(loaded.config.theme, crate::app_config::Theme::System);
    assert_eq!(
        loaded.config.workspaces.len(),
        1,
        "the workspaces are not discarded with the unknown theme"
    );
    assert!(path.exists(), "the file is not moved aside");
}

#[test]
fn the_saved_document_carries_the_theme_as_the_stylesheet_names_it() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    let mut config = AppConfig::default();
    config.set_theme(crate::app_config::Theme::GentlemanCute);

    save(&path, &config).unwrap();

    let document: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(document["theme"], serde_json::json!("Gentleman-Cute"));
    assert_eq!(document["version"], serde_json::json!(CONFIG_VERSION));
}

/// A v1 document written BEFORE the shell home choice existed.
#[test]
fn a_config_without_the_shell_home_field_loads_with_no_choice() {
    let data =
        br#"{"version":1,"theme":"dark","workspaces":[{"path":"C:/dev/p","collapsed":false}]}"#;
    let loaded = super::document::parse(data).expect("an older v1 document loads");

    assert_eq!(loaded.shell_home, None);
    assert_eq!(loaded.workspaces.len(), 1);
}

#[test]
fn a_shell_home_choice_round_trips_and_is_not_written_while_unset() {
    for choice in [
        gentle_backend::home::HomeChoice::Link,
        gentle_backend::home::HomeChoice::Isolated,
    ] {
        let mut config = crate::app_config::AppConfig::default();
        assert!(config.set_shell_home(choice));
        let bytes = super::document::bytes(&config).expect("serialises");
        let back = super::document::parse(&bytes).expect("parses");
        assert_eq!(back.shell_home, Some(choice));
    }
    let bytes = super::document::bytes(&crate::app_config::AppConfig::default()).unwrap();
    assert!(!String::from_utf8(bytes).unwrap().contains("shell_home"));
}

#[test]
fn an_unknown_shell_home_loads_as_no_choice_and_keeps_the_workspaces() {
    let data = br#"{"version":1,"shell_home":"cloud","workspaces":[{"path":"C:/dev/p","collapsed":false}]}"#;
    let loaded = super::document::parse(data).expect("an unknown choice is not a damaged file");

    assert_eq!(loaded.shell_home, None);
    assert_eq!(loaded.workspaces.len(), 1);
}

#[test]
fn choosing_the_same_shell_home_again_changes_nothing() {
    let mut config = crate::app_config::AppConfig::default();
    assert!(config.set_shell_home(gentle_backend::home::HomeChoice::Link));
    assert!(!config.set_shell_home(gentle_backend::home::HomeChoice::Link));
    assert!(config.set_shell_home(gentle_backend::home::HomeChoice::Isolated));
}

/// A v1 document as the version BEFORE the notifications setting wrote it.
const WITHOUT_NOTIFICATIONS: &[u8] = br#"{"version":1,"theme":"dark","language":"es","workspaces":[{"path":"C:/dev/p","collapsed":true}]}"#;

#[test]
fn a_config_without_the_notifications_field_falls_back_to_the_default() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    write_damaged(&path, WITHOUT_NOTIFICATIONS);

    let loaded = load(&path);

    assert_eq!(loaded.notice, None, "an older file is still this document");
    assert_eq!(
        loaded.config.notifications,
        crate::app_config::Notifications::WhenUnfocused
    );
    assert_eq!(loaded.config.theme, crate::app_config::Theme::Dark);
    assert_eq!(
        loaded.config.language,
        Some(gentle_core::i18n::Language::Es)
    );
    assert_eq!(
        loaded.config.workspaces.len(),
        1,
        "the workspaces of the file are kept"
    );
    assert!(loaded.config.workspaces[0].collapsed);
}

#[test]
fn every_notifications_mode_round_trips_through_the_file() {
    for mode in crate::app_config::Notifications::ALL {
        let dir = tempfile::tempdir().unwrap();
        let path = config_file(dir.path());
        let mut config = AppConfig::default();
        assert!(config.add_workspace(r"C:\dev\p"));
        // The default is not written at all; what matters is that it reads back.
        config.set_notifications(mode);

        save(&path, &config).unwrap();

        let loaded = load(&path);
        assert_eq!(loaded.notice, None);
        assert_eq!(loaded.config, config, "{mode:?} did not survive the file");
        assert_eq!(loaded.config.notifications, mode);
    }
}

#[test]
fn an_unknown_notifications_value_loads_as_the_default_and_keeps_the_rest_of_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    let document = br#"{"version":1,"theme":"Gentle","language":"es","shell_home":"isolated","notifications":"always","workspaces":[{"path":"C:/dev/p","collapsed":false}]}"#;
    write_damaged(&path, document);

    let loaded = load(&path);

    assert_eq!(loaded.notice, None, "an unknown mode is not a damaged file");
    assert_eq!(
        loaded.config.notifications,
        crate::app_config::Notifications::WhenUnfocused,
        "a value this version does not know reads as the default"
    );
    assert_eq!(loaded.config.theme, crate::app_config::Theme::Gentle);
    assert_eq!(
        loaded.config.language,
        Some(gentle_core::i18n::Language::Es)
    );
    assert_eq!(
        loaded.config.shell_home,
        Some(gentle_backend::home::HomeChoice::Isolated)
    );
    assert_eq!(
        loaded.config.workspaces.len(),
        1,
        "the rest of the file survives the unknown mode"
    );
    assert!(path.exists(), "the file is not moved aside");
}

#[test]
fn the_saved_document_carries_the_notifications_mode_the_setting_names() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    let mut config = AppConfig::default();
    assert!(config.set_notifications(crate::app_config::Notifications::Off));

    save(&path, &config).unwrap();

    let document: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(document["notifications"], serde_json::json!("off"));
    assert_eq!(document["version"], serde_json::json!(CONFIG_VERSION));
}

#[test]
fn the_default_notifications_mode_is_not_written_at_all() {
    // A file of a user who never touched the setting stays byte-for-byte what
    // older builds wrote: the key only appears once the alert is turned off.
    let config = AppConfig {
        theme: crate::app_config::Theme::Dark,
        ..AppConfig::default()
    };

    let bytes = super::document::bytes(&config).expect("serialises");

    assert_eq!(
        String::from_utf8(bytes).unwrap(),
        r#"{"version":1,"theme":"dark","workspaces":[]}"#,
        "nothing of this setting reaches a file at its default"
    );
}

#[test]
fn choosing_the_default_again_takes_the_notifications_key_out_of_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    write_damaged(&path, WITHOUT_NOTIFICATIONS);
    let mut config = load(&path).config;

    assert!(config.set_notifications(crate::app_config::Notifications::Off));
    save(&path, &config).unwrap();
    let written = fs::read(&path).unwrap();
    assert!(
        String::from_utf8(written)
            .unwrap()
            .contains(r#""notifications":"off""#),
        "the mode off is in the file"
    );

    assert!(
        config.set_notifications(crate::app_config::Notifications::WhenUnfocused),
        "back to the default is a change, so it is saved"
    );
    save(&path, &config).unwrap();

    // The rewrite is the document the user's own file was before the key
    // existed, plus nothing: the key is gone again, not stored as the default.
    let after: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert!(
        after.get("notifications").is_none(),
        "the default is not written: {after}"
    );
    assert_eq!(after["theme"], serde_json::json!("dark"));
    assert_eq!(after["language"], serde_json::json!("es"));
    assert_eq!(after["workspaces"][0]["collapsed"], serde_json::json!(true));
    assert_eq!(
        load(&path).config.notifications,
        crate::app_config::Notifications::WhenUnfocused
    );
}

#[test]
fn a_rewrite_for_the_notifications_field_keeps_every_other_setting() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    write_damaged(&path, WITHOUT_NOTIFICATIONS);
    let mut loaded = load(&path).config;

    assert!(loaded.set_notifications(crate::app_config::Notifications::Off));
    save(&path, &loaded).unwrap();

    // What another key of the file held is still there after the rewrite, and
    // the new mode is beside it, not instead of it.
    let back = load(&path);
    assert_eq!(back.notice, None);
    assert_eq!(
        back.config.notifications,
        crate::app_config::Notifications::Off
    );
    assert_eq!(back.config.theme, crate::app_config::Theme::Dark);
    assert_eq!(back.config.language, Some(gentle_core::i18n::Language::Es));
    assert_eq!(back.config.workspaces.len(), 1);
    assert!(back.config.workspaces[0].collapsed);
}

/// A v1 document as the version BEFORE "skip this version" wrote it.
const WITHOUT_SKIP: &[u8] = br#"{"version":1,"theme":"dark","language":"es","notifications":"off","shell_home":"isolated","workspaces":[{"path":"C:/dev/p","collapsed":true}]}"#;

#[test]
fn a_config_without_the_skipped_field_skips_no_version() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    write_damaged(&path, WITHOUT_SKIP);

    let loaded = load(&path);

    assert_eq!(loaded.notice, None, "an older file is still this document");
    assert_eq!(
        loaded.config.skipped_update, None,
        "nothing skipped is what a file written before the field says"
    );
    assert_eq!(loaded.config.theme, crate::app_config::Theme::Dark);
    assert_eq!(
        loaded.config.workspaces.len(),
        1,
        "the rest of the file is kept"
    );
}

#[test]
fn the_skipped_version_survives_the_file() {
    // S1: the skip is remembered in the app config, so a restart keeps it.
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    let mut config = AppConfig::default();
    assert!(config.add_workspace(r"C:\dev\p"));
    assert!(config.set_skipped_update("2.0.5"));

    save(&path, &config).unwrap();

    let loaded = load(&path);
    assert_eq!(loaded.notice, None);
    assert_eq!(loaded.config, config, "the skip did not survive the file");
    assert_eq!(loaded.config.skipped_update.as_deref(), Some("2.0.5"));
}

#[test]
fn the_saved_document_carries_the_skipped_version_and_omits_it_while_unset() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    let mut config = AppConfig::default();
    save(&path, &config).unwrap();
    let document: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        document.get("skipped_update"),
        None,
        "a run that skipped nothing writes the document older builds wrote"
    );

    assert!(config.set_skipped_update("2.0.5"));
    save(&path, &config).unwrap();
    let document: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(document["skipped_update"], serde_json::json!("2.0.5"));
    assert_eq!(document["version"], serde_json::json!(CONFIG_VERSION));
}

#[test]
fn a_mistyped_or_unparsable_skipped_value_reads_as_no_skip_and_keeps_the_rest_of_the_file() {
    for value in [
        serde_json::json!(2),
        serde_json::json!(true),
        serde_json::json!(["2.0.5"]),
        serde_json::json!({"version": "2.0.5"}),
        serde_json::json!(""),
        serde_json::json!("v2.0.5"),
        serde_json::json!("2.0.5-beta.1"),
        serde_json::json!("latest"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = config_file(dir.path());
        let document = format!(
            r#"{{"version":1,"theme":"Gentle","notifications":"off","skipped_update":{},"workspaces":[{{"path":"C:/dev/p","collapsed":true}}]}}"#,
            value
        );
        write_damaged(&path, document.as_bytes());

        let loaded = load(&path);

        assert_eq!(
            loaded.notice, None,
            "{value} is not a damaged file: the skip is one field of it"
        );
        assert_eq!(
            loaded.config.skipped_update, None,
            "{value} is not a version this app compares, so it skips nothing"
        );
        assert_eq!(loaded.config.theme, crate::app_config::Theme::Gentle);
        assert_eq!(
            loaded.config.notifications,
            crate::app_config::Notifications::Off
        );
        assert_eq!(loaded.config.workspaces.len(), 1);
        assert!(loaded.config.workspaces[0].collapsed);
        assert!(path.exists(), "the file is not moved aside");
    }
}

#[test]
fn a_rewrite_for_the_skip_keeps_every_other_setting() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    write_damaged(&path, WITHOUT_SKIP);
    let mut loaded = load(&path).config;

    assert!(loaded.set_skipped_update("2.0.5"));
    save(&path, &loaded).unwrap();

    let back = load(&path);
    assert_eq!(back.notice, None);
    assert_eq!(back.config.skipped_update.as_deref(), Some("2.0.5"));
    assert_eq!(back.config.theme, crate::app_config::Theme::Dark);
    assert_eq!(back.config.language, Some(gentle_core::i18n::Language::Es));
    assert_eq!(
        back.config.notifications,
        crate::app_config::Notifications::Off
    );
    assert_eq!(
        back.config.shell_home,
        Some(gentle_backend::home::HomeChoice::Isolated)
    );
    assert_eq!(back.config.workspaces.len(), 1);
    assert!(back.config.workspaces[0].collapsed);
}
