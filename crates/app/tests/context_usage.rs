//! The context reading through the app's public interface (0.1.12, S5): the
//! chat asks its own child for its statistics when a turn settles, and the
//! reply fills THAT chat's reading - never a zero the child did not report.

use gentle_app::{update_chats, AppState, ChatCommand, ChatId, Effects, Input, Intent};
use gentle_core::rpc::{decode_line, Decoded, RpcCommand, RpcEvent};

fn chat(name: &str) -> ChatId {
    ChatId::of_session(name)
}

/// One folded input from the state so far.
fn step(state: AppState, input: Input) -> (AppState, Effects) {
    update_chats(state, input)
}

/// The state after opening `chat` and confirming its child is up.
fn opened(chat_id: &ChatId) -> AppState {
    let (state, _) = step(
        AppState::default(),
        Input::Intent(Intent::OpenChat(chat_id.clone())),
    );
    let (state, _) = step(
        state,
        Input::ChatOpened {
            chat: chat_id.clone(),
        },
    );
    state
}

/// One event of one chat, folded.
fn event(state: AppState, chat_id: &ChatId, line: &str) -> (AppState, Effects) {
    let Decoded::Event(event) = decode_line(line) else {
        panic!("not an event: {line}");
    };
    step(
        state,
        Input::ChatEvent {
            chat: chat_id.clone(),
            event,
        },
    )
}

/// The `get_session_stats` ask one chat's settled turn sends.
fn stats_ask(chat_id: &ChatId) -> ChatCommand {
    ChatCommand::Send {
        chat: chat_id.clone(),
        rpc: RpcCommand::GetSessionStats,
    }
}

#[test]
fn a_settled_turn_asks_that_chats_child_for_its_statistics() {
    let (state, effects) = event(
        opened(&chat("a")),
        &chat("a"),
        "{\"type\":\"agent_settled\"}",
    );

    assert!(
        effects.chat_commands.contains(&stats_ask(&chat("a"))),
        "the settled turn asks its own child, got {:?}",
        effects.chat_commands
    );
    assert!(
        effects.chat_commands.iter().all(|command| matches!(
            command,
            ChatCommand::Send { chat: to, .. } if to == &chat("a")
        )),
        "nothing of this fold reaches another chat: {:?}",
        effects.chat_commands
    );
    // The list pass is still asked for: the context ask is added, not swapped.
    assert_eq!(effects.commands, vec![gentle_app::Command::RefreshSessions]);
    assert_eq!(state.chats.context(&chat("a")), None, "no reading yet");
}

#[test]
fn the_reply_is_that_chats_reading_and_no_other() {
    let state = opened(&chat("a"));
    let (state, _) = step(state, Input::Intent(Intent::OpenChat(chat("b"))));

    // A background chat's settled turn asks its own child and its reply is kept
    // for when the user returns to it (S5).
    let (state, effects) = event(state, &chat("b"), "{\"type\":\"agent_settled\"}");
    assert!(effects.chat_commands.contains(&stats_ask(&chat("b"))));

    let (state, _) = event(
        state,
        &chat("b"),
        "{\"type\":\"response\",\"command\":\"get_session_stats\",\"success\":true,\
         \"data\":{\"contextUsage\":{\"tokens\":38000,\"contextWindow\":200000,\"percent\":19.0}}}",
    );

    let background = state.chats.context(&chat("b")).expect("b has a reading");
    assert_eq!(background.tokens, Some(38_000));
    assert_eq!(background.context_window, Some(200_000));
    assert_eq!(background.percent, Some(19.0));
    assert_eq!(
        state.chats.context(&chat("a")),
        None,
        "another chat's reply never fills this one"
    );
}

#[test]
fn a_compacted_reply_is_a_reading_with_no_numbers() {
    // Right after a compaction pi answers both as null: the app must not turn
    // that into 0 (S5), so the fields stay unknown.
    let (state, _) = event(
        opened(&chat("a")),
        &chat("a"),
        "{\"type\":\"response\",\"command\":\"get_session_stats\",\"success\":true,\
         \"data\":{\"contextUsage\":{\"tokens\":null,\"contextWindow\":200000,\"percent\":null}}}",
    );

    let usage = state.chats.context(&chat("a")).expect("a reading");
    assert_eq!(usage.tokens, None);
    assert_eq!(usage.percent, None);
    assert_eq!(usage.context_window, Some(200_000));
}

#[test]
fn a_reply_with_no_context_usage_changes_nothing() {
    let state = opened(&chat("a"));
    let (state, _) = event(
        state,
        &chat("a"),
        "{\"type\":\"response\",\"command\":\"get_session_stats\",\"success\":true,\
         \"data\":{\"tokens\":1}}",
    );
    assert_eq!(state.chats.context(&chat("a")), None);

    // A failed ask is as quiet as a silent one: a statistics read that fails is
    // no news worth a notice.
    let (state, _) = event(
        state,
        &chat("a"),
        "{\"type\":\"response\",\"command\":\"get_session_stats\",\"success\":false,\
         \"error\":\"not supported\"}",
    );
    assert_eq!(state.chats.context(&chat("a")), None);
    assert!(
        state.notices.is_empty(),
        "a periodic read that fails is not a notice, got {:?}",
        state.notices
    );
}

#[test]
fn a_new_reading_replaces_the_previous_one() {
    let state = opened(&chat("a"));
    let (state, _) = event(
        state,
        &chat("a"),
        "{\"type\":\"response\",\"command\":\"get_session_stats\",\"success\":true,\
         \"data\":{\"contextUsage\":{\"tokens\":38000,\"contextWindow\":200000,\"percent\":19.0}}}",
    );
    let (state, _) = event(
        state,
        &chat("a"),
        "{\"type\":\"response\",\"command\":\"get_session_stats\",\"success\":true,\
         \"data\":{\"contextUsage\":{\"tokens\":150000,\"contextWindow\":200000,\"percent\":75.0}}}",
    );

    let usage = state.chats.context(&chat("a")).expect("a reading");
    assert_eq!(usage.tokens, Some(150_000));
    assert_eq!(usage.percent, Some(75.0));
}

#[test]
fn another_response_of_the_same_chat_is_not_a_reading() {
    // The chip's replies and the thread's history travel the same path and must
    // not be read as statistics.
    let (state, _) = event(
        opened(&chat("a")),
        &chat("a"),
        "{\"type\":\"response\",\"command\":\"get_state\",\"success\":true,\
         \"data\":{\"contextUsage\":{\"tokens\":5,\"contextWindow\":10,\"percent\":50}}}",
    );
    assert_eq!(state.chats.context(&chat("a")), None);
}

#[test]
fn the_one_child_shell_is_untouched_by_the_context_reading() {
    // `update` is the single-child contract: it answers the same shared command
    // and never a chat command, so a settle there asks nothing of any chat.
    let (_, commands) = gentle_app::update(AppState::default(), Input::Rpc(RpcEvent::AgentSettled));
    assert_eq!(commands, vec![gentle_app::Command::RefreshSessions]);
}
