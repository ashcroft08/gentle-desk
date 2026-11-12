//! Per-chat state through the app layer's public interface: one event reaches
//! one chat, the activity the sidebar shows follows it, and nothing else moves.

use gentle_app::{update_chats, AppState, ChatActivity, ChatId, Command, Input, Intent};
use gentle_core::rpc::{decode_line, Decoded, RpcEvent, UiRequest};

fn chat(name: &str) -> ChatId {
    ChatId::of_session(name)
}

fn apply(state: AppState, inputs: Vec<Input>) -> (AppState, Vec<gentle_app::Effects>) {
    inputs
        .into_iter()
        .fold((state, Vec::new()), |(state, mut all), input| {
            let (next, effects) = update_chats(state, input);
            all.push(effects);
            (next, all)
        })
}

fn event(chat: &ChatId, event: RpcEvent) -> Input {
    Input::ChatEvent {
        chat: chat.clone(),
        event,
    }
}

fn open(chat: &ChatId) -> Input {
    Input::Intent(Intent::OpenChat(chat.clone()))
}

/// What the real UI sends the moment the host answers an open, so the
/// opening phase is over from the next step on.
fn confirmed(chat: &ChatId) -> Input {
    Input::ChatOpened { chat: chat.clone() }
}

fn dialog(id: &str, title: &str) -> RpcEvent {
    RpcEvent::UiRequest(UiRequest::Confirm {
        id: id.into(),
        title: title.into(),
        message: "go on?".into(),
        timeout: None,
    })
}

#[test]
fn an_event_reaches_only_the_chat_it_names() {
    let (state, _) = apply(
        AppState::default(),
        vec![event(&chat("a"), RpcEvent::AgentStart)],
    );

    assert_eq!(
        state.chats.activity(&chat("a")),
        Some(ChatActivity::Working)
    );
    assert_eq!(state.chats.state(&chat("b")), None);
    assert_eq!(state.chats.ids(), vec![&chat("a")]);
    // The single-chat state the one-child shell still renders is untouched.
    assert_eq!(state.chat, Default::default());
}

#[test]
fn an_event_for_an_unknown_chat_creates_it() {
    let (state, _) = apply(
        AppState::default(),
        vec![event(&chat("nueva"), RpcEvent::AgentStart)],
    );

    assert!(state.chats.state(&chat("nueva")).is_some());
    assert_eq!(state.chats.len(), 1);
}

#[test]
fn a_background_chat_that_finishes_is_finished_unread() {
    let (state, effects) = apply(
        AppState::default(),
        vec![
            open(&chat("a")),
            confirmed(&chat("a")),
            open(&chat("b")),
            confirmed(&chat("b")),
            event(&chat("a"), RpcEvent::AgentStart),
            event(&chat("a"), RpcEvent::AgentSettled),
        ],
    );

    assert_eq!(
        state.chats.activity(&chat("a")),
        Some(ChatActivity::FinishedUnread)
    );
    assert_eq!(
        state.chats.activity(&chat("b")),
        Some(ChatActivity::Idle),
        "the chat the user is on has nothing to read"
    );
    // A finished turn may have written a session file: the list is refreshed
    // exactly as on the single-chat path.
    // The AgentSettled step: the confirmed opens before it ask for nothing.
    assert_eq!(effects[5].commands, vec![Command::RefreshSessions]);
}

#[test]
fn the_foreground_chat_never_shows_finished_unread() {
    let (state, _) = apply(
        AppState::default(),
        vec![
            open(&chat("a")),
            confirmed(&chat("a")),
            event(&chat("a"), RpcEvent::AgentStart),
            event(&chat("a"), RpcEvent::AgentSettled),
        ],
    );

    assert_eq!(state.chats.activity(&chat("a")), Some(ChatActivity::Idle));
}

#[test]
fn opening_a_chat_clears_its_finished_unread() {
    let (state, _) = apply(
        AppState::default(),
        vec![
            open(&chat("a")),
            confirmed(&chat("a")),
            open(&chat("b")),
            event(&chat("a"), RpcEvent::AgentSettled),
            open(&chat("a")),
            confirmed(&chat("a")),
        ],
    );

    assert_eq!(state.chats.activity(&chat("a")), Some(ChatActivity::Idle));
    assert_eq!(state.chats.foreground(), Some(&chat("a")));
}

#[test]
fn a_pending_dialog_needs_attention_and_the_dialogs_are_queued() {
    let (state, _) = apply(
        AppState::default(),
        vec![
            open(&chat("a")),
            open(&chat("b")),
            event(&chat("a"), dialog("d1", "first")),
            event(&chat("a"), dialog("d2", "second")),
        ],
    );

    assert_eq!(
        state.chats.activity(&chat("a")),
        Some(ChatActivity::NeedsAttention),
        "a background chat's dialog still asks for the user"
    );
    let titles: Vec<String> = state
        .chats
        .dialogs(&chat("a"))
        .iter()
        .map(|dialog| match dialog {
            gentle_core::chat::Dialog::Confirm { title, .. } => title.clone(),
            other => panic!("not a confirm dialog: {other:?}"),
        })
        .collect();
    assert_eq!(titles, ["first", "second"], "queued, in arrival order");
    assert!(state.chats.dialogs(&chat("b")).is_empty());
}

#[test]
fn working_count_counts_the_chats_running_a_turn() {
    let (state, _) = apply(
        AppState::default(),
        vec![
            event(&chat("a"), RpcEvent::AgentStart),
            event(&chat("b"), RpcEvent::AgentStart),
            event(&chat("b"), RpcEvent::AgentEnd { will_retry: None }),
        ],
    );

    assert_eq!(state.chats.working_count(), 1);
}

