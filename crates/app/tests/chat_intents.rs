//! The intents that act on one chat's child: opening one, starting a new one,
//! and sending a prompt to the foreground chat.

use gentle_app::{
    update, update_chats, AppState, ChatActivity, ChatCommand, ChatId, Command, Effects, Input,
    Intent,
};
use gentle_core::rpc::{RpcCommand, RpcEvent, UiAnswer, UiRequest};

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

/// What the real UI sends the moment the host answers an open: the opening
/// phase is over and the chat can take a prompt again.
fn opened(state: AppState, chat: &ChatId) -> (AppState, Effects) {
    step(state, Input::ChatOpened { chat: chat.clone() })
}

/// What the composer sends the first time a new chat's box is focused, or the
/// first key lands in it (welcome-screen T4, R2).
fn ensure_started(state: AppState, chat: &ChatId) -> (AppState, Effects) {
    step(state, Input::Intent(Intent::EnsureStarted(chat.clone())))
}

/// The foreground chat, as the slice names it back to the page.
fn foreground(state: &AppState) -> ChatId {
    state
        .chats
        .foreground()
        .cloned()
        .expect("a foreground chat")
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
    let first_chat = foreground(&state);
    let (state, second) = step(state, Input::Intent(Intent::OpenNewChat));
    let second_chat = foreground(&state);

    assert_ne!(first_chat, second_chat);
    assert_eq!(state.chats.len(), 2);
    assert_eq!(
        first.chat_commands,
        Vec::new(),
        "opening a new chat starts nothing: its child waits for the composer (welcome-screen T4, R2)"
    );
    assert_eq!(second.chat_commands, Vec::new());
    assert_eq!(first.commands, Vec::new());
    assert_eq!(second.commands, Vec::new());
}

#[test]
fn a_new_chat_asks_for_its_child_only_when_its_composer_asks_and_only_once() {
    // R2: the click allocates the chat, and the process starts on the composer's
    // own trigger. Every later trigger of the same chat changes nothing: the
    // focus and the first keystroke of one box ask once between them.
    let (state, allocated) = step(AppState::default(), Input::Intent(Intent::OpenNewChat));
    let chat = foreground(&state);
    assert_eq!(
        allocated.chat_commands,
        Vec::new(),
        "the click starts nothing"
    );
    assert_eq!(
        state.chats.activity(&chat),
        Some(ChatActivity::Opening),
        "the chat says it is coming up, so the composer keeps its reason"
    );

    let (state, first) = ensure_started(state, &chat);
    assert_eq!(
        first.chat_commands,
        vec![ChatCommand::OpenProcess { chat: chat.clone() }],
        "the first trigger starts the child"
    );
    assert_eq!(first.commands, Vec::new());
    assert_eq!(
        state.chats.activity(&chat),
        Some(ChatActivity::Opening),
        "and the chat stays opening until its child answers"
    );

    let (state, second) = ensure_started(state, &chat);
    assert_eq!(
        second,
        Effects::default(),
        "a second trigger while the child is coming up does nothing"
    );

    // The child answers, the chat is usable, and a later trigger still asks for
    // nothing: the start of one chat is behind it.
    let (state, _) = opened(state, &chat);
    assert_eq!(state.chats.activity(&chat), Some(ChatActivity::Idle));
    let (state, again) = ensure_started(state, &chat);
    assert_eq!(
        again,
        Effects::default(),
        "a chat whose child is up is not started twice"
    );
    assert_eq!(state.chats.activity(&chat), Some(ChatActivity::Idle));
}

#[test]
fn ensure_started_changes_nothing_for_a_chat_that_is_already_working() {
    let (state, _) = open(AppState::default(), &chat("a"));
    let (state, _) = step(
        state,
        Input::ChatEvent {
            chat: chat("a"),
            event: RpcEvent::AgentStart,
        },
    );

    let (state, effects) = ensure_started(state, &chat("a"));

    assert_eq!(effects, Effects::default(), "no child is asked for twice");
    assert_eq!(
        state.chats.activity(&chat("a")),
        Some(ChatActivity::Working),
        "the turn the chat is running is untouched"
    );
}

#[test]
fn ensure_started_starts_a_chat_this_run_never_started() {
    // A chat the state has never heard of - the id of a stored session the page
    // holds - is started by the same ask, and it is Opening from that moment.
    let (state, effects) = ensure_started(AppState::default(), &chat("a"));

    assert_eq!(
        effects.chat_commands,
        vec![ChatCommand::OpenProcess { chat: chat("a") }]
    );
    assert_eq!(
        state.chats.activity(&chat("a")),
        Some(ChatActivity::Opening)
    );
}

#[test]
fn ensure_started_starts_again_only_for_a_chat_that_was_closed() {
    let (state, _) = step(AppState::default(), Input::Intent(Intent::OpenNewChat));
    let chat = foreground(&state);
    let (state, _) = ensure_started(state, &chat);
    let (state, _) = opened(state, &chat);
    let (state, closed) = step(state, Input::Intent(Intent::CloseChat(chat.clone())));
    assert_eq!(
        closed.chat_commands,
        vec![ChatCommand::CloseProcess { chat: chat.clone() }],
        "the chat closed, so its child went away"
    );

    let (state, effects) = ensure_started(state, &chat);

    assert_eq!(
        effects.chat_commands,
        vec![ChatCommand::OpenProcess { chat: chat.clone() }],
        "a chat that left the open set has no child, so it is asked for again"
    );
    assert_eq!(
        state.chats.activity(&chat),
        Some(ChatActivity::Opening),
        "and it is Opening again until the new child answers"
    );
}

