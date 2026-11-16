//! The language preference as the web side sees it (D3): the config's stored
//! language projected onto the view state as `null` while it is unset and as the
//! `"en"`/`"es"` tag once it is stored - the same shape the theme travels in, and
//! the one the generated bindings declare - and the write path that puts it
//! there, so a choice re-emits the very slice the web side draws (S2).
//!
//! Every assertion names the tag literally and asks whether the KEY is there: a
//! missing `language` is `undefined` on the web side, not `null`, so a projection
//! that skipped the field while it is unset has to fail here.
//!
//! The config file lives in a temporary directory, so the user's own config is
//! never read.

use std::fs;
use std::path::Path;

use serde_json::{json, Value};

use gentle_core::i18n::Language;
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::config::{ConfigSnapshot, ConfigWriter};
use gentle_desk_lib::contract::{LanguageView, ThemeView, UiIntent, ViewState};
use gentle_runtime::app_config::{self, AppConfig};

/// The document every release before this feature wrote: no language key.
const WITHOUT_A_LANGUAGE: &str = r#"{"version":1,"theme":"dark","workspaces":[]}"#;

/// A holder over `document`, written to a config file inside `dir`.
fn holder_over(dir: &Path, document: &str) -> AppStateHolder {
    let path = dir.join("config.json");
    fs::write(&path, document).unwrap();
    AppStateHolder::with_config(ConfigSnapshot::at(Some(&path)))
}

/// The view state as it reaches the web side, as JSON.
fn wire(view: &ViewState) -> Value {
    serde_json::to_value(view).expect("the view state serializes")
}

/// What the wire carries under `language`: `Some` for the key, `None` when the
/// field was not sent at all - which is `undefined` on the web side, and not
/// what the bindings declare.
fn wired_language(view: &ViewState) -> Option<Value> {
    wire(view).get("language").cloned()
}

#[test]
fn a_config_with_no_stored_language_projects_null() {
    let dir = tempfile::tempdir().unwrap();
    let holder = holder_over(dir.path(), WITHOUT_A_LANGUAGE);

    let view = holder.view();

    assert_eq!(view.language, None, "nothing was chosen yet");
    assert_eq!(
        wired_language(&view),
        Some(Value::Null),
        "the key is there, carrying null: the bindings declare it required"
    );
    assert_eq!(view.theme, ThemeView::Dark, "the theme is untouched");
}

#[test]
fn a_stored_english_preference_projects_en() {
    let dir = tempfile::tempdir().unwrap();
    let document = r#"{"version":1,"theme":"system","language":"en","workspaces":[]}"#;
    let holder = holder_over(dir.path(), document);

    let view = holder.view();

    assert_eq!(view.language, Some(LanguageView::En));
    assert_eq!(wired_language(&view), Some(json!("en")));
}

#[test]
fn a_stored_spanish_preference_projects_es() {
    let dir = tempfile::tempdir().unwrap();
    let document = r#"{"version":1,"theme":"system","language":"es","workspaces":[]}"#;
    let holder = holder_over(dir.path(), document);

    let view = holder.view();

    assert_eq!(view.language, Some(LanguageView::Es));
    assert_eq!(wired_language(&view), Some(json!("es")));
}

