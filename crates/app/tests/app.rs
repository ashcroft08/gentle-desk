//! Behaviour of the app layer through its public interface only.

use gentle_app::{update, AppState, Command, Input, Intent, NoticeLevel};
use gentle_core::chat::{ChatRole, Dialog};
use gentle_core::rpc::{decode_line, Decoded, RpcCommand, UiAnswer};

fn event(line: &str) -> Input {
    match decode_line(line) {
        Decoded::Event(event) => Input::Rpc(event),
        Decoded::Unknown(raw) => panic!("not an event: {raw}"),
    }
}

fn intent(intent: Intent) -> Input {
    Input::Intent(intent)
}

fn run(state: AppState, inputs: Vec<Input>) -> (AppState, Vec<Command>) {
    inputs
        .into_iter()
        .fold((state, vec![]), |(state, mut all), input| {
            let (next, commands) = update(state, input);
            all.extend(commands);
            (next, all)
        })
}

const START: &str = r#"{"type":"agent_start"}"#;
const SETTLED: &str = r#"{"type":"agent_settled"}"#;
const REPLY_START: &str = r#"{"type":"message_start","message":{"role":"assistant","content":[]}}"#;
const REPLY_END: &str = r#"{"type":"message_end","message":{"role":"assistant","content":[{"type":"text","text":"**hola**"}]}}"#;

fn after_one_reply() -> AppState {
    let (state, _) = run(
        AppState::default(),
        vec![
            intent(Intent::SendPrompt("hi".into())),
            event(START),
            event(REPLY_START),
            event(REPLY_END),
            event(SETTLED),
        ],
    );
    state
}

mod prompts {
    use super::*;

    #[test]
    fn sending_a_prompt_appends_the_user_message_and_sends_the_command() {
        let (state, commands) = update(
            AppState::default(),
            intent(Intent::SendPrompt("hola".into())),
        );
        assert_eq!(state.chat.messages.len(), 1);
        assert_eq!(state.chat.messages[0].role, ChatRole::User);
        assert_eq!(state.chat.messages[0].text, "hola");
        assert_eq!(state.chat.messages[0].id, "msg-0");
        assert_eq!(
            commands,
            vec![Command::Send(RpcCommand::Prompt {
                message: "hola".into()
            })]
        );
    }

    #[test]
    fn a_blank_prompt_does_nothing() {
        let (state, commands) = update(
            AppState::default(),
            intent(Intent::SendPrompt("  \n".into())),
        );
        assert_eq!(state, AppState::default());
        assert!(commands.is_empty());
    }

    #[test]
    fn a_prompt_while_the_agent_works_is_ignored() {
        let (state, _) = update(AppState::default(), event(START));
        let (after, commands) = update(state.clone(), intent(Intent::SendPrompt("otra".into())));
        assert_eq!(after, state);
        assert!(commands.is_empty());
    }

    #[test]
    fn ids_stay_contiguous_across_user_and_assistant_messages() {
        let state = after_one_reply();
        let ids: Vec<_> = state.chat.messages.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids, ["msg-0", "msg-1"]);
        let (state, _) = update(state, intent(Intent::SendPrompt("dos".into())));
        assert_eq!(state.chat.messages[2].id, "msg-2");
    }

    #[test]
    fn rpc_events_still_fold_into_the_chat() {
        let state = after_one_reply();
        assert_eq!(state.chat.messages[1].text, "**hola**");
        assert!(!state.chat.working);
    }

    #[test]
    fn abort_sends_the_abort_command() {
        let (_, commands) = update(AppState::default(), intent(Intent::Abort));
        assert_eq!(commands, vec![Command::Send(RpcCommand::Abort)]);
    }
}

mod new_chat {
    use super::*;

    #[test]
    fn new_chat_clears_the_thread_and_asks_for_a_new_session() {
        let (state, commands) = update(after_one_reply(), intent(Intent::NewChat));
        assert_eq!(state, AppState::default());
        assert_eq!(commands, vec![Command::Send(RpcCommand::NewSession)]);
    }

    /// The reference app's "New chat" did nothing when the state was already
    /// "new" (reference equality); every request must reach the backend.
    #[test]
    fn new_chat_on_an_empty_chat_still_sends_the_command_each_time() {
        let (state, first) = update(AppState::default(), intent(Intent::NewChat));
        let (_, second) = update(state, intent(Intent::NewChat));
        assert_eq!(first, vec![Command::Send(RpcCommand::NewSession)]);
        assert_eq!(second, first);
    }

    #[test]
    fn new_chat_while_working_aborts_first() {
        let (state, _) = update(AppState::default(), event(START));
        let (state, commands) = update(state, intent(Intent::NewChat));
        assert!(!state.chat.working);
        assert_eq!(
            commands,
            vec![
                Command::Send(RpcCommand::Abort),
                Command::Send(RpcCommand::NewSession)
            ]
        );
    }
}

