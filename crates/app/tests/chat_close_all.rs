//! Closing every chat at once (close-chats T1, S2, S3): one intent closes every
//! open chat through the very path `CloseChat` takes - each chat's stop command,
//! its removal from the open set, the closed set that keeps it closed, and the
//! foreground fallback - and it never bypasses the guard `CloseChat` has: a chat
//! that is working is SILENTLY skipped (no command, no notice, no change to it),
//! because stopping a turn is the user's.
//!
//! Nothing on disk changes: closing is not deleting, so no session-file command
//! is ever asked for and the session list keeps every row.

use gentle_app::{
    update_chats, AppState, ChatCommand, ChatId, Effects, Input, Intent, SessionItem,
};
use gentle_core::rpc::{RpcEvent, UiRequest};

const SESSION: &str = "/sessions/a.jsonl";
const OTHER_SESSION: &str = "/sessions/b.jsonl";
const THIRD_SESSION: &str = "/sessions/c.jsonl";
const CWD: &str = "/dev/repo";

fn chat(path: &str) -> ChatId {
    ChatId::of_session(path)
}

fn step(state: AppState, input: Input) -> (AppState, Effects) {
    update_chats(state, input)
}

/// The effects of one input, folded from the state so far.
fn intent(state: AppState, intent: Intent) -> (AppState, Effects) {
    step(state, Input::Intent(intent))
}

/// A chat that is open the way a click on its row opens it: foreground, child up.
fn open(state: AppState, path: &str) -> AppState {
    let (state, _) = intent(state, Intent::OpenChat(chat(path)));
    let (state, _) = step(state, Input::ChatOpened { chat: chat(path) });
    state
}

fn event(state: AppState, chat: &ChatId, event: RpcEvent) -> AppState {
    let (state, _) = step(
        state,
        Input::ChatEvent {
            chat: chat.clone(),
            event,
        },
    );
    state
}

fn dialog(id: &str, title: &str) -> RpcEvent {
    RpcEvent::UiRequest(UiRequest::Confirm {
        id: id.into(),
        title: title.into(),
        message: "go on?".into(),
        timeout: None,
    })
}

/// One session file of the list, as the first pass reports it.
fn session(path: &str) -> SessionItem {
    SessionItem {
        path: path.to_owned(),
        title: "A chat".to_owned(),
        cwd: CWD.to_owned(),
        modified_ms: 0,
        message_count: 1,
    }
}

/// The order a bulk close takes the open chats of `state` in: a stable id order
/// with the FOREGROUND chat last, so the view keeps a chat in front while the
/// others go instead of hopping between them.
fn expected_stops(state: &AppState) -> Vec<ChatCommand> {
    let foreground = state.chats.foreground().cloned();
    let mut chats: Vec<ChatId> = state.chats.ids().into_iter().cloned().collect();
    chats.sort_by_key(|chat| foreground.as_ref() == Some(chat));
    chats
        .into_iter()
        .map(|chat| ChatCommand::CloseProcess { chat })
        .collect()
}

#[test]
fn closing_every_chat_with_none_open_asks_for_nothing() {
    let before = AppState::default();

    let (state, effects) = intent(before.clone(), Intent::CloseAllChats);

    assert_eq!(state, before, "nothing to close, nothing changed");
    assert_eq!(effects.chat_commands, Vec::new(), "no child to stop");
    assert_eq!(
        effects.commands,
        Vec::new(),
        "and nothing of the shell runs"
    );
    assert_eq!(state.notices, Vec::new(), "and nothing is reported");
}

#[test]
fn closing_every_chat_takes_the_one_open_chat_off_the_open_set() {
    let state = open(AppState::default(), SESSION);

    let (state, effects) = intent(state, Intent::CloseAllChats);

    assert_eq!(state.chats.len(), 0, "the gate counts no open chat");
    assert_eq!(state.chats.foreground(), None, "nothing is on screen");
    assert_eq!(
        effects.chat_commands,
        vec![ChatCommand::CloseProcess {
            chat: chat(SESSION)
        }],
        "the shell stops that chat's child, exactly as CloseChat does"
    );
    assert_eq!(
        effects.commands,
        Vec::new(),
        "closing a chat runs nothing of the single-child shell"
    );
    assert_eq!(
        state.notices,
        Vec::new(),
        "a close that works reports nothing"
    );
}

