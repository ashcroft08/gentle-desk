//! The model chip's data through the app's public interface: the model list,
//! the session's current model and thinking level, and their failure states.

use gentle_app::{update, AppState, Command, Input, Intent, NoticeLevel};
use gentle_core::rpc::{decode_line, Decoded};

fn event(line: &str) -> Input {
    match decode_line(line) {
        Decoded::Event(event) => Input::Rpc(event),
        Decoded::Unknown(raw) => panic!("not an event: {raw}"),
    }
}

fn intent(intent: Intent) -> Input {
    Input::Intent(intent)
}

mod list {
    use super::*;

    /// pi replies `{ models: Model[] }`; a model carries id, name, provider
    /// and the `reasoning` flag (pi rpc-types.d.ts Model fields).
    #[test]
    fn the_reply_fills_the_list_and_marks_it_ready() {
        let (state, _) = update(
            AppState::default(),
            event(
                r#"{"type":"response","command":"get_available_models","success":true,
                    "data":{"models":[
                        {"id":"m1","name":"Model One","provider":"prov","reasoning":true},
                        {"id":"m2","name":"Model Two","provider":"prov","reasoning":false}
                    ]}}"#,
            ),
        );
        assert_eq!(state.models.len(), 2);
        let first = &state.models[0];
        assert_eq!(first.provider, "prov");
        assert_eq!(first.model_id, "m1");
        assert_eq!(first.name, "Model One");
        assert!(first.reasoning);
        assert!(!state.models[1].reasoning);
        assert!(matches!(
            state.models_status,
            gentle_app::ModelsStatus::Ready
        ));
    }

    #[test]
    fn an_empty_list_is_a_state_of_its_own_not_an_error() {
        let (state, _) = update(
            AppState::default(),
            event(
                r#"{"type":"response","command":"get_available_models","success":true,"data":{"models":[]}}"#,
            ),
        );
        assert!(state.models.is_empty());
        assert!(matches!(
            state.models_status,
            gentle_app::ModelsStatus::Empty
        ));
    }

    #[test]
    fn a_failed_list_reply_is_a_state_and_a_notice_not_a_crash() {
        let (state, _) = update(
            AppState::default(),
            event(
                r#"{"type":"response","command":"get_available_models","success":false,"error":"child gone"}"#,
            ),
        );
        assert!(state.models.is_empty());
        assert!(matches!(
            state.models_status,
            gentle_app::ModelsStatus::Failed(_)
        ));
        assert!(state.notices.iter().any(|n| n.level == NoticeLevel::Error));
    }

    #[test]
    fn asking_for_the_list_sends_get_available_models() {
        let (state, commands) = update(AppState::default(), intent(Intent::RequestModels));
        assert_eq!(
            commands,
            vec![Command::Send(
                gentle_core::rpc::RpcCommand::GetAvailableModels
            )]
        );
        assert!(matches!(
            state.models_status,
            gentle_app::ModelsStatus::Loading
        ));
    }
}

mod current {
    use super::*;

    const BEFORE: &str =
        r#"{"type":"response","command":"set_model","success":false,"error":"ignored"}"#;