mod notices {
    use super::*;

    const NOTIFY: &str = r#"{"type":"extension_ui_request","id":"1","method":"notify","message":"rate limit reached","notifyType":"warning"}"#;

    #[test]
    fn a_notify_becomes_a_notice_with_level_and_id() {
        let (state, commands) = update(AppState::default(), event(NOTIFY));
        assert_eq!(state.notices.len(), 1);
        assert_eq!(state.notices[0].level, NoticeLevel::Warning);
        assert_eq!(state.notices[0].text, "rate limit reached");
        assert_eq!(
            state.notices[0].action, None,
            "an extension notice asks for nothing: only a chat's death carries a Restart (S3)"
        );
        assert!(commands.is_empty());
    }

    #[test]
    fn missing_or_unknown_levels_are_info() {
        let bare = r#"{"type":"extension_ui_request","id":"2","method":"notify","message":"hi"}"#;
        let odd = r#"{"type":"extension_ui_request","id":"3","method":"notify","message":"yo","notifyType":"weird"}"#;
        let (state, _) = run(AppState::default(), vec![event(bare), event(odd)]);
        assert!(state.notices.iter().all(|n| n.level == NoticeLevel::Info));
    }

    #[test]
    fn other_fire_and_forget_requests_are_not_notices() {
        let status = r#"{"type":"extension_ui_request","id":"4","method":"setStatus","statusKey":"x","statusText":"y"}"#;
        let (state, _) = update(AppState::default(), event(status));
        assert!(state.notices.is_empty());
    }

    fn notify(text: &str) -> Input {
        event(&format!(
            r#"{{"type":"extension_ui_request","id":"n","method":"notify","message":"{text}","notifyType":"warning"}}"#
        ))
    }

    #[test]
    fn an_identical_notice_already_on_screen_is_not_repeated() {
        let (state, _) = run(AppState::default(), vec![event(NOTIFY), event(NOTIFY)]);
        assert_eq!(state.notices.len(), 1);
        // Once dismissed it may come back: it is news again.
        let id = state.notices[0].id;
        let (state, _) = update(state, intent(Intent::DismissNotice(id)));
        assert!(state.notices.is_empty());
        let (state, _) = update(state, event(NOTIFY));
        assert_eq!(state.notices.len(), 1);
    }

    #[test]
    fn dismissing_removes_only_that_notice_and_ids_are_never_reused() {
        let (state, _) = run(AppState::default(), vec![notify("a"), notify("b")]);
        let (first, second) = (state.notices[0].id, state.notices[1].id);
        assert_ne!(first, second);
        let (state, _) = update(state, intent(Intent::DismissNotice(first)));
        assert_eq!(state.notices.len(), 1);
        assert_eq!(state.notices[0].id, second);
        let (state, _) = update(state, notify("c"));
        assert!(state.notices.iter().all(|n| n.id != first));
    }

    #[test]
    fn only_the_latest_fifty_notices_are_kept() {
        let inputs = (0..60).map(|n| notify(&format!("n{n}"))).collect();
        let (state, _) = run(AppState::default(), inputs);
        assert_eq!(state.notices.len(), 50);
        assert_eq!(state.notices.last().unwrap().id, 59);
    }

    #[test]
    fn new_chat_clears_notices() {
        let (state, _) = update(AppState::default(), event(NOTIFY));
        let (state, _) = update(state, intent(Intent::NewChat));
        assert!(state.notices.is_empty());
    }
}

mod copy_and_dialogs {
    use super::*;

    #[test]
    fn copying_a_message_yields_its_raw_markdown() {
        let (_, commands) = update(
            after_one_reply(),
            intent(Intent::CopyMessage("msg-1".into())),
        );
        assert_eq!(commands, vec![Command::CopyToClipboard("**hola**".into())]);
    }

    #[test]
    fn copying_an_unknown_message_does_nothing() {
        let (_, commands) = update(
            after_one_reply(),
            intent(Intent::CopyMessage("msg-9".into())),
        );
        assert!(commands.is_empty());
    }

    const CONFIRM: &str = r#"{"type":"extension_ui_request","id":"d1","method":"confirm","title":"Sure?","message":"really"}"#;

    #[test]
    fn answering_a_dialog_sends_the_response_and_removes_it() {
        let (state, _) = update(AppState::default(), event(CONFIRM));
        assert!(matches!(
            state.chat.pending_dialogs[0],
            Dialog::Confirm { .. }
        ));
        let (state, commands) = update(
            state,
            intent(Intent::AnswerDialog {
                id: "d1".into(),
                answer: UiAnswer::Confirmed(true),
            }),
        );
        assert!(state.chat.pending_dialogs.is_empty());
        assert_eq!(
            commands,
            vec![Command::Send(RpcCommand::UiResponse {
                id: "d1".into(),
                answer: UiAnswer::Confirmed(true)
            })]
        );
    }

