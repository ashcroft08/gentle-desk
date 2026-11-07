//! Depth and spelling rules shared by the classifier.

use std::path::{Path, PathBuf};

use super::*;

/// An absolute directory that exists on every platform, spelled portably.
fn root() -> PathBuf {
    std::env::temp_dir().join("gentle-watch-paths")
}

fn parts_of(sessions: &Path, path: &Path) -> Option<Vec<String>> {
    below(sessions, path)
}

#[test]
fn a_directory_in_the_sessions_root_is_one_component() {
    let parts = parts_of(&root(), &root().join("proj")).unwrap();
    assert_eq!(parts, ["proj"]);
}

#[test]
fn a_session_file_is_two_components() {
    let parts = parts_of(&root(), &root().join("proj").join("x.jsonl")).unwrap();
    assert_eq!(parts, ["proj", "x.jsonl"]);
}

#[test]
fn anything_not_below_the_sessions_directory_is_none() {
    let sessions = root();
    // The directory itself.
    assert!(parts_of(&sessions, &sessions).is_none());
    // A sibling with a shared name prefix: a component, not a string prefix.
    assert!(parts_of(
        &sessions,
        &std::env::temp_dir().join("gentle-watch-paths-x")
    )
    .is_none());
    // A relative path.
    assert!(parts_of(&sessions, Path::new("sessions/proj/x.jsonl")).is_none());
    // A path that climbs back out.
    assert!(parts_of(&sessions, &sessions.join("..").join("x").join("p.jsonl")).is_none());
}

#[cfg(windows)]
#[test]
fn a_windows_path_on_another_drive_is_not_below_the_sessions_directory() {
    assert!(parts_of(&root(), Path::new(r"D:\elsewhere\proj\x.jsonl")).is_none());
}

#[cfg(unix)]
#[test]
fn a_unix_path_in_another_directory_is_not_below_the_sessions_directory() {
    assert!(parts_of(&root(), Path::new("/elsewhere/proj/x.jsonl")).is_none());
}

#[test]
fn a_trailing_separator_on_the_sessions_directory_is_ignored() {
    let mut spelled = root().into_os_string();
    spelled.push(std::path::MAIN_SEPARATOR.to_string());
    let parts = parts_of(
        &PathBuf::from(spelled),
        &root().join("proj").join("x.jsonl"),
    )
    .unwrap();
    assert_eq!(parts, ["proj", "x.jsonl"]);
}

/// A watcher on Windows reports `\\?\` verbatim paths, `/` separators and
/// another case; all of them name the same file. Only Windows folds case, so
/// that is the platform these spellings are checked on.
#[cfg(windows)]
#[test]
fn windows_spellings_of_one_path_compare_equal() {
    let sessions = Path::new(r"C:\Users\me\.pi\sessions");
    for path in [
        r"\\?\C:\Users\me\.pi\sessions\proj\x.jsonl",
        r"\\?\c:\users\ME\.pi\sessions\proj\x.jsonl",
        "C:/Users/me/.pi/sessions/proj/x.jsonl",
    ] {
        assert_eq!(
            below(sessions, Path::new(path)).unwrap(),
            ["proj", "x.jsonl"],
            "{path}"
        );
    }
    // The verbatim prefix and the case folding apply to the sessions side too.
    let sessions = Path::new(r"\\?\C:\Users\me\.pi\sessions");
    assert_eq!(
        below(
            sessions,
            Path::new(r"c:\users\me\.pi\SESSIONS\proj\x.jsonl")
        )
        .unwrap(),
        ["proj", "x.jsonl"]
    );
}

#[cfg(windows)]
#[test]
fn a_windows_path_outside_the_sessions_directory_is_not_below_it() {
    let sessions = Path::new(r"C:\Users\me\.pi\sessions");
    assert!(below(sessions, Path::new(r"\\?\C:\Users\me\.pi\sessions")).is_none());
    assert!(below(
        sessions,
        Path::new(r"\\?\C:\Users\me\.pi\sessions-backup\proj\x.jsonl")
    )
    .is_none());
    assert!(below(
        sessions,
        Path::new(r"D:\Users\me\.pi\sessions\proj\x.jsonl")
    )
    .is_none());
}