    /// `set_model` answers with the Model object it switched to.
    #[test]
    fn a_successful_set_model_becomes_the_chips_model() {
        let (state, _) = update(
            AppState::default(),
            event(
                r#"{"type":"response","command":"set_model","success":true,
                    "data":{"id":"m1","name":"Model One","provider":"prov","reasoning":true}}"#,
            ),
        );
        let model = state.model.expect("the chip knows the model");
        assert_eq!(model.provider, "prov");
        assert_eq!(model.model_id, "m1");
        assert_eq!(model.name, "Model One");
    }

    /// The old model stays on the chip: the failed command changed nothing.
    #[test]
    fn a_failed_set_model_keeps_the_previous_model() {
        let (state, _) = update(
            AppState::default(),
            event(
                r#"{"type":"response","command":"set_model","success":true,
                    "data":{"id":"m1","name":"Model One","provider":"prov"}}"#,
            ),
        );
        let (state, _) = update(state, event(BEFORE));
        let model = state.model.expect("the previous model survives");
        assert_eq!(model.model_id, "m1");
    }

    /// `get_state` reports the session's model, so a chat opened from the
    /// sidebar shows it without a new turn.
    #[test]
    fn get_state_carries_the_current_model() {
        let (state, _) = update(
            AppState::default(),
            event(
                r#"{"type":"response","command":"get_state","success":true,
                    "data":{"sessionId":"s","model":{"id":"m2","name":"Model Two","provider":"prov"},"thinkingLevel":"high"}}"#,
            ),
        );
        let model = state.model.expect("the chip knows the model");
        assert_eq!(model.model_id, "m2");
        assert_eq!(model.name, "Model Two");
        assert_eq!(state.thinking_level.as_deref(), Some("high"));
    }

    #[test]
    fn choosing_a_model_sends_set_model_and_the_reply_updates_the_chip() {
        let (state, commands) = update(
            AppState::default(),
            intent(Intent::SelectModel {
                provider: "prov".into(),
                model_id: "m2".into(),
            }),
        );
        assert_eq!(
            commands,
            vec![Command::Send(gentle_core::rpc::RpcCommand::SetModel {
                provider: "prov".into(),
                model_id: "m2".into(),
            })]
        );
        let (state, _) = update(
            state,
            event(
                r#"{"type":"response","command":"set_model","success":true,
                    "data":{"id":"m2","name":"Model Two","provider":"prov"}}"#,
            ),
        );
        assert_eq!(state.model.expect("applied").model_id, "m2");
    }

    /// The new model owns the levels list and the current level: the reply
    /// queues the two refills (R3-002).
    #[test]
    fn a_successful_set_model_asks_for_the_new_models_levels() {
        let (state, _) = update(
            AppState::default(),
            event(
                r#"{"type":"response","command":"set_model","success":true,
                    "data":{"id":"m2","name":"Model Two","provider":"prov"}}"#,
            ),
        );
        let (_, commands) = update(
            state,
            event(
                r#"{"type":"response","command":"get_state","success":true,
                    "data":{"model":{"id":"m2","name":"Model Two","provider":"prov"},"thinkingLevel":"low"}}"#,
            ),
        );
        assert!(commands.is_empty(), "only set_model queues the refill");

        let (state, _) = update(
            AppState::default(),
            event(
                r#"{"type":"response","command":"set_model","success":true,
                    "data":{"id":"m2","name":"Model Two","provider":"prov"}}"#,
            ),
        );
        let (_, commands) = update(
            state,
            event(
                r#"{"type":"response","command":"set_model","success":true,
                    "data":{"id":"m3","name":"Model Three","provider":"prov"}}"#,
            ),
        );
        assert_eq!(
            commands,
            vec![
                Command::Send(gentle_core::rpc::RpcCommand::GetState),
                Command::Send(gentle_core::rpc::RpcCommand::GetAvailableThinkingLevels),
            ]
        );
    }

    /// The chip's data belongs to one child: opening another chat leaves the
    /// chip blank until that chat's child answers — and the switch asks
    /// again, so the blank is a moment, not a state (R3-002).
    #[test]
    fn opening_a_session_clears_the_chip_and_asks_again() {
        let (state, _) = update(
            AppState::default(),
            event(
                r#"{"type":"response","command":"get_available_models","success":true,
                    "data":{"models":[{"id":"m1","name":"Model One","provider":"prov"}]}}"#,
            ),
        );
        let (state, _) = update(
            state,
            event(
                r#"{"type":"response","command":"set_model","success":true,
                    "data":{"id":"m1","name":"Model One","provider":"prov"}}"#,
            ),
        );
        let (state, commands) = update(state, intent(Intent::OpenSession("s.jsonl".into())));
        assert!(state.models.is_empty());
        assert!(matches!(
            state.models_status,
            gentle_app::ModelsStatus::Unknown
        ));
        assert!(state.model.is_none());
        assert_eq!(state.thinking_level, None);
        assert!(state.thinking_levels.is_empty());
        // The switch itself carries the history ask; the refill goes out after
        // the session answers it (events.rs).
        assert_eq!(
            commands,
            vec![Command::Send(gentle_core::rpc::RpcCommand::SwitchSession {
                session_path: "s.jsonl".into(),
            })]
        );

        let (state, commands) = update(
            state,
            event(r#"{"type":"response","command":"switch_session","success":true,"data":{}}"#),
        );
        assert_eq!(state.current_session.as_deref(), Some("s.jsonl"));
        assert_eq!(
            commands,
            vec![
                Command::Send(gentle_core::rpc::RpcCommand::GetMessages { id: None }),
                Command::Send(gentle_core::rpc::RpcCommand::GetState),
                Command::Send(gentle_core::rpc::RpcCommand::GetAvailableModels),
                Command::Send(gentle_core::rpc::RpcCommand::GetAvailableThinkingLevels),
            ]
        );
    }
}

mod thinking {
    use super::*;

    /// The reply carries no data, so the level is set when the command goes
    /// out and dropped to unknown when it fails (the chip shows nothing
    /// rather than a level the session may not have).
    #[test]
    fn choosing_a_level_sends_it_and_applies_it_at_once() {
        let (state, commands) = update(
            AppState::default(),
            intent(Intent::SelectThinkingLevel {
                level: "high".into(),
            }),
        );
        assert_eq!(
            commands,
            vec![Command::Send(
                gentle_core::rpc::RpcCommand::SetThinkingLevel {
                    level: "high".into(),
                }
            )]
        );
        assert_eq!(state.thinking_level.as_deref(), Some("high"));
    }

    #[test]
    fn a_failed_level_change_leaves_the_level_unknown_and_notices() {
        let (state, _) = update(
            AppState::default(),
            intent(Intent::SelectThinkingLevel {
                level: "high".into(),
            }),
        );
        let (state, _) = update(
            state,
            event(
                r#"{"type":"response","command":"set_thinking_level","success":false,"error":"bad level"}"#,
            ),
        );
        assert_eq!(state.thinking_level, None);
        assert!(state.notices.iter().any(|n| n.level == NoticeLevel::Error));
    }

    /// `get_available_thinking_levels` answers the levels the session's model
    /// supports, as `{ levels: [...] }` (rpc-types.d.ts).
    #[test]
    fn the_levels_list_is_parsed_from_the_reply() {
        let (state, _) = update(
            AppState::default(),
            event(
                r#"{"type":"response","command":"get_available_thinking_levels","success":true,
                    "data":{"levels":["off","low","medium","high"]}}"#,
            ),
        );
        assert_eq!(state.thinking_levels, vec!["off", "low", "medium", "high"]);
    }

    /// A bare array is accepted too, so a shape drift cannot empty the list
    /// silently.
    #[test]
    fn a_bare_array_of_levels_is_accepted_as_well() {
        let (state, _) = update(
            AppState::default(),
            event(
                r#"{"type":"response","command":"get_available_thinking_levels","success":true,
                    "data":["off","high"]}"#,
            ),
        );
        assert_eq!(state.thinking_levels, vec!["off", "high"]);
    }
}
