//! The intents that act on one chat's child: opening one, starting a new one,
//! and sending a prompt to the foreground chat.

use gentle_app::{
    update, update_chats, AppState, ChatActivity, ChatCommand, ChatId, Command, Effects, Input,
    Intent,
};
use gentle_core::rpc::{RpcCommand, RpcEvent};

fn chat(name: &str) -> ChatId {
    ChatId::of_session(name)
}

/// The effects of one input, folded from the state so far.
fn step(state: AppState, input: Input) -> (AppState, Effects) {
    update_chats(state, input)
}

fn open(state: AppState, chat: &ChatId) -> (AppState, Effects) {
    step(state, Input::Intent(Intent::OpenChat(chat.clone())))
}

#[test]
fn opening_a_chat_asks_the_host_for_it_and_never_for_a_session_switch() {
    let (state, effects) = open(AppState::default(), &chat("a"));

    assert_eq!(
        effects.chat_commands,
        vec![ChatCommand::OpenProcess { chat: chat("a") }]
    );
    assert_eq!(
        effects.commands,
        Vec::new(),
        "no switch_session and no new_session: the session goes on the command line"
    );
    assert_eq!(state.chats.foreground(), Some(&chat("a")));
}

#[test]
fn opening_another_chat_only_moves_the_foreground() {
    let (state, _) = open(AppState::default(), &chat("a"));
    let (state, _) = step(
        state,
        Input::ChatEvent {
            chat: chat("a"),
            event: RpcEvent::AgentStart,
        },
    );
    let (state, effects) = open(state, &chat("b"));

    assert_eq!(
        effects.chat_commands,
        vec![ChatCommand::OpenProcess { chat: chat("b") }]
    );
    assert_eq!(state.chats.foreground(), Some(&chat("b")));
    assert_eq!(
        state.chats.activity(&chat("a")),
        Some(ChatActivity::Working),
        "the chat left behind keeps working"
    );
}

#[test]
fn a_new_chat_gets_an_id_of_its_own_and_becomes_the_foreground_one() {
    let (state, first) = step(AppState::default(), Input::Intent(Intent::OpenNewChat));
    let (state, second) = step(state, Input::Intent(Intent::OpenNewChat));

    let ChatCommand::OpenProcess { chat: first_chat } = &first.chat_commands[0] else {
        panic!(
            "a new chat must ask the host for its child: {:?}",
            first.chat_commands
        );
    };
    let ChatCommand::OpenProcess { chat: second_chat } = &second.chat_commands[0] else {
        panic!(
            "a new chat must ask the host for its child: {:?}",
            second.chat_commands
        );
    };
    assert_ne!(first_chat, second_chat);
    assert_eq!(state.chats.foreground(), Some(second_chat));
    assert_eq!(state.chats.len(), 2);
    assert_eq!(first.commands, Vec::new());
    assert_eq!(second.commands, Vec::new());
}

#[test]
fn a_prompt_goes_to_the_foreground_chat_only() {
    let (state, _) = open(AppState::default(), &chat("a"));
    let (state, _) = open(state, &chat("b"));

    let (state, effects) = step(state, Input::Intent(Intent::SendPrompt("hola".into())));

    assert_eq!(
        effects.chat_commands,
        vec![ChatCommand::Send {
            chat: chat("b"),
            rpc: RpcCommand::Prompt {
                message: "hola".into()
            },
        }]
    );
    assert_eq!(effects.commands, Vec::new(), "not the single-chat command");
    let sent = state.chats.state(&chat("b")).expect("chat b");
    assert_eq!(sent.messages.len(), 1);
    assert_eq!(sent.messages[0].text, "hola");
    assert_eq!(sent.messages[0].id, "msg-0");
    assert!(
        state
            .chats
            .state(&chat("a"))
            .is_none_or(|a| a.messages.is_empty()),
        "the chat the user left receives nothing"
    );
}

#[test]
fn a_prompt_is_refused_while_that_chat_is_working() {
    let (state, _) = open(AppState::default(), &chat("a"));
    let (state, _) = step(
        state,
        Input::ChatEvent {
            chat: chat("a"),
            event: RpcEvent::AgentStart,
        },
    );

    let (state, effects) = step(state, Input::Intent(Intent::SendPrompt("otra".into())));

    assert_eq!(effects.chat_commands, Vec::new());
    let messages = &state.chats.state(&chat("a")).expect("chat a").messages;
    assert!(messages.is_empty());
}

#[test]
fn a_blank_prompt_does_nothing() {
    let (state, _) = open(AppState::default(), &chat("a"));

    let (state, effects) = step(state, Input::Intent(Intent::SendPrompt("  \n".into())));

    assert_eq!(effects.chat_commands, Vec::new());
    assert!(state
        .chats
        .state(&chat("a"))
        .expect("chat a")
        .messages
        .is_empty());
}

#[test]
fn with_no_foreground_chat_the_single_chat_path_still_answers() {
    // The one-child shell never opens a chat, so `SendPrompt` keeps emitting the
    // command it always did and `update` stays the shell's whole answer.
    let (state, commands) = update(
        AppState::default(),
        Input::Intent(Intent::SendPrompt("hola".into())),
    );

    assert_eq!(
        commands,
        vec![Command::Send(RpcCommand::Prompt {
            message: "hola".into()
        })]
    );
    assert_eq!(state.chat.messages.len(), 1);
    assert!(state.chats.is_empty());
}
