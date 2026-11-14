//! Closing a chat (0.1.14): the row menu's Close chat stops that chat's child
//! and takes the chat off the open set, which is what the Updates pane's gate
//! counts. Nothing on disk changes and the session list keeps its row, so this
//! is NOT a delete: the row is still there and opens the chat again.
//!
//! A chat that is running a turn is refused with the delete flow's own rule and
//! its sentence: the child is mid-turn, and stopping it is the user's, so the
//! row menu asks for that first.

use gentle_app::{
    close_working, update_chats, AppState, ChatCommand, ChatId, Effects, Input, Intent,
    NoticeLevel, SessionItem, WORKING_CHAT,
};
use gentle_core::rpc::RpcEvent;

const SESSION: &str = "/sessions/a.jsonl";
const OTHER_SESSION: &str = "/sessions/b.jsonl";
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

#[test]
fn closing_a_chat_takes_it_off_the_open_set_and_asks_for_its_child_to_stop() {
    let state = open(AppState::default(), SESSION);

    let (state, effects) = intent(state, Intent::CloseChat(chat(SESSION)));

    assert_eq!(
        state.chats.len(),
        0,
        "the open set the Updates gate counts is empty again"
    );
    assert_eq!(
        state.chats.foreground(),
        None,
        "the chat the user was on is not on screen any more"
    );
    assert_eq!(
        effects.chat_commands,
        vec![ChatCommand::CloseProcess {
            chat: chat(SESSION)
        }],
        "the shell stops exactly that chat's child"
    );
    assert_eq!(
        effects.commands,
        Vec::new(),
        "closing a chat runs nothing of the single-child shell"
    );
}

#[test]
fn closing_one_chat_leaves_every_other_chat_open_and_their_children_alone() {
    let state = open(AppState::default(), SESSION);
    let state = open(state, OTHER_SESSION);

    let (state, effects) = intent(state, Intent::CloseChat(chat(OTHER_SESSION)));

    assert_eq!(
        state.chats.len(),
        1,
        "only the closed chat left the open set"
    );
    assert!(
        state.chats.ids().iter().any(|open| *open == &chat(SESSION)),
        "the chat the user was not closing is still there"
    );
    assert_eq!(state.chats.foreground(), None);
    assert_eq!(
        effects.chat_commands,
        vec![ChatCommand::CloseProcess {
            chat: chat(OTHER_SESSION)
        }],
        "no command reaches another chat's child"
    );
}

#[test]
fn closing_a_chat_never_touches_the_session_list_or_its_file() {
    let (state, _) = step(
        AppState::default(),
        Input::SessionsLoaded(vec![session(SESSION)]),
    );
    let state = open(state, SESSION);

    let (state, effects) = intent(state, Intent::CloseChat(chat(SESSION)));

    assert_eq!(
        state.sessions,
        vec![session(SESSION)],
        "the list keeps the chat's row: closing is not deleting"
    );
    assert!(
        effects.commands.is_empty()
            && effects.chat_commands
                == vec![ChatCommand::CloseProcess {
                    chat: chat(SESSION)
                }],
        "nothing but the stop is asked for: no removal, no rename, no write"
    );
    assert_eq!(
        state.notices,
        Vec::new(),
        "a close that works reports nothing"
    );
}

#[test]
fn a_working_chat_is_refused_with_the_delete_flows_own_words() {
    let state = open(AppState::default(), SESSION);
    let (state, _) = step(
        state,
        Input::ChatEvent {
            chat: chat(SESSION),
            event: RpcEvent::AgentStart,
        },
    );
    assert_eq!(state.chats.len(), 1);

    let (state, effects) = intent(state, Intent::CloseChat(chat(SESSION)));

    assert_eq!(state.chats.len(), 1, "a working chat stays open");
    assert_eq!(
        effects.chat_commands,
        Vec::new(),
        "its child is never stopped from here: the user stops it"
    );
    let notice = state.notices.last().expect("the refusal is one notice");
    assert_eq!(notice.level, NoticeLevel::Warning);
    assert_eq!(
        notice.text,
        close_working(),
        "the refusal is the delete flow's sentence, with this action's verb"
    );
    assert_eq!(
        notice.text,
        "This chat is working. Stop it first, then close it."
    );
    assert!(
        notice.text.starts_with(WORKING_CHAT),
        "the rule and the words are the delete flow's, not a second wording"
    );
    assert_eq!(notice.action, None, "there is nothing to press on it");
}

#[test]
fn a_chat_that_holds_a_queued_prompt_takes_that_prompt_with_it() {
    // One working slot: a's turn holds it, so b's prompt waits in the queue.
    let (state, _) = step(AppState::default(), Input::SetWorkingCap(1));
    let state = open(state, SESSION);
    let (state, _) = intent(state, Intent::SendPrompt("a's turn".to_owned()));
    let (state, _) = step(
        state,
        Input::ChatEvent {
            chat: chat(SESSION),
            event: RpcEvent::AgentStart,
        },
    );
    let state = open(state, OTHER_SESSION);
    let (state, _) = intent(state, Intent::SendPrompt("b waits".to_owned()));
    assert!(
        state.chats.is_queued(&chat(OTHER_SESSION)),
        "b's prompt is waiting for the slot a holds"
    );

    let (state, effects) = intent(state, Intent::CloseChat(chat(OTHER_SESSION)));
    assert_eq!(
        effects.chat_commands,
        vec![ChatCommand::CloseProcess {
            chat: chat(OTHER_SESSION)
        }]
    );

    // a's turn ends and frees the slot: the closed chat's prompt must not go to
    // a child that is gone (which would put the chat back in the open set).
    let (state, effects) = step(
        state,
        Input::ChatEvent {
            chat: chat(SESSION),
            event: RpcEvent::AgentSettled,
        },
    );
    assert_eq!(
        state.chats.len(),
        1,
        "the closed chat is not put back by a prompt of its own"
    );
    assert!(
        !effects
            .chat_commands
            .iter()
            .any(|command| command.chat() == &chat(OTHER_SESSION)),
        "nothing is sent to the chat that was closed"
    );
}

#[test]
fn closing_a_chat_the_app_never_opened_changes_nothing() {
    let (state, _) = step(
        AppState::default(),
        Input::SessionsLoaded(vec![session(SESSION)]),
    );
    let before = state.clone();

    let (state, effects) = intent(state, Intent::CloseChat(chat(SESSION)));

    assert_eq!(
        state, before,
        "a chat this run never opened has no child to stop and no place to leave"
    );
    assert_eq!(effects.chat_commands, Vec::new());
    assert_eq!(state.notices, Vec::new());
}
