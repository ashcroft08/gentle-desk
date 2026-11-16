//! Removing one chat's pi session file (T3, D2(b), S3).
//!
//! No real pi home: a temp directory holds the session files of a test, and the
//! one fact that matters is asserted on the disk itself - the named file is
//! gone and every other file of that folder is byte for byte what it was. A fake
//! answers the seam as well, so the test can say that the delete asks for
//! exactly ONE path and never walks a folder.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use gentle_backend::session_delete::{
    self, DeleteError, DiskRemovals, OsTrash, SessionRemovals, TrashMoves, MISSING_SESSION,
};

/// One session file and one of its neighbours, both under the same folder.
const SESSION: &str = "2026-01-01T00-00-00_s1.jsonl";
const SIBLING: &str = "2026-01-02T00-00-00_s2.jsonl";
const OTHER: &str = "notes.txt";

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

#[test]
fn a_missing_file_is_refused_with_one_clear_line_and_nothing_is_created() {
    let dir = tempfile::tempdir().expect("a temp directory");
    let sessions = dir.path().join("--dev-p--");
    fs::create_dir(&sessions).expect("the project folder");
    let path = sessions.join("gone.jsonl");

    let error = session_delete::delete_session(&DiskRemovals, &path)
        .expect_err("a file that is not there cannot be deleted");

    assert_eq!(error, DeleteError::Missing);
    assert_eq!(
        error.to_string(),
        MISSING_SESSION,
        "the one line a user reads"
    );
    assert!(
        fs::read_dir(&sessions)
            .expect("the folder")
            .next()
            .is_none(),
        "a refused delete creates nothing"
    );
}

#[test]
fn the_named_file_is_removed_and_its_siblings_are_left_alone() {
    let dir = tempfile::tempdir().expect("a temp directory");
    let sessions = dir.path().join("--dev-p--");
    let nested = sessions.join("another");
    fs::create_dir_all(&nested).expect("the project folder");
    let path = sessions.join(SESSION);
    fs::write(&path, "{}\n").expect("the session file");
    fs::write(sessions.join(SIBLING), "{\"type\":\"session\"}\n").expect("a sibling session");
    fs::write(sessions.join(OTHER), "kept by hand\n").expect("a note beside it");

    session_delete::delete_session(&DiskRemovals, &path).expect("the file is removed");

    assert!(!path.exists(), "the named session file is gone");
    assert_eq!(
        fs::read_to_string(sessions.join(SIBLING)).expect("the sibling session"),
        "{\"type\":\"session\"}\n",
        "a sibling session of the same folder is byte for byte"
    );
    assert_eq!(
        fs::read_to_string(sessions.join(OTHER)).expect("the note"),
        "kept by hand\n",
        "a file of another kind beside it is untouched"
    );
    assert!(nested.is_dir(), "no folder is ever removed");
}

#[test]
fn the_seam_is_asked_for_exactly_one_path() {
    let fake = FakeRemovals::holding(&[r"C:\pi\sessions\--dev-p--\s1.jsonl"]);
    let path = Path::new(r"C:\pi\sessions\--dev-p--\s1.jsonl");

    session_delete::delete_session(&fake, path).expect("the file is removed");

    assert_eq!(
        fake.calls(),
        vec![path.display().to_string()],
        "one path, and nothing that walks a folder"
    );
    assert!(!fake.holds(path), "the fake really removed it");
}

#[test]
fn an_empty_path_is_refused_before_the_disk_is_asked() {
    let fake = FakeRemovals::default();

    let error = session_delete::delete_session(&fake, Path::new(""))
        .expect_err("an empty path names no file");

    assert_eq!(error, DeleteError::Missing);
    assert_eq!(error.to_string(), MISSING_SESSION);
    assert!(fake.calls().is_empty(), "the seam is never asked");
}

#[test]
fn a_file_the_system_will_not_remove_says_so_instead_of_claiming_a_delete() {
    let fake = FakeRemovals::refusing(io::Error::new(
        io::ErrorKind::PermissionDenied,
        "access is denied",
    ));
    let path = Path::new(r"C:\pi\sessions\--dev-p--\locked.jsonl");

    let error = session_delete::delete_session(&fake, path)
        .expect_err("a file that cannot be removed is reported");

    assert!(
        matches!(error, DeleteError::Disk(_)),
        "the reason is the system's, got {error:?}"
    );
    let line = error.to_string();
    assert!(
        line.contains("could not be removed") && line.contains("access is denied"),
        "got {line:?}"
    );
}

// --- The trash first (S4) ---------------------------------------------------

#[test]
fn a_deleted_session_file_leaves_its_folder() {
    let dir = tempfile::tempdir().expect("a temp directory");
    let path = dir.path().join(SESSION);
    fs::write(&path, "{\"type\":\"session\"}\n").expect("the session file");

    session_delete::delete_session(&DiskRemovals, &path).expect("the file is removed");

    assert!(
        !path.exists(),
        "the chat is gone from the folder it lived in, whichever way the system took it"
    );
}