#[test]
fn a_chat_closed_in_bulk_stays_closed_against_its_old_childs_inputs() {
    let state = open(AppState::default(), SESSION);
    let (state, _effects) = intent(state, Intent::CloseAllChats);

    // What a stopped child can still leave behind, in the order it can arrive.
    let (state, _) = step(
        state,
        Input::ChatStderr {
            chat: chat(SESSION),
            line: "gentle-shell: goodbye".to_owned(),
        },
    );
    let (state, _) = step(
        state,
        Input::ChatEvent {
            chat: chat(SESSION),
            event: RpcEvent::Response(gentle_core::rpc::RpcResponse {
                id: None,
                command: "get_session_stats".to_owned(),
                outcome: Ok(serde_json::json!({"tokens": 10, "contextWindow": 100})),
            }),
        },
    );
    let (state, _) = step(
        state,
        Input::ChatClosed {
            chat: chat(SESSION),
            expected: false,
        },
    );

    assert_eq!(
        state.chats.len(),
        0,
        "a last line, a reply in flight and a death do not reopen a closed chat"
    );
    assert_eq!(
        state.chats.foreground(),
        None,
        "nothing took the screen back"
    );
    assert!(
        state.notices.is_empty(),
        "and no Restart is offered for a chat that is not open: {:?}",
        state.notices
    );
}

#[test]
fn closing_every_chat_closes_them_all_and_the_foreground_one_last() {
    let (state, _) = step(
        AppState::default(),
        Input::SessionsLoaded(vec![session(SESSION), session(THIRD_SESSION)]),
    );
    // One started chat, one fresh chat this run allocated and never started (the
    // phantom the gate used to be stuck on), and one chat waiting for an answer.
    let state = open(state, SESSION);
    let (state, _) = intent(state, Intent::OpenNewChat);
    let fresh = state
        .chats
        .foreground()
        .cloned()
        .expect("the new chat is in front");
    let state = open(state, THIRD_SESSION);
    let state = event(state, &chat(THIRD_SESSION), dialog("d1", "Sure?"));
    assert_eq!(state.chats.len(), 3, "three open chats to close");
    assert_eq!(
        state.chats.foreground(),
        Some(&chat(THIRD_SESSION)),
        "the chat with the dialog is the one in front"
    );

    let before = state.clone();
    let (state, effects) = intent(state, Intent::CloseAllChats);

    let stops = expected_stops(&before);
    assert_eq!(
        effects.chat_commands, stops,
        "one stop per open chat, in id order with the foreground last"
    );
    assert_eq!(
        stops.last(),
        Some(&ChatCommand::CloseProcess {
            chat: chat(THIRD_SESSION)
        }),
        "the foreground chat is closed LAST, so the view never hops"
    );
    assert!(
        stops
            .iter()
            .any(|command| matches!(command, ChatCommand::CloseProcess { chat } if chat == &fresh)),
        "a chat whose child never started leaves the open set too: {stops:?}"
    );
    assert_eq!(state.chats.len(), 0, "every open chat left the open set");
    assert_eq!(state.chats.foreground(), None, "nothing is on screen");
    assert!(state.notices.is_empty(), "and nothing is reported");
    assert_eq!(
        effects.commands,
        Vec::new(),
        "no session-file command is asked for: closing a chat never deletes its session"
    );
    assert_eq!(
        state.sessions, before.sessions,
        "the session list keeps its rows"
    );
}

#[test]
fn a_working_chat_is_skipped_silently_while_the_idle_ones_close() {
    let state = open(AppState::default(), SESSION);
    let state = open(state, OTHER_SESSION);
    let state = open(state, THIRD_SESSION);
    // b is mid-turn: CloseChat refuses it, so a bulk close must not stop it.
    let state = event(state, &chat(OTHER_SESSION), RpcEvent::AgentStart);
    assert_eq!(
        state.chats.activity(&chat(OTHER_SESSION)),
        Some(gentle_app::ChatActivity::Working)
    );

    let before = state.clone();
    let (state, effects) = intent(state, Intent::CloseAllChats);

    assert_eq!(
        effects.chat_commands,
        vec![
            ChatCommand::CloseProcess {
                chat: chat(SESSION)
            },
            ChatCommand::CloseProcess {
                chat: chat(THIRD_SESSION)
            },
        ],
        "the idle chats close in id order, the foreground one last"
    );
    assert_eq!(state.chats.len(), 1, "only the working chat is still open");
    assert_eq!(
        state.chats.ids(),
        vec![&chat(OTHER_SESSION)],
        "and it is the one that was working"
    );
    assert_eq!(
        state.chats.foreground(),
        None,
        "the chat in front was idle, so it closed"
    );
    assert!(
        state.notices.is_empty(),
        "the skipped chat is not reported one by one: {:?}",
        state.notices
    );
    assert_eq!(
        state.sessions, before.sessions,
        "and no session file is touched"
    );
}

#[test]
fn a_bulk_close_of_only_working_chats_is_a_no_op() {
    let state = open(AppState::default(), SESSION);
    let state = event(state, &chat(SESSION), RpcEvent::AgentStart);
    let before = state.clone();

    let (state, effects) = intent(state, Intent::CloseAllChats);

    assert_eq!(state, before, "a working chat is never closed from here");
    assert_eq!(effects.chat_commands, Vec::new());
    assert_eq!(effects.commands, Vec::new());
    assert!(
        state.notices.is_empty(),
        "and the guard is not reported per chat: {:?}",
        state.notices
    );
}