#[test]
fn a_prompt_goes_to_the_foreground_chat_only() {
    let (state, _) = open(AppState::default(), &chat("a"));
    let (state, _) = opened(state, &chat("a"));
    let (state, _) = open(state, &chat("b"));
    let (state, _) = opened(state, &chat("b"));

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
    assert_eq!(sent.messages().len(), 1);
    assert_eq!(sent.messages()[0].text, "hola");
    assert_eq!(sent.messages()[0].id, "msg-0");
    assert!(
        state
            .chats
            .state(&chat("a"))
            .is_none_or(|a| a.messages().is_empty()),
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
    let messages = &state.chats.state(&chat("a")).expect("chat a").messages();
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
        .messages()
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
    assert_eq!(state.chat.messages().len(), 1);
    assert!(state.chats.is_empty());
}

#[test]
fn stopping_sends_an_abort_to_the_foreground_chat_and_nothing_else() {
    let (state, _) = open(AppState::default(), &chat("a"));
    let (state, _) = step(
        state,
        Input::ChatEvent {
            chat: chat("a"),
            event: RpcEvent::AgentStart,
        },
    );

    let (_, effects) = step(state, Input::Intent(Intent::Abort));

    assert_eq!(
        effects.chat_commands,
        vec![ChatCommand::Send {
            chat: chat("a"),
            rpc: RpcCommand::Abort,
        }]
    );
    assert_eq!(effects.commands, Vec::new(), "not the single child");
}

#[test]
fn copying_a_message_copies_it_from_the_foreground_chat() {
    let (state, _) = open(AppState::default(), &chat("a"));
    let (state, _) = opened(state, &chat("a"));
    let (state, _) = step(state, Input::Intent(Intent::SendPrompt("hola".into())));

    let (_, effects) = step(state, Input::Intent(Intent::CopyMessage("msg-0".into())));

    assert_eq!(
        effects.commands,
        vec![Command::CopyToClipboard("hola".into())]
    );
    assert_eq!(effects.chat_commands, Vec::new());
}

#[test]
fn a_dialog_is_answered_in_the_chat_that_raised_it() {
    let (state, _) = open(AppState::default(), &chat("a"));
    let (state, _) = step(
        state,
        Input::ChatEvent {
            chat: chat("a"),
            event: RpcEvent::UiRequest(UiRequest::Confirm {
                id: "d1".into(),
                title: "run?".into(),
                message: "go on?".into(),
                timeout: None,
            }),
        },
    );

    let (state, effects) = step(
        state,
        Input::Intent(Intent::AnswerDialog {
            id: "d1".into(),
            answer: UiAnswer::Confirmed(true),
        }),
    );

    assert_eq!(
        effects.chat_commands,
        vec![ChatCommand::Send {
            chat: chat("a"),
            rpc: RpcCommand::UiResponse {
                id: "d1".into(),
                answer: UiAnswer::Confirmed(true),
            },
        }]
    );
    assert!(state.chats.dialogs(&chat("a")).is_empty());
}

#[test]
fn another_chats_dialog_is_not_answered_from_the_foreground_chat() {
    let (state, _) = open(AppState::default(), &chat("a"));
    let (state, _) = step(
        state,
        Input::ChatEvent {
            chat: chat("a"),
            event: RpcEvent::UiRequest(UiRequest::Select {
                id: "d1".into(),
                title: "pick".into(),
                options: vec!["one".into()],
                timeout: None,
            }),
        },
    );
    let (state, _) = open(state, &chat("b"));

    let (state, effects) = step(
        state,
        Input::Intent(Intent::AnswerDialog {
            id: "d1".into(),
            answer: UiAnswer::Value("one".into()),
        }),
    );

    assert_eq!(effects.chat_commands, Vec::new(), "no child is asked");
    assert_eq!(state.chats.dialogs(&chat("a")).len(), 1, "still waiting");
}

#[test]
fn no_click_produces_a_switch_session_or_a_new_session() {
    // S1: the session of a chat travels on the command line when its child is
    // started. Nothing a click can produce replaces a live child's session.
    let clicks = [
        Input::Intent(Intent::OpenChat(chat("a"))),
        Input::Intent(Intent::OpenNewChat),
        Input::Intent(Intent::SendPrompt("hola".into())),
        Input::Intent(Intent::Abort),
    ];
    let mut state = AppState::default();
    for click in clicks {
        let (next, effects) = step(state, click.clone());
        assert_no_session_replacement(&click, &effects);
        state = next;
    }
}

fn assert_no_session_replacement(input: &Input, effects: &Effects) {
    for command in &effects.chat_commands {
        if let ChatCommand::Send { rpc, .. } = command {
            assert_no_replacement(input, rpc);
        }
    }
    for command in &effects.commands {
        if let Command::Send(rpc) = command {
            assert_no_replacement(input, rpc);
        }
    }
}

fn assert_no_replacement(input: &Input, rpc: &RpcCommand) {
    assert!(
        !matches!(
            rpc,
            RpcCommand::NewSession | RpcCommand::SwitchSession { .. }
        ),
        "{input:?} produced {rpc:?}"
    );
}
