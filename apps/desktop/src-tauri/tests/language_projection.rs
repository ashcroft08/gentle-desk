//! The language preference as the web side sees it (D3, reading side): the
//! config's stored language projected onto the view state as `null` while it is
//! unset and as the `"en"`/`"es"` tag once it is stored - the same shape the
//! theme travels in, and the one the generated bindings declare.
//!
//! Every assertion names the tag literally and asks whether the KEY is there: a
//! missing `language` is `undefined` on the web side, not `null`, so a projection
//! that skipped the field while it is unset has to fail here.
//!
//! The config file lives in a temporary directory, so the user's own config is
//! never read. Nothing here writes a preference: the write path is not part of
//! this slice.

use std::fs;
use std::path::Path;

use serde_json::{json, Value};

use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::config::ConfigSnapshot;
use gentle_desk_lib::contract::{LanguageView, ThemeView, ViewState};

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
