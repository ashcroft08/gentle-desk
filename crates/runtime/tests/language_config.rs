//! The stored language preference (S4, reading side): an ADDITIVE, optional
//! field of version 1 that never breaks a boot and never appears in the file
//! until the user chooses one.
//!
//! What is pinned here is the READ rule, not the write path: a stored value that
//! is not `"en"`/`"es"` reads as "no preference yet" instead of turning the whole
//! document into a damaged one, so the workspaces under it survive; an unset
//! preference is not written at all; and a load never writes, not even to
//! normalise what it just read.
//!
//! Every file lives in a temporary directory outside the repository, so no
//! user's own config is read or written.

use std::fs;
use std::path::{Path, PathBuf};

use gentle_core::i18n::Language;
use gentle_runtime::app_config::{self, AppConfig, CONFIG_VERSION};

/// A config path inside a fresh temporary directory.
fn path_in(dir: &Path) -> PathBuf {
    dir.join("config.json")
}

/// The parsed JSON of a file, as the file really holds it.
fn document(path: &Path) -> serde_json::Value {
    let data = fs::read(path).expect("the file was written");
    serde_json::from_slice(&data).expect("the file is JSON")
}

/// The one workspace/title-bearing document a test writes by hand.
fn document_with_language(value: &str) -> String {
    format!(
        r#"{{"version":1,"theme":"dark","language":{value},"workspaces":[{{"path":"C:/dev/p","collapsed":true}}]}}"#
    )
}

#[test]
fn a_chosen_language_round_trips_through_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = path_in(dir.path());
    let config = AppConfig {
        language: Some(Language::Es),
        ..AppConfig::default()
    };

    app_config::save(&path, &config).unwrap();
    let loaded = app_config::load(&path);

    assert_eq!(loaded.notice, None, "a language is not a damaged file");
    assert_eq!(loaded.config.language, Some(Language::Es));
    assert_eq!(loaded.config, config, "the whole document came back");
}

#[test]
fn a_config_without_the_field_loads_with_no_language() {
    // The file every release before this feature wrote.
    let dir = tempfile::tempdir().unwrap();
    let path = path_in(dir.path());
    let data =
        br#"{"version":1,"theme":"dark","workspaces":[{"path":"C:/dev/p","collapsed":false}]}"#;
    fs::write(&path, data).unwrap();

    let loaded = app_config::load(&path);

    assert_eq!(loaded.notice, None);
    assert_eq!(loaded.config.language, None, "nothing was chosen yet");
    assert_eq!(loaded.config.version, CONFIG_VERSION);
    assert_eq!(loaded.config.workspaces.len(), 1, "the workspaces survive");
}

