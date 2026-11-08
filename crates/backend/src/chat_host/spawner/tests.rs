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

    let options = spawner().options_for(&saved);

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
fn a_new_chat_carries_no_session_flag() {
    assert_eq!(
        spawner().options_for(&ChatKey::New(7)).args,
        Vec::<OsString>::new()
    );
}

#[test]
fn every_child_is_told_it_has_an_interactive_host() {
    let options = spawner().options_for(&ChatKey::New(1));

    assert_eq!(
        options.env,
        vec![(
            OsString::from("GENTLE_SHELL_INTERACTIVE_HOST"),
            OsString::from("1")
        )]
    );
}