    #[test]
    fn answering_an_unknown_dialog_does_nothing() {
        let (state, commands) = update(
            AppState::default(),
            intent(Intent::AnswerDialog {
                id: "ghost".into(),
                answer: UiAnswer::Cancelled,
            }),
        );
        assert_eq!(state, AppState::default());
        assert!(commands.is_empty());
    }

    #[test]
    fn unrelated_events_do_not_touch_notices() {
        let (state, _) = update(AppState::default(), event(START));
        assert!(state.notices.is_empty());
    }
}

mod sessions {
    use super::*;
    use gentle_app::SessionItem;

    fn item(path: &str, title: &str) -> SessionItem {
        SessionItem {
            path: path.into(),
            title: title.into(),
            cwd: String::new(),
            modified_ms: 1,
            message_count: 2,
        }
    }

    const SWITCH_OK: &str = r#"{"type":"response","command":"switch_session","success":true,"data":{"cancelled":false}}"#;
    const HISTORY: &str = r#"{"type":"response","command":"get_messages","success":true,"data":{"messages":[{"role":"system","content":""},{"role":"user","content":[{"type":"text","text":"hola"}]},{"role":"assistant","content":[{"type":"text","text":"**ey**"}]}]}}"#;

    #[test]
    fn refresh_asks_the_shell_to_reload_the_list() {
        let (_, commands) = update(AppState::default(), intent(Intent::RefreshSessions));
        assert_eq!(commands, vec![Command::RefreshSessions]);
    }

    #[test]
    fn a_loaded_list_replaces_the_previous_one() {
        let (state, _) = update(
            AppState::default(),
            Input::SessionsLoaded(vec![item("a", "A")]),
        );
        let (state, commands) = update(
            state,
            Input::SessionsLoaded(vec![item("b", "B"), item("c", "C")]),
        );
        assert_eq!(
            state
                .sessions
                .iter()
                .map(|s| s.path.as_str())
                .collect::<Vec<_>>(),
            ["b", "c"]
        );
        assert!(commands.is_empty());
    }

    #[test]
    fn a_settled_turn_refreshes_the_list() {
        let (_, commands) = update(AppState::default(), event(SETTLED));
        assert_eq!(commands, vec![Command::RefreshSessions]);
    }

    #[test]
    fn opening_a_session_clears_the_thread_and_switches() {
        let (state, commands) = update(
            after_one_reply(),
            intent(Intent::OpenSession("C:/s/a.jsonl".into())),
        );
        assert!(state.chat.messages.is_empty());
        assert_eq!(state.current_session.as_deref(), Some("C:/s/a.jsonl"));
        assert_eq!(
            commands,
            vec![Command::Send(RpcCommand::SwitchSession {
                session_path: "C:/s/a.jsonl".into()
            })]
        );
    }

    #[test]
    fn opening_a_session_while_working_aborts_first() {
        let (state, _) = update(AppState::default(), event(START));
        let (state, commands) = update(state, intent(Intent::OpenSession("p".into())));
        assert!(!state.chat.working);
        assert_eq!(commands[0], Command::Send(RpcCommand::Abort));
        assert_eq!(commands.len(), 2);
    }

    #[test]
    fn a_successful_switch_asks_for_the_history() {
        let (_, commands) = update(AppState::default(), event(SWITCH_OK));
        assert_eq!(
            commands,
            vec![Command::Send(RpcCommand::GetMessages { id: None })]
        );
    }

    #[test]
    fn the_history_response_fills_the_thread() {
        let (state, _) = update(AppState::default(), event(HISTORY));
        let texts: Vec<_> = state
            .chat
            .messages
            .iter()
            .map(|m| (m.role, m.text.as_str()))
            .collect();
        assert_eq!(
            texts,
            [(ChatRole::User, "hola"), (ChatRole::Assistant, "**ey**")]
        );
    }

    #[test]
    fn a_failed_switch_is_an_error_notice_and_forgets_the_current_session() {
        let (state, _) = update(AppState::default(), intent(Intent::OpenSession("p".into())));
        let failed = r#"{"type":"response","command":"switch_session","success":false,"error":"EISDIR: nope"}"#;
        let (state, commands) = update(state, event(failed));
        assert_eq!(state.current_session, None);
        assert_eq!(state.notices.len(), 1);
        assert_eq!(state.notices[0].level, NoticeLevel::Error);
        assert!(
            state.notices[0].text.contains("switch_session")
                && state.notices[0].text.contains("EISDIR")
        );
        assert!(commands.is_empty());
    }