#[test]
fn the_trash_is_asked_for_the_named_path_before_anything_is_unlinked() {
    let dir = tempfile::tempdir().expect("a temp directory");
    let path = dir.path().join(SESSION);
    fs::write(&path, "{}\n").expect("the session file");
    let trash = FakeTrash::accepting();

    session_delete::remove_file_trash_first(&trash, &path).expect("the trash took it");

    assert_eq!(
        trash.calls(),
        vec![path.display().to_string()],
        "exactly the named file was offered to the trash"
    );
    assert!(
        path.exists(),
        "a trash move that worked is the whole removal: the file is never unlinked"
    );
}

#[test]
fn a_trash_move_the_system_refuses_falls_back_to_unlinking_the_file() {
    let dir = tempfile::tempdir().expect("a temp directory");
    let path = dir.path().join(SESSION);
    fs::write(&path, "{}\n").expect("the session file");
    let trash = FakeTrash::refusing();

    session_delete::remove_file_trash_first(&trash, &path).expect("the file is removed anyway");

    assert_eq!(
        trash.calls(),
        vec![path.display().to_string()],
        "the trash was tried first, and refused"
    );
    assert!(!path.exists(), "the fallback removed the file");
}

#[test]
fn a_trash_that_refuses_a_file_that_is_not_there_is_still_a_missing_session() {
    let dir = tempfile::tempdir().expect("a temp directory");
    let path = dir.path().join("gone.jsonl");

    let error = session_delete::delete_session(&RefusingTrashRemovals, &path)
        .expect_err("a file that is not there cannot be deleted");

    assert_eq!(error, DeleteError::Missing);
    assert_eq!(error.to_string(), MISSING_SESSION);
}

#[test]
fn this_systems_own_trash_takes_a_session_file() {
    let dir = tempfile::tempdir().expect("a temp directory");
    let path = dir.path().join(SESSION);
    fs::write(&path, "{\"type\":\"session\"}\n").expect("the session file");

    OsTrash
        .move_to_trash(&path)
        .expect("the system's own trash takes a file; the unlink is only the fallback");

    assert!(!path.exists(), "a moved file leaves the folder it lived in");
}

/// The real folder, with a trash that always refuses: what a system whose
/// recycle bin is unavailable looks like to the delete.
struct RefusingTrashRemovals;

impl SessionRemovals for RefusingTrashRemovals {
    fn remove(&self, path: &Path) -> io::Result<()> {
        session_delete::remove_file_trash_first(&FakeTrash::refusing(), path)
    }
}

/// A trash that records every path it was offered and either takes it (moving
/// nothing, so a test can see that no unlink followed) or refuses it.
struct FakeTrash {
    calls: Mutex<Vec<String>>,
    refusal: Option<String>,
}

impl FakeTrash {
    fn accepting() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            refusal: None,
        }
    }

    fn refusing() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            refusal: Some("the recycle bin is not available".to_owned()),
        }
    }

    fn calls(&self) -> Vec<String> {
        lock(&self.calls).clone()
    }
}

impl session_delete::TrashMoves for FakeTrash {
    fn move_to_trash(&self, path: &Path) -> io::Result<()> {
        lock(&self.calls).push(path.display().to_string());
        match &self.refusal {
            Some(reason) => Err(io::Error::other(reason.clone())),
            None => Ok(()),
        }
    }
}

/// The session files a test holds, in memory, and every path the seam was asked
/// to remove.
#[derive(Default)]
struct FakeRemovals {
    files: Mutex<Vec<PathBuf>>,
    calls: Mutex<Vec<String>>,
    refusal: Mutex<Option<(io::ErrorKind, String)>>,
}

impl FakeRemovals {
    fn holding(paths: &[&str]) -> Self {
        let fake = FakeRemovals::default();
        *lock(&fake.files) = paths.iter().map(PathBuf::from).collect();
        fake
    }

    fn refusing(error: io::Error) -> Self {
        let fake = FakeRemovals::default();
        *lock(&fake.refusal) = Some((error.kind(), error.to_string()));
        fake
    }

    fn calls(&self) -> Vec<String> {
        lock(&self.calls).clone()
    }

    fn holds(&self, path: &Path) -> bool {
        lock(&self.files).iter().any(|held| held.as_path() == path)
    }
}

impl SessionRemovals for FakeRemovals {
    fn remove(&self, path: &Path) -> io::Result<()> {
        lock(&self.calls).push(path.display().to_string());
        if let Some((kind, message)) = lock(&self.refusal).as_ref() {
            return Err(io::Error::new(*kind, message.clone()));
        }
        let mut files = lock(&self.files);
        match files.iter().position(|held| held.as_path() == path) {
            Some(at) => {
                files.remove(at);
                Ok(())
            }
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "no such session file",
            )),
        }
    }
}
