//! The app's own words THROUGH THE WHOLE CHANNEL (0.2.0, S3, D4): the config's
//! stored language reaches the folded state, and the texts the shell derives
//! from that state - the chat header, and every notice the reducer words - come
//! out in it.
//!
//! Nothing here is about the catalog's texts (`gentle-core`'s `i18n_messages`
//! pins those) or about the reducers' rules (`crates/app` pins those): this is
//! the seam between them, which is exactly what a regression in `speak` or in
//! the projection would break silently.
//!
//! The config file lives in a temporary directory, so the user's own config is
//! never read.

use std::fs;
use std::path::Path;

use gentle_app::{ChatId, Input, Intent};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::config::ConfigSnapshot;
use gentle_desk_lib::contract::LanguageView;

/// A config that stores Spanish, in the document shape this app writes.
const SPANISH: &str = r#"{"version":1,"theme":"dark","workspaces":[],"language":"es"}"#;

/// A holder over `document`, written to a config file inside `dir`.
fn holder_over(dir: &Path, document: &str) -> AppStateHolder {
    let path = dir.join("config.json");
    fs::write(&path, document).unwrap();
    AppStateHolder::with_config(ConfigSnapshot::at(Some(&path)))
}

#[test]
fn the_header_names_a_new_chat_in_the_language_the_config_holds() {
    let dir = tempfile::tempdir().unwrap();
    let holder = holder_over(dir.path(), SPANISH);

    holder.fold_input(Input::Intent(Intent::OpenNewChat));

    assert_eq!(
        holder.view().foreground_title.as_deref(),
        Some("Chat nuevo"),
        "a chat with no session yet reads the header's own words, in Spanish"
    );
}

#[test]
fn the_header_of_a_read_that_folded_nothing_speaks_the_config_language_too() {
    let dir = tempfile::tempdir().unwrap();
    let holder = holder_over(dir.path(), SPANISH);

    // No fold at all: the first paint of the window is a read, and it must
    // already carry the config's words.
    holder.fold_input(Input::Intent(Intent::OpenNewChat));
    let first = holder.view();
    assert_eq!(first.foreground_title.as_deref(), Some("Chat nuevo"));

    let read = holder.view();
    assert_eq!(
        read.foreground_title.as_deref(),
        Some("Chat nuevo"),
        "reading the slice twice says the same thing"
    );
}

#[test]
fn a_notice_folded_through_the_holder_is_worded_in_the_config_language() {
    let dir = tempfile::tempdir().unwrap();
    let holder = holder_over(dir.path(), SPANISH);
    let chat = ChatId::of_session("sessions/a.jsonl");

    holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
    let view = holder.fold_input(Input::ChatClosed {
        chat,
        expected: false,
    });

    let notice = view.notices.last().expect("a death leaves one notice");
    assert_eq!(
        notice.text,
        "a.jsonl: el proceso que ejecuta este chat terminó. Reinícialo para seguir hablando.",
        "the reducer words its own notices through the config's language"
    );
}

#[test]
fn choosing_the_language_reaches_the_notices_with_no_restart() {
    let dir = tempfile::tempdir().unwrap();
    let holder = holder_over(
        dir.path(),
        r#"{"version":1,"theme":"dark","workspaces":[]}"#,
    );
    let chat = ChatId::of_session("sessions/a.jsonl");

    // The choice the Settings dialog makes, before the chat comes up: the same
    // write path the instant switch uses (S2).
    holder.set_language(LanguageView::Es);
    holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
    let view = holder.fold_input(Input::ChatClosed {
        chat,
        expected: false,
    });

    assert_eq!(
        view.notices.last().expect("a death leaves one notice").text,
        "a.jsonl: el proceso que ejecuta este chat terminó. Reinícialo para seguir hablando.",
        "the language chosen at runtime words the next notice, with no restart"
    );
}

#[test]
fn a_run_with_no_stored_language_words_its_notices_in_english() {
    let dir = tempfile::tempdir().unwrap();
    let holder = holder_over(
        dir.path(),
        r#"{"version":1,"theme":"dark","workspaces":[]}"#,
    );
    let chat = ChatId::of_session("sessions/a.jsonl");

    holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
    let view = holder.fold_input(Input::ChatClosed {
        chat,
        expected: false,
    });

    assert_eq!(
        view.notices.last().expect("a death leaves one notice").text,
        "a.jsonl: the child that runs this chat exited. Restart to keep talking.",
        "no stored preference is English (D2: Rust never detects a locale)"
    );
}