    #[test]
    fn new_chat_keeps_the_session_list_but_forgets_the_current_one() {
        let (state, _) = update(
            AppState::default(),
            Input::SessionsLoaded(vec![item("a", "A")]),
        );
        let (state, _) = update(state, intent(Intent::OpenSession("a".into())));
        let (state, _) = update(state, intent(Intent::NewChat));
        assert_eq!(state.sessions.len(), 1);
        assert_eq!(state.current_session, None);
        assert!(state.chat.messages.is_empty());
    }
}

mod helpers {
    use super::*;
    use gentle_core::helpers::HelperStatus;

    fn widget(summary: &str, tasks: &str) -> Input {
        let lines = format!(
            r#"{{"schema":"gentle-agents.activity/v1","summary":{summary},"tasks":{tasks}}}"#
        );
        let raw = serde_json::json!({
            "type": "extension_ui_request", "id": "w", "method": "setWidget",
            "widgetKey": "gentle-agents", "widgetLines": [lines]
        });
        event(&raw.to_string())
    }

    const RUNNING: &str =
        r#"[{"summary":{"id":"t1","status":"running","createdAt":"2026-09-22T10:00:00.000Z"}}]"#;
    const COUNTS_1: &str = r#"{"running":1,"queued":0,"waiting":0,"finished":0}"#;
    const COUNTS_DONE: &str = r#"{"running":0,"queued":0,"waiting":0,"finished":1}"#;

    #[test]
    fn helper_frames_reach_the_chat_state() {
        let (state, commands) = update(AppState::default(), widget(COUNTS_1, RUNNING));
        assert_eq!(state.chat.helpers.summary.running, 1);
        assert_eq!(state.chat.helpers.tasks[0].id, "t1");
        assert!(commands.is_empty());
    }

    #[test]
    fn the_arrival_time_stamps_a_task_that_gentle_agents_stops_reporting() {
        let (state, _) = update(AppState::default(), widget(COUNTS_1, RUNNING));
        let Input::Rpc(dropped) = widget(COUNTS_DONE, "[]") else {
            panic!("rpc input")
        };
        let (state, _) = update(state, Input::RpcAt(dropped, 1_790_074_800_000));
        let task = &state.chat.helpers.tasks[0];
        assert_eq!(
            (task.status, task.ended_at_ms),
            (HelperStatus::Done, Some(1_790_074_800_000))
        );
    }

    #[test]
    fn rpc_at_behaves_like_rpc_for_everything_else() {
        let Input::Rpc(start) = event(START) else {
            panic!("rpc input")
        };
        let (state, _) = update(AppState::default(), Input::RpcAt(start, 5));
        assert!(state.chat.working);
    }

    #[test]
    fn a_new_chat_and_opening_a_session_forget_the_helpers() {
        let (state, _) = update(AppState::default(), widget(COUNTS_1, RUNNING));
        let (fresh, _) = update(state.clone(), intent(Intent::NewChat));
        assert!(fresh.chat.helpers.tasks.is_empty());
        let (opened, _) = update(state, intent(Intent::OpenSession("p".into())));
        assert!(opened.chat.helpers.tasks.is_empty());
    }
}

mod helper_selection {
    use super::*;

    fn with_helper() -> AppState {
        let lines = r#"{"schema":"gentle-agents.activity/v1","summary":{"running":1},"tasks":[{"summary":{"id":"t1","status":"running","createdAt":"2026-09-22T10:00:00.000Z"}}]}"#;
        let raw = serde_json::json!({
            "type": "extension_ui_request", "id": "w", "method": "setWidget",
            "widgetKey": "gentle-agents", "widgetLines": [lines]
        });
        update(AppState::default(), event(&raw.to_string())).0
    }

    #[test]
    fn selecting_a_known_helper_remembers_it() {
        let (state, commands) = update(with_helper(), intent(Intent::SelectHelper("t1".into())));
        assert_eq!(state.selected_helper.as_deref(), Some("t1"));
        assert!(commands.is_empty());
    }

    #[test]
    fn selecting_an_unknown_helper_does_nothing() {
        let (state, _) = update(with_helper(), intent(Intent::SelectHelper("ghost".into())));
        assert_eq!(state.selected_helper, None);
    }

    #[test]
    fn closing_goes_back_to_the_list() {
        let (state, _) = update(with_helper(), intent(Intent::SelectHelper("t1".into())));
        let (state, _) = update(state, intent(Intent::CloseHelper));
        assert_eq!(state.selected_helper, None);
    }

    #[test]
    fn a_new_chat_and_opening_a_session_drop_the_selection() {
        let (selected, _) = update(with_helper(), intent(Intent::SelectHelper("t1".into())));
        let (fresh, _) = update(selected.clone(), intent(Intent::NewChat));
        assert_eq!(fresh.selected_helper, None);
        let (opened, _) = update(selected, intent(Intent::OpenSession("p".into())));
        assert_eq!(opened.selected_helper, None);
    }
}