#[test]
fn the_stored_tag_reads_case_insensitively_and_without_its_spaces() {
    // Each row names the variant and the tag literally: a projection that
    // swapped the two languages could not pass this, however it wrote them.
    for (document, expected, tag) in [
        (
            r#"{"version":1,"theme":"system","language":"ES","workspaces":[]}"#,
            LanguageView::Es,
            "es",
        ),
        (
            r#"{"version":1,"theme":"system","language":" es ","workspaces":[]}"#,
            LanguageView::Es,
            "es",
        ),
        (
            r#"{"version":1,"theme":"system","language":"EN","workspaces":[]}"#,
            LanguageView::En,
            "en",
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let holder = holder_over(dir.path(), document);

        let view = holder.view();

        assert_eq!(view.language, Some(expected), "{document}");
        assert_eq!(
            wired_language(&view),
            Some(json!(tag)),
            "{document} reaches the web as its canonical tag"
        );
    }
}

#[test]
fn a_stored_value_that_is_not_a_tag_projects_null_and_the_run_still_boots() {
    for document in [
        r#"{"version":1,"theme":"Gentle","language":"xx","workspaces":[]}"#,
        r#"{"version":1,"theme":"Gentle","language":"","workspaces":[]}"#,
        r#"{"version":1,"theme":"Gentle","language":42,"workspaces":[]}"#,
    ] {
        let dir = tempfile::tempdir().unwrap();
        let holder = holder_over(dir.path(), document);

        let view = holder.view();

        assert_eq!(view.language, None, "{document} stores no language");
        assert_eq!(
            wired_language(&view),
            Some(Value::Null),
            "{document} reaches the web as null, under a key that is there"
        );
        assert_eq!(
            view.theme,
            ThemeView::Gentle,
            "{document} is read, not treated as a damaged config"
        );
        assert_eq!(
            view.config_notice, None,
            "{document} gives the user no config notice"
        );
    }
}

// The WRITE path of the same preference (D3, S2/S4): choosing a language is one
// atomic config edit, saved by the one writer and re-emitted as the very slice
// the web side draws, so the switch needs no restart and no webview reload.
//
// A save runs on the writer's own thread, so a test that reads the file back
// waits for it, bounded.

/// Every wait of the write tests: a save of one small file, never a hang.
const WAIT: std::time::Duration = std::time::Duration::from_secs(10);

/// How long a file is watched after a change that should not have touched it.
const SETTLE: std::time::Duration = std::time::Duration::from_millis(100);

/// A holder over the config file at `path`, with the one writer armed.
fn holder_at(path: &Path) -> AppStateHolder {
    let holder = AppStateHolder::with_config(ConfigSnapshot::at(Some(path)));
    holder.arm_config_saves(ConfigWriter::spawn(Some(path.to_path_buf())));
    holder
}

/// Waits for what a bounded wait is about, failing the test instead of hanging.
fn wait_for<T>(what: &str, mut probe: impl FnMut() -> Option<T>) -> T {
    let deadline = std::time::Instant::now() + WAIT;
    loop {
        if let Some(value) = probe() {
            return value;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "waited {WAIT:?} for {what} and it never came"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[test]
fn a_language_intent_stores_the_choice_and_re_emits_it() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    fs::write(&path, WITHOUT_A_LANGUAGE).unwrap();
    let holder = holder_at(&path);

    let pushed = holder.dispatch(UiIntent::SetLanguage {
        language: LanguageView::Es,
    });

    // The slice the command returns AND the next push carries: the same one the
    // web side draws, so the words change in this tick.
    assert_eq!(pushed.language, Some(LanguageView::Es));
    assert_eq!(wired_language(&pushed), Some(json!("es")));
    assert_eq!(wired_language(&holder.view()), Some(json!("es")));

    let loaded = wait_for("the language to reach the config file", || {
        let loaded = app_config::load(&path);
        (loaded.config.language == Some(Language::Es)).then_some(loaded)
    });
    assert_eq!(loaded.notice, None, "the file was written, not damaged");
    assert_eq!(loaded.config.language, Some(Language::Es));
}

#[test]
fn choosing_the_same_language_again_saves_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let mut saved = AppConfig::default();
    assert!(saved.set_language(Language::Es));
    app_config::save(&path, &saved).unwrap();
    let holder = holder_at(&path);

    let before = holder.view();
    let written = fs::read(&path).unwrap();
    let after = holder.dispatch(UiIntent::SetLanguage {
        language: LanguageView::Es,
    });
    std::thread::sleep(SETTLE);

    assert_eq!(
        after, before,
        "the same language changes no slice of the state"
    );
    assert_eq!(
        fs::read(&path).unwrap(),
        written,
        "and asks for no save at all"
    );
}

#[test]
fn the_chosen_language_survives_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    fs::write(&path, WITHOUT_A_LANGUAGE).unwrap();
    let holder = holder_at(&path);

    holder.dispatch(UiIntent::SetLanguage {
        language: LanguageView::Es,
    });
    wait_for("the language to reach the config file", || {
        (app_config::load(&path).config.language == Some(Language::Es)).then_some(())
    });

    // The next start: the same file, read before the holder exists.
    let next = AppStateHolder::with_config(ConfigSnapshot::at(Some(&path)));

    assert_eq!(next.view().language, Some(LanguageView::Es));
}
