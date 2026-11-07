//! The classifier: one rule per case, with events built by hand.

use std::path::PathBuf;

use notify::event::{AccessKind, CreateKind, DataChange, Flag, MetadataKind};
use notify::event::{ModifyKind, RemoveKind, RenameMode};
use notify::Event;

use super::*;

/// An absolute sessions directory that exists on every platform.
fn sessions() -> PathBuf {
    std::env::temp_dir().join("gentle-watch-classify")
}

fn in_project(name: &str) -> PathBuf {
    sessions().join("--home-me--").join(name)
}

fn event(kind: EventKind, paths: &[PathBuf]) -> Event {
    let mut event = Event::new(kind);
    for path in paths {
        event = event.add_path(path.clone());
    }
    event
}

fn classify_paths(kind: EventKind, paths: &[PathBuf]) -> Classified {
    classify(&sessions(), &event(kind, paths))
}

fn create() -> EventKind {
    EventKind::Create(CreateKind::File)
}

#[test]
fn a_created_session_file_is_kept() {
    let path = in_project("chat.jsonl");
    let out = classify_paths(create(), std::slice::from_ref(&path));
    assert_eq!(
        out,
        Classified {
            paths: vec![path],
            rescan: false,
        }
    );
}

#[test]
fn the_two_events_of_one_created_file_both_ask_for_that_path() {
    // m15 measured create then modify for one created file.
    let path = in_project("chat.jsonl");
    for kind in [
        create(),
        EventKind::Modify(ModifyKind::Data(DataChange::Any)),
    ] {
        assert_eq!(
            classify_paths(kind, std::slice::from_ref(&path)).paths,
            vec![path.clone()]
        );
    }
}

#[test]
fn any_other_mutating_kind_on_a_session_file_is_kept() {
    let path = in_project("chat.jsonl");
    let kinds = [
        EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any)),
        EventKind::Modify(ModifyKind::Name(RenameMode::Any)),
        EventKind::Remove(RemoveKind::File),
        EventKind::Any,
    ];
    for kind in kinds {
        assert_eq!(
            classify_paths(kind, std::slice::from_ref(&path)),
            Classified {
                paths: vec![path.clone()],
                rescan: false,
            },
            "{kind:?}"
        );
    }
}

#[test]
fn a_rescan_flag_is_a_full_pass_whatever_the_kind() {
    for kind in [
        EventKind::Any,
        create(),
        EventKind::Access(AccessKind::Read),
        EventKind::Other,
    ] {
        let out = classify_paths(kind, &[in_project("chat.jsonl")]);
        assert!(!out.rescan, "{kind:?} only means a rescan when flagged");
        let flagged = event(kind, &[in_project("chat.jsonl")]).set_flag(Flag::Rescan);
        let out = classify(&sessions(), &flagged);
        assert!(out.rescan, "{kind:?} with a rescan flag must rescan");
    }
}

#[test]
fn a_flagged_event_without_paths_is_still_a_full_pass() {
    let out = classify(
        &sessions(),
        &Event::new(EventKind::Other).set_flag(Flag::Rescan),
    );
    assert_eq!(
        out,
        Classified {
            paths: Vec::new(),
            rescan: true,
        }
    );
}

#[test]
fn a_removed_or_renamed_project_directory_is_a_full_pass() {
    let project = sessions().join("--home-me--");
    let kinds = [
        EventKind::Remove(RemoveKind::Folder),
        EventKind::Remove(RemoveKind::Any),
        EventKind::Modify(ModifyKind::Name(RenameMode::From)),
        EventKind::Modify(ModifyKind::Name(RenameMode::To)),
        EventKind::Modify(ModifyKind::Name(RenameMode::Both)),
    ];
    for kind in kinds {
        let out = classify_paths(kind, std::slice::from_ref(&project));
        assert_eq!(
            out,
            Classified {
                paths: Vec::new(),
                rescan: true,
            },
            "{kind:?}"
        );
    }
}

#[test]
fn a_removed_directory_at_the_sessions_root_is_a_full_pass() {
    // `<sessions>/x.jsonl` (one component) is a directory that looks like a
    // session name only to a rule that forgets the depth.
    let out = classify_paths(
        EventKind::Remove(RemoveKind::Folder),
        &[sessions().join("x.jsonl")],
    );
    assert_eq!(
        out,
        Classified {
            paths: Vec::new(),
            rescan: true,
        },
        "a removed directory at the root is a full pass, not a path"
    );
}

#[test]
fn duplicates_are_collapsed_and_the_order_is_deterministic() {
    let (first, second) = (in_project("b.jsonl"), in_project("a.jsonl"));
    let out = classify_paths(create(), &[first.clone(), second.clone(), first.clone()]);
    assert_eq!(out.paths, vec![second, first]);
}

#[test]
fn one_event_can_both_list_a_file_and_ask_for_a_pass() {
    let mut event = event(create(), &[in_project("chat.jsonl")]).set_flag(Flag::Rescan);
    event = event.add_path(sessions().join("--home-me--"));
    let out = classify(&sessions(), &event);
    assert_eq!(out.paths, vec![in_project("chat.jsonl")]);
    assert!(out.rescan);
}

/// Watchers on Windows report verbatim (`\\?\`) paths, another case, or `/`.
#[cfg(windows)]
#[test]
fn windows_spellings_of_the_session_directory_still_identify_the_file() {
    use std::path::Path;

    let sessions = Path::new(r"C:\Users\me\.pi\sessions");
    let reported = PathBuf::from(r"\\?\C:\Users\me\.pi\sessions\--home-me--\chat.jsonl");
    assert_eq!(
        classify(sessions, &event(create(), std::slice::from_ref(&reported))),
        Classified {
            paths: vec![reported],
            rescan: false,
        }
    );
    let upper = PathBuf::from(r"c:/users/me/.pi/SESSIONS/--home-me--/chat.jsonl");
    assert_eq!(
        classify(sessions, &event(create(), std::slice::from_ref(&upper))).paths,
        vec![upper],
        "the reported spelling is kept, only the comparison is folded"
    );
}

mod ignored;
