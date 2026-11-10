//! Windowless tests of the chat chrome slice of the contract (T6): the title the
//! chat header draws and the action a death notice carries, which is what the
//! composer and the restart notice need and the pushed state did not have.
//!
//! The state is seeded through the same reducer inputs the bridge folds, so what
//! is asserted is the slice the web side really receives: no child, no window
//! and no config file runs here.

use gentle_app::{ChatId, Input, Intent, SessionItem};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::{NoticeActionView, NoticeLevel, UiIntent, ViewState};

const SESSION_PATH: &str = "/sessions/a.jsonl";
const CWD: &str = "/dev/repo";

/// One session file of the list, as the first pass reports it.
fn session(path: &str, title: &str) -> SessionItem {
    SessionItem {
        path: path.to_owned(),
        title: title.to_owned(),
        cwd: CWD.to_owned(),
        modified_ms: 0,
        message_count: 1,
    }
}

/// The holder with one listed session and its chat open, which is the state a
/// click on the row reaches.
fn open_stored(title: &str) -> (AppStateHolder, ChatId) {
    let holder = AppStateHolder::default();
    holder.fold_input(Input::SessionsLoaded(vec![session(SESSION_PATH, title)]));
    let chat = ChatId::of_session(SESSION_PATH);
    holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
    holder.fold_input(Input::ChatOpened { chat: chat.clone() });
    (holder, chat)
}

#[test]
fn the_open_chat_carries_the_title_its_sidebar_row_draws() {
    let (holder, _chat) = open_stored("Tauri skeleton: window and bridge");

    let view = holder.view();

    assert_eq!(
        view.foreground_title,
        Some("Tauri skeleton: window and bridge".to_owned()),
        "the header names the chat by the same title the sidebar row draws"
    );
}

#[test]
fn the_header_title_is_the_whole_title_never_the_rows_cut() {
    // The sidebar row cuts its title at 32 characters; the header keeps the whole
    // name and truncates it for its own width, so a reader who asks for it (a
    // tooltip, a screen reader) still gets the full text.
    let long = "A title long enough that the sidebar row has to cut it short";
    let (holder, _chat) = open_stored(long);

    assert_eq!(holder.view().foreground_title, Some(long.to_owned()));
}

#[test]
fn a_new_chat_names_itself_until_its_session_file_exists() {
    let holder = AppStateHolder::default();
    holder.fold_input(Input::Intent(Intent::OpenNewChat));

    let view = holder.view();

    assert!(view.foreground_chat.is_some());
    assert_eq!(
        view.foreground_title,
        Some("New chat".to_owned()),
        "a chat with no session file yet is still named, never left blank"
    );
}

#[test]
fn no_open_chat_means_no_title() {
    let holder = AppStateHolder::default();

    let view: ViewState = holder.view();

    assert_eq!(view.foreground_title, None);
}

#[test]
fn a_death_notice_carries_the_action_that_restarts_its_chat() {
    let holder = AppStateHolder::default();
    let chat = ChatId::of_session(SESSION_PATH);

    let view = holder.fold_input(Input::ChatClosed {
        chat: chat.clone(),
        expected: false,
    });

    let notice = view
        .notices
        .first()
        .expect("an unexpected close leaves one notice");
    assert_eq!(notice.level, NoticeLevel::Error);
    assert_eq!(
        notice.action,
        Some(NoticeActionView::RestartChat {
            chat_id: chat.as_str().to_owned()
        }),
        "the restart notice names the chat its one action starts again"
    );
}

#[test]
fn the_notices_action_round_trips_through_the_restart_intent() {
    let holder = AppStateHolder::default();
    let chat = ChatId::of_session(SESSION_PATH);
    let view = holder.fold_input(Input::ChatClosed {
        chat: chat.clone(),
        expected: false,
    });
    let Some(NoticeActionView::RestartChat { chat_id }) = view
        .notices
        .first()
        .and_then(|notice| notice.action.clone())
    else {
        panic!("a death notice carries a restart action");
    };

    // The chat id the action carries is the one the intent takes back, and taking
    // it is the fix that takes the notice with it.
    let after = holder.dispatch(UiIntent::RestartChat { chat_id });

    assert_eq!(
        after.notices,
        Vec::new(),
        "the restarted chat's notice is gone"
    );
}

#[test]
fn a_notice_with_nothing_to_fix_carries_no_action() {
    let holder = AppStateHolder::default();
    // A command that could not reach a child: the app reports it and has nothing
    // to offer, so the composer draws no button for it.
    let view = holder.fold_input(Input::ChatFailed {
        chat: ChatId::fresh(),
        message: "the child could not be started".to_owned(),
    });

    let notice = view.notices.first().expect("a failed start is reported");
    assert_eq!(notice.level, NoticeLevel::Error);
    assert_eq!(notice.action, None);
}
