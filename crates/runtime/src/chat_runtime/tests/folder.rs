//! The folder an open names (W2's rule, carried to the runtime): a child starts
//! in the folder of its open, and an open that cannot start one there is a
//! failure the shell shows, never a chat that quietly starts somewhere else.

use std::fs;
use std::path::{Path, PathBuf};

use super::*;

#[test]
fn the_plain_open_names_no_folder() {
    let (runtime, spawner) = runtime();
    let chat = ChatId::of_session("s.jsonl");

    runtime.execute(open(&chat));

    assert_eq!(spawner.folders(), vec![None::<PathBuf>]);
}

#[test]
fn an_open_that_names_a_folder_starts_the_child_there() {
    let dir = tempfile::tempdir().unwrap();
    let folder = folder_of(&dir.path().join("repo"));
    let (runtime, spawner) = runtime();
    let chat = ChatId::of_session("s.jsonl");

    let outcome = runtime.execute_in(open(&chat), Some(folder.clone()));

    assert_eq!(
        outcome,
        Outcome::Opened {
            chat,
            how: Opened::Started
        }
    );
    assert_eq!(
        spawner.folders(),
        vec![Some(folder)],
        "the child starts in the folder its open named"
    );
}

#[test]
fn a_folder_that_is_not_there_fails_the_open_and_starts_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("gone");
    let (runtime, spawner) = runtime();
    let chat = ChatId::of_session("s.jsonl");

    let outcome = runtime.execute_in(open(&chat), Some(missing.clone()));

    let Outcome::OpenFailed {
        chat: failed,
        message,
    } = outcome
    else {
        panic!("expected a failed open, got {outcome:?}");
    };
    assert_eq!(failed, chat);
    assert!(
        message.contains("no folder at"),
        "the host's own reason reaches the caller, got {message:?}"
    );
    assert!(
        spawner.started().is_empty(),
        "no child is started in a folder that cannot host one"
    );
}

#[test]
fn a_path_that_is_a_file_fails_the_open_the_same_way() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("note.txt");
    fs::write(&file, b"not a folder").unwrap();
    let (runtime, spawner) = runtime();
    let chat = ChatId::of_session("s.jsonl");

    let outcome = runtime.execute_in(open(&chat), Some(file));

    assert!(
        matches!(outcome, Outcome::OpenFailed { .. }),
        "got {outcome:?}"
    );
    assert!(spawner.started().is_empty());
}

/// An existing folder: the directory is created here, because a folder that is
/// not there is a refusal the host answers itself.
fn folder_of(path: &Path) -> PathBuf {
    fs::create_dir_all(path).unwrap();
    path.to_path_buf()
}
