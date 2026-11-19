//! Windowless tests of the folder chip's slice of the contract (design/
//! composer-dock S2, R4): the LAST SEGMENT of the foreground chat's working
//! directory, the same name its sidebar row draws, with the whole path beside it
//! for the hover.
//!
//! The state is seeded through the same reducer inputs the bridge folds, so what
//! is asserted is the slice the web side really receives: no child, no window
//! and no config file runs here.

use gentle_app::{ChatId, Input, Intent, SessionItem};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::ViewState;

const SESSION_PATH: &str = "/sessions/a.jsonl";

/// One session file of the list, as the first pass reports it.
fn session(cwd: &str) -> SessionItem {
    SessionItem {
        path: SESSION_PATH.to_owned(),
        title: "a chat".to_owned(),
        cwd: cwd.to_owned(),
        modified_ms: 0,
        message_count: 1,
    }
}

/// The holder with that session listed and its chat open.
fn open_in(cwd: &str) -> AppStateHolder {
    let holder = AppStateHolder::default();
    holder.fold_input(Input::SessionsLoaded(vec![session(cwd)]));
    let chat = ChatId::of_session(SESSION_PATH);
    holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
    holder.fold_input(Input::ChatOpened { chat });
    holder
}

#[test]
fn the_open_chat_carries_the_last_segment_and_the_whole_path() {
    let holder = open_in("/dev/proyectos/Kallari_Pagina_web");

    let folder = holder
        .view()
        .foreground_folder
        .expect("an open chat with a listed session has a folder");

    assert_eq!(folder.name, "Kallari_Pagina_web");
    assert_eq!(folder.path, "/dev/proyectos/Kallari_Pagina_web");
}

#[test]
fn a_windows_path_with_a_trailing_separator_names_the_same_segment() {
    // The rule is `SessionItem::project`, the one the sidebar row uses: both
    // separators count and a trailing one does not name an empty folder.
    let holder = open_in(r"C:\Users\josep\dev\gentle-desk\");

    let folder = holder.view().foreground_folder.expect("a folder is named");

    assert_eq!(folder.name, "gentle-desk");
    assert_eq!(folder.path, r"C:\Users\josep\dev\gentle-desk\");
}

#[test]
fn a_chat_with_no_listed_session_carries_no_folder() {
    // A chat started a moment ago has no session row yet: absent, not a stub.
    let holder = AppStateHolder::default();
    holder.fold_input(Input::Intent(Intent::OpenNewChat));

    let view: ViewState = holder.view();

    assert!(view.foreground_chat.is_some());
    assert_eq!(view.foreground_folder, None);
}

#[test]
fn no_open_chat_means_no_folder() {
    let holder = AppStateHolder::default();

    assert_eq!(holder.view().foreground_folder, None);
}