#[test]
fn the_stored_tag_reads_case_insensitively_and_without_its_spaces() {
    for (stored, expected) in [
        (r#""es""#, Language::Es),
        (r#""ES""#, Language::Es),
        (r#"" es ""#, Language::Es),
        (r#""en""#, Language::En),
        (r#""EN""#, Language::En),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = path_in(dir.path());
        fs::write(&path, document_with_language(stored)).unwrap();

        let loaded = app_config::load(&path);

        assert_eq!(loaded.notice, None, "{stored} is a value, not damage");
        assert_eq!(loaded.config.language, Some(expected), "{stored}");
        assert!(
            loaded.config.workspaces[0].collapsed,
            "the document around {stored} survives"
        );
    }
}

#[test]
fn a_value_that_is_not_a_tag_reads_as_no_language() {
    // Not the stored tags: another language, nothing at all, a number, an
    // object, a list, a flag. Each is "no preference yet", never a damaged
    // file, so a boot never loses the workspaces over it.
    for stored in [
        r#""xx""#,
        r#""""#,
        r#""es-ES""#,
        r#""e""#,
        "42",
        r#"{"tag":"es"}"#,
        r#"["es"]"#,
        "true",
        "null",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = path_in(dir.path());
        fs::write(&path, document_with_language(stored)).unwrap();

        let loaded = app_config::load(&path);

        assert_eq!(
            loaded.notice, None,
            "{stored} must not turn the file into a damaged one"
        );
        assert_eq!(loaded.config.language, None, "{stored} stores no language");
        assert_eq!(
            loaded.config.theme,
            app_config::Theme::Dark,
            "the theme survives {stored}"
        );
        assert!(
            loaded.config.workspaces[0].collapsed,
            "the workspaces survive {stored}"
        );
    }
}

#[test]
fn an_unset_preference_is_never_written_into_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = path_in(dir.path());

    app_config::save(&path, &AppConfig::default()).unwrap();

    let document = document(&path);
    assert_eq!(
        document.get("language"),
        None,
        "an unset preference leaves no key at all"
    );
    assert_eq!(document["version"], CONFIG_VERSION);
    assert_eq!(document["theme"], "system");
}

#[test]
fn a_chosen_language_is_written_as_its_tag() {
    let dir = tempfile::tempdir().unwrap();
    let path = path_in(dir.path());
    let config = AppConfig {
        language: Some(Language::En),
        ..AppConfig::default()
    };

    app_config::save(&path, &config).unwrap();

    assert_eq!(
        document(&path)["language"],
        "en",
        "the stored tag is the canonical one"
    );
}

#[test]
fn an_unrelated_edit_keeps_the_stored_language() {
    // The ordinary path of the app: a document with a stored language is read,
    // edited through the config's own helpers, and written back. The choice is
    // the user's, so a theme change or a fold must not lose it.
    let dir = tempfile::tempdir().unwrap();
    let path = path_in(dir.path());
    fs::write(&path, document_with_language(r#""es""#)).unwrap();

    let mut config = app_config::load(&path).config;
    assert!(config.set_theme(app_config::Theme::Light));
    assert!(
        config.set_collapsed("C:/dev/p", false),
        "the document names the workspace it folds"
    );
    app_config::save(&path, &config).unwrap();

    let reloaded = app_config::load(&path);
    assert_eq!(reloaded.notice, None, "the saved file is read, not damaged");
    assert_eq!(
        reloaded.config.language,
        Some(Language::Es),
        "an unrelated edit kept the stored language"
    );
    assert_eq!(reloaded.config.theme, app_config::Theme::Light);
    assert!(
        !reloaded.config.workspaces[0].collapsed,
        "and the fold reached the file too"
    );
}

#[test]
fn an_unset_preference_writes_exactly_the_document_older_builds_wrote() {
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
        "nothing of this feature reaches a file whose preference is unset"
    );
}

#[test]
fn a_load_writes_nothing_not_even_the_value_it_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = path_in(dir.path());
    // `"ES"` and `"xx"` both read as a preference, so neither is a damaged file:
    // the load normalises nothing and keeps the file exactly as it found it.
    for stored in [r#""es""#, r#""ES""#, r#""xx""#] {
        let data = document_with_language(stored);
        fs::write(&path, &data).unwrap();

        let loaded = app_config::load(&path);

        assert_eq!(loaded.notice, None, "{stored}");
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            data,
            "the load left {stored} exactly as it found it"
        );
        assert_eq!(
            fs::read_dir(dir.path()).unwrap().count(),
            1,
            "no aside file was created for {stored}"
        );
    }
}

#[test]
fn setting_a_language_from_none_stores_it_and_answers_true() {
    let mut config = AppConfig::default();
    assert_eq!(config.language, None, "nothing is stored yet");

    assert!(
        config.set_language(Language::Es),
        "the first choice changed the document"
    );
    assert_eq!(config.language, Some(Language::Es));
}

#[test]
fn setting_the_same_language_again_answers_false_and_changes_nothing() {
    let mut config = AppConfig::default();
    assert!(config.set_language(Language::Es));

    assert!(
        !config.set_language(Language::Es),
        "the same choice again changes nothing, so nothing is saved"
    );
    assert_eq!(config.language, Some(Language::Es));

    // The other language IS a change, and one choice never keeps the other.
    assert!(config.set_language(Language::En));
    assert_eq!(config.language, Some(Language::En));
}

#[test]
fn a_language_set_through_the_helper_survives_a_save_and_reload() {
    let dir = tempfile::tempdir().unwrap();
    let path = path_in(dir.path());
    let mut config = AppConfig::default();

    assert!(config.set_language(Language::Es));
    app_config::save(&path, &config).unwrap();
    let loaded = app_config::load(&path);

    assert_eq!(loaded.notice, None, "the written file is read, not damaged");
    assert_eq!(loaded.config.language, Some(Language::Es));
    assert_eq!(loaded.config, config, "the whole document came back");
}
