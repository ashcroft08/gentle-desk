//! What the real spawner asks for, without starting anything.

use std::ffi::OsString;
use std::path::PathBuf;

use super::*;

fn spawner() -> ProcessSpawner {
    ProcessSpawner::new(crate::launcher::Launcher {
        program: "gentle-shell".into(),
        args: vec![],
    })
}

#[test]
fn a_saved_chat_passes_its_session_and_never_a_switch() {
    let saved = ChatKey::Saved(PathBuf::from("c:/home/sessions/a.jsonl"));

    let options = spawner().options_for(&saved, None);

    assert_eq!(
        options.args,
        vec![
            OsString::from("--session"),
            OsString::from("c:/home/sessions/a.jsonl"),
        ]
    );
    assert_eq!(options.cwd, None);
}

#[test]
fn a_folder_becomes_the_working_folder_and_no_folder_keeps_ours() {
    let folder = PathBuf::from("c:/work/project");

    assert_eq!(
        spawner().options_for(&ChatKey::New(1), Some(&folder)).cwd,
        Some(folder)
    );
    assert_eq!(spawner().options_for(&ChatKey::New(1), None).cwd, None);
}

#[test]
fn a_new_chat_carries_no_session_flag() {
    assert_eq!(
        spawner().options_for(&ChatKey::New(7), None).args,
        Vec::<OsString>::new()
    );
}

#[test]
fn forwarded_arguments_come_before_the_session_of_a_saved_chat() {
    let spawner = spawner().with_args(vec![OsString::from("--isolated")]);
    let saved = ChatKey::Saved(PathBuf::from("c:/home/sessions/a.jsonl"));

    assert_eq!(
        spawner.options_for(&saved, None).args,
        vec![
            OsString::from("--isolated"),
            OsString::from("--session"),
            OsString::from("c:/home/sessions/a.jsonl"),
        ]
    );
}

#[test]
fn a_new_chat_still_carries_the_forwarded_arguments() {
    let spawner = spawner().with_args(vec![OsString::from("--isolated")]);

    assert_eq!(
        spawner.options_for(&ChatKey::New(1), None).args,
        vec![OsString::from("--isolated")]
    );
}

/// The one intercom id `env` carries. pi's intercom extension reads it as the
/// identity of the session it holds, so a child without exactly one is a child
/// that can steal another chat's id.
fn intercom_id(env: &[(OsString, OsString)]) -> OsString {
    let var = OsString::from("PI_INTERCOM_STABLE_ID");
    let values: Vec<&OsString> = env
        .iter()
        .filter(|(name, _)| *name == var)
        .map(|(_, value)| value)
        .collect();

    assert_eq!(values.len(), 1, "one PI_INTERCOM_STABLE_ID, got {env:?}");
    values[0].clone()
}

/// The id of one chat, asked of a spawner that started no child.
fn id_of(key: &ChatKey) -> OsString {
    intercom_id(&spawner().options_for(key, None).env)
}

#[test]
fn two_chats_get_two_intercom_ids() {
    let first = ChatKey::Saved(PathBuf::from("c:/pi/sessions/a.jsonl"));
    let second = ChatKey::Saved(PathBuf::from("c:/pi/sessions/b.jsonl"));

    assert_ne!(id_of(&first), id_of(&second));
    assert_ne!(id_of(&ChatKey::New(1)), id_of(&ChatKey::New(2)));
    assert_ne!(id_of(&first), id_of(&ChatKey::New(1)));
}

#[test]
fn the_same_chat_keeps_its_intercom_id() {
    let saved = ChatKey::Saved(PathBuf::from("c:/pi/sessions/a.jsonl"));
    let new = ChatKey::New(3);

    // Asked twice of the same spawner, and again of a fresh one: the id of a
    // chat belongs to the chat, never to the spawner or the moment.
    assert_eq!(id_of(&saved), id_of(&saved));
    assert_eq!(id_of(&saved), id_of(&saved));
    assert_eq!(id_of(&new), id_of(&new));
    assert_eq!(id_of(&new), id_of(&new));
}

#[test]
fn the_intercom_id_is_ascii_and_bounded() {
    let keys = [
        // A folder whose name is not ASCII (escaped, so this file stays ASCII):
        // the id must be ASCII anyway.
        ChatKey::Saved(PathBuf::from("c:/pi/sesiones/caf\u{e9}-\u{f1}.jsonl")),
        ChatKey::Saved(PathBuf::from("c:/pi/sessions/a.jsonl")),
        ChatKey::New(9),
    ];

    for key in keys {
        let id = id_of(&key);
        let id = id.to_str().expect("the id is UTF-8");

        assert!(!id.is_empty(), "{key} got an empty id");
        assert!(id.is_ascii(), "{key} got {id:?}");
        assert!(id.len() <= 64, "{key} got {} bytes", id.len());
    }
}

