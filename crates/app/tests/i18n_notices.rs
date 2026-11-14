//! The app's own notices AS THE REDUCER PRODUCES THEM (0.2.0, S3, D6): the
//! language is the shell's, it reaches the reducer with the state it folds, and
//! every notice the reducer words itself is written through it.
//!
//! What is proven here is the wiring and nothing about the catalog's texts,
//! which `gentle-core`'s `i18n_messages` already pins: a `None` language is
//! English (D2: nothing in `crates/app` detects a locale), a stored `es` words
//! the notice in Spanish, and a fold that rebuilds the state keeps the language
//! it was given.

use gentle_app::{close_working, update_chats, AppState, ChatId, Input, Intent, NoticeLevel};
use gentle_core::i18n::Language;

/// The state of a run that speaks `language`.
fn speaking(language: Language) -> AppState {
    let mut state = AppState::default();
    state.language = Some(language);
    state
}

#[test]
fn a_death_notice_is_worded_in_the_language_the_state_speaks() {
    let chat = ChatId::of_session("sessions/a.jsonl");

    let (spanish, _) = update_chats(
        speaking(Language::Es),
        Input::ChatClosed {
            chat: chat.clone(),
            expected: false,
        },
    );
    let notice = spanish.notices.last().expect("a death leaves one notice");
    assert_eq!(notice.level, NoticeLevel::Error);
    assert_eq!(
        notice.text,
        "a.jsonl: el proceso que ejecuta este chat terminó. Reinícialo para seguir hablando."
    );

    let (english, _) = update_chats(
        speaking(Language::En),
        Input::ChatClosed {
            chat,
            expected: false,
        },
    );
    assert_eq!(
        english
            .notices
            .last()
            .expect("a death leaves one notice")
            .text,
        "a.jsonl: the child that runs this chat exited. Restart to keep talking.",
        "the English notice is the one the app already showed"
    );
}

#[test]
fn no_language_yet_is_english() {
    let (state, _) = update_chats(
        AppState::default(),
        Input::ChatClosed {
            chat: ChatId::of_session("a.jsonl"),
            expected: false,
        },
    );

    assert_eq!(
        state
            .notices
            .last()
            .expect("a death leaves one notice")
            .text,
        "a.jsonl: the child that runs this chat exited. Restart to keep talking.",
        "an unset language reads as English (D2: Rust never detects a locale)"
    );
}

#[test]
fn the_working_refusal_follows_the_language_it_is_asked_in() {
    assert_eq!(
        close_working(Language::Es),
        "Este chat está trabajando. Detenlo primero, luego ciérralo."
    );
    assert_eq!(
        close_working(Language::En),
        "This chat is working. Stop it first, then close it.",
        "the English sentence is the one the app already showed"
    );
}

#[test]
fn a_fold_that_rebuilds_the_state_keeps_the_language() {
    let (state, _) = update_chats(speaking(Language::Es), Input::Intent(Intent::NewChat));

    assert_eq!(
        state.language,
        Some(Language::Es),
        "New chat rebuilds the state, and the language is the shell's, not the fold's"
    );
}