#[test]
fn rekey_moves_the_chat_and_the_foreground_follows_it() {
    let (state, _) = apply(
        AppState::default(),
        vec![Input::Intent(Intent::OpenNewChat)],
    );
    let fresh = state
        .chats
        .foreground()
        .cloned()
        .expect("the new chat is the foreground one");
    let (mut state, _) = apply(state, vec![event(&fresh, dialog("d1", "quit?"))]);

    state
        .chats
        .rekey(&fresh, ChatId::of_session("/sessions/s1.jsonl"));

    let saved = ChatId::of_session("/sessions/s1.jsonl");
    assert_eq!(state.chats.foreground(), Some(&saved));
    assert_eq!(
        state.chats.activity(&saved),
        Some(ChatActivity::NeedsAttention)
    );
    assert!(state.chats.state(&fresh).is_none(), "the old id is gone");
    assert_eq!(state.chats.len(), 1);
}

#[test]
fn the_history_of_a_stored_chat_fills_its_own_thread() {
    const HISTORY: &str = r#"{"type":"response","command":"get_messages","success":true,"data":{"messages":[{"role":"system","content":""},{"role":"user","content":[{"type":"text","text":"hola"}]},{"role":"assistant","content":[{"type":"text","text":"ey"}]}]}}"#;
    let Decoded::Event(history) = decode_line(HISTORY) else {
        panic!("history is an event");
    };

    let (state, _) = apply(
        AppState::default(),
        vec![
            open(&chat("a")),
            open(&chat("b")),
            event(&chat("a"), history),
        ],
    );

    let texts: Vec<_> = state
        .chats
        .state(&chat("a"))
        .expect("chat a")
        .messages
        .iter()
        .map(|message| (message.role, message.text.as_str()))
        .collect();
    assert_eq!(
        texts,
        [
            (gentle_core::chat::ChatRole::User, "hola"),
            (gentle_core::chat::ChatRole::Assistant, "ey"),
        ]
    );
    assert!(
        state
            .chats
            .state(&chat("b"))
            .is_none_or(|b| b.messages.is_empty()),
        "the history belongs to one chat"
    );
}

mod chip {
    use super::*;
    use gentle_core::rpc::RpcCommand;

    /// The chip belongs to the foreground chat's child: its replies fold into
    /// the app state through the per-chat path too (0.1.11).
    #[test]
    fn the_foreground_childs_model_reply_fills_the_chip() {
        let chat = chat("a");
        let (state, _) = apply(AppState::default(), vec![open(&chat), confirmed(&chat)]);
        let (state, _) = apply(
            state,
            vec![event(
                &chat,
                RpcEvent::Response(gentle_core::rpc::RpcResponse {
                    id: None,
                    command: "get_available_models".into(),
                    outcome: Ok(serde_json::json!({
                        "models": [{"id": "m1", "name": "Model One", "provider": "prov", "reasoning": true}]
                    })),
                }),
            )],
        );
        assert_eq!(state.models.len(), 1);
        assert_eq!(state.models[0].name, "Model One");
        assert_eq!(state.models_status, gentle_app::ModelsStatus::Ready);
    }

    /// A background chat's reply must not move the foreground chip.
    #[test]
    fn a_background_childs_reply_leaves_the_chip_alone() {
        let a = chat("a");
        let b = chat("b");
        let (state, _) = apply(AppState::default(), vec![open(&a), confirmed(&a)]);
        let (state, _) = apply(
            state,
            vec![event(
                &b,
                RpcEvent::Response(gentle_core::rpc::RpcResponse {
                    id: None,
                    command: "get_available_models".into(),
                    outcome: Ok(serde_json::json!({"models": []})),
                }),
            )],
        );
        assert!(state.models.is_empty());
        assert_eq!(state.models_status, gentle_app::ModelsStatus::Unknown);
    }

    /// Choosing a model sends it to the foreground chat's child, not to the
    /// one-child send the chats shell drops.
    #[test]
    fn choosing_a_model_reaches_the_foreground_chat() {
        let chat = chat("a");
        let (state, effects) = apply(
            AppState::default(),
            vec![
                open(&chat),
                confirmed(&chat),
                Input::Intent(Intent::SelectModel {
                    provider: "prov".into(),
                    model_id: "m1".into(),
                }),
            ],
        );
        assert_eq!(
            effects.last().unwrap().chat_commands,
            vec![gentle_app::ChatCommand::Send {
                chat,
                rpc: RpcCommand::SetModel {
                    provider: "prov".into(),
                    model_id: "m1".into(),
                },
            }]
        );
        let _ = state;
    }

    #[test]
    fn choosing_a_level_applies_it_and_reaches_the_foreground_chat() {
        let chat = chat("a");
        let (state, effects) = apply(
            AppState::default(),
            vec![
                open(&chat),
                confirmed(&chat),
                Input::Intent(Intent::SelectThinkingLevel {
                    level: "high".into(),
                }),
            ],
        );
        assert_eq!(state.thinking_level.as_deref(), Some("high"));
        assert_eq!(
            effects.last().unwrap().chat_commands,
            vec![gentle_app::ChatCommand::Send {
                chat,
                rpc: RpcCommand::SetThinkingLevel {
                    level: "high".into()
                },
            }]
        );
    }

    #[test]
    fn asking_for_the_list_marks_it_loading_and_reaches_the_child() {
        let chat = chat("a");
        let (state, effects) = apply(
            AppState::default(),
            vec![
                open(&chat),
                confirmed(&chat),
                Input::Intent(Intent::RequestModels),
            ],
        );
        assert_eq!(state.models_status, gentle_app::ModelsStatus::Loading);
        assert_eq!(
            effects.last().unwrap().chat_commands,
            vec![gentle_app::ChatCommand::Send {
                chat,
                rpc: RpcCommand::GetAvailableModels,
            }]
        );
    }
}
