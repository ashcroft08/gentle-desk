//! The other half of the classifier: what it refuses to act on.

use notify::event::{AccessKind, AccessMode, CreateKind, MetadataKind};
use notify::event::{ModifyKind, RemoveKind, RenameMode};

use super::*;

#[test]
fn access_events_are_ignored() {
    // Opening or closing a session changes nothing in the list.
    let path = in_project("chat.jsonl");
    for kind in [
        AccessKind::Read,
        AccessKind::Open(AccessMode::Write),
        AccessKind::Close(AccessMode::Write),
    ] {
        let out = classify_paths(EventKind::Access(kind), std::slice::from_ref(&path));
        assert!(out.is_empty(), "{kind:?} asked for {out:?}");
    }
}

#[test]
fn other_events_are_ignored() {
    let out = classify_paths(EventKind::Other, &[in_project("chat.jsonl")]);
    assert!(out.is_empty());
}

#[test]
fn a_created_or_modified_project_directory_is_not_a_full_pass() {
    // Its files arrive as their own events; a directory event alone says nothing
    // the index can act on.
    let project = sessions().join("--home-me--");
    for kind in [
        create(),
        EventKind::Create(CreateKind::Folder),
        EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any)),
        EventKind::Any,
    ] {
        let out = classify_paths(kind, std::slice::from_ref(&project));
        assert!(out.is_empty(), "{kind:?} asked for {out:?}");
    }
}

#[test]
fn a_deeper_path_that_is_not_a_session_file_is_ignored() {
    let deeper = in_project("notes.txt");
    let kinds = [
        create(),
        EventKind::Modify(ModifyKind::Name(RenameMode::Both)),
        EventKind::Remove(RemoveKind::File),
        EventKind::Any,
    ];
    for kind in kinds {
        assert!(
            classify_paths(kind, std::slice::from_ref(&deeper)).is_empty(),
            "{kind:?} asked for something on {deeper:?}"
        );
    }
}

#[test]
fn paths_outside_a_project_directory_are_ignored() {
    let paths = [
        // At the sessions root: a file, not a session.
        sessions().join("x.jsonl"),
        // Too deep.
        sessions().join("a").join("b").join("x.jsonl"),
        // Another directory.
        std::env::temp_dir().join("elsewhere").join("x.jsonl"),
        // A relative path.
        PathBuf::from("--home-me--/x.jsonl"),
    ];
    for path in paths {
        let out = classify_paths(create(), std::slice::from_ref(&path));
        assert!(out.is_empty(), "{path:?} was accepted");
    }
}