#[test]
fn every_child_is_told_it_has_an_interactive_host() {
    let env = spawner().options_for(&ChatKey::New(1), None).env;

    assert_eq!(
        env.len(),
        2,
        "the interactive host and the intercom id, got {env:?}"
    );
    assert_eq!(
        env[0],
        (
            OsString::from("GENTLE_SHELL_INTERACTIVE_HOST"),
            OsString::from("1")
        )
    );
}

/// The one value of `name` in `env`, and how many entries carry it: a child can
/// only hold one, so a duplicate would be a bug the tests must see.
fn value_of(env: &[(OsString, OsString)], name: &str) -> (Option<OsString>, usize) {
    let var = OsString::from(name);
    let values: Vec<&OsString> = env
        .iter()
        .filter(|(key, _)| *key == var)
        .map(|(_, value)| value)
        .collect();
    (values.first().map(|value| (*value).clone()), values.len())
}

#[test]
fn a_child_of_an_unchanged_spawner_carries_no_mode() {
    // gentle-pi's own default is organic, so nothing is passed until the user
    // chooses one: the variable is absent, never set to a value guessed here.
    let env = spawner().options_for(&ChatKey::New(1), None).env;
    assert_eq!(value_of(&env, ORCHESTRATOR_MODE_VAR), (None, 0));
}

#[test]
fn the_switch_reaches_the_children_started_after_it() {
    let spawner = spawner();

    spawner.set_orchestrator_mode(Some(OrchestratorMode::Organic));
    let organic = spawner.options_for(&ChatKey::New(1), None).env;
    assert_eq!(
        value_of(&organic, ORCHESTRATOR_MODE_VAR),
        (Some(OsString::from("organic")), 1)
    );

    // The same spawner, asked again: the mode is per child, resolved at spawn.
    spawner.set_orchestrator_mode(Some(OrchestratorMode::FullDelegation));
    let delegated = spawner.options_for(&ChatKey::New(2), None).env;
    assert_eq!(
        value_of(&delegated, ORCHESTRATOR_MODE_VAR),
        (Some(OsString::from("full-delegation")), 1)
    );
}

#[test]
fn clearing_the_mode_puts_later_children_back_on_the_default() {
    let spawner = spawner();
    spawner.set_orchestrator_mode(Some(OrchestratorMode::FullDelegation));
    assert_eq!(
        spawner.orchestrator_mode(),
        Some(OrchestratorMode::FullDelegation)
    );

    spawner.set_orchestrator_mode(None);

    assert_eq!(spawner.orchestrator_mode(), None);
    let env = spawner.options_for(&ChatKey::New(3), None).env;
    assert_eq!(value_of(&env, ORCHESTRATOR_MODE_VAR), (None, 0));
}

#[test]
fn the_mode_replaces_an_entry_the_environment_already_had() {
    // The caller's own environment may name the variable (a developer set it for
    // the whole run); the switch the user made is the one the child gets, once.
    let spawner = ProcessSpawner::with_env(
        crate::launcher::Launcher {
            program: "gentle-shell".into(),
            args: vec![],
        },
        vec![
            (
                OsString::from("GENTLE_SHELL_INTERACTIVE_HOST"),
                OsString::from("1"),
            ),
            (
                OsString::from(ORCHESTRATOR_MODE_VAR),
                OsString::from("organic"),
            ),
        ],
    );
    spawner.set_orchestrator_mode(Some(OrchestratorMode::FullDelegation));

    let env = spawner.options_for(&ChatKey::New(4), None).env;

    assert_eq!(
        value_of(&env, ORCHESTRATOR_MODE_VAR),
        (Some(OsString::from("full-delegation")), 1)
    );
    assert_eq!(
        env.len(),
        3,
        "interactive host, intercom id and one mode: {env:?}"
    );
}

#[test]
fn the_switch_is_never_set_in_this_process() {
    // Session-scoped and per child (S1): the spawner carries the mode, and the
    // process that starts the children keeps its own environment untouched.
    let before = std::env::var_os(ORCHESTRATOR_MODE_VAR);
    let spawner = spawner();
    spawner.set_orchestrator_mode(Some(OrchestratorMode::FullDelegation));
    let _ = spawner.options_for(&ChatKey::New(5), None);
    assert_eq!(std::env::var_os(ORCHESTRATOR_MODE_VAR), before);
}

#[test]
fn the_two_modes_are_the_two_values_gentle_pi_reads() {
    assert_eq!(OrchestratorMode::Organic.as_str(), "organic");
    assert_eq!(OrchestratorMode::FullDelegation.as_str(), "full-delegation");
    assert_eq!(
        OrchestratorMode::parse("full-delegation"),
        Some(OrchestratorMode::FullDelegation)
    );
    assert_eq!(
        OrchestratorMode::parse("organic"),
        Some(OrchestratorMode::Organic)
    );
    assert_eq!(OrchestratorMode::parse("delegation"), None);
    assert_eq!(OrchestratorMode::parse(""), None);
}
