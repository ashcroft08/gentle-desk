use super::*;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::Duration;

fn header(id: &str, timestamp: &str) -> String {
    format!(
        r#"{{"type":"session","version":3,"id":"{id}","timestamp":"{timestamp}","cwd":"C:\\dev\\{id}"}}"#
    )
}

fn user(text: &str, timestamp: &str) -> String {
    format!(
        r#"{{"type":"message","id":"m","timestamp":"{timestamp}","message":{{"role":"user","content":"{text}"}}}}"#
    )
}

struct Home {
    dir: tempfile::TempDir,
}

impl Home {
    fn new() -> Self {
        Self {
            dir: tempfile::tempdir().unwrap(),
        }
    }

    fn sessions(&self) -> PathBuf {
        self.dir.path().join("sessions")
    }

    /// Writes `lines` as the whole file `<project>/<name>.jsonl`.
    fn write(&self, project: &str, name: &str, lines: &[String]) -> PathBuf {
        let dir = self.sessions().join(project);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("{name}.jsonl"));
        fs::write(
            &path,
            lines.iter().map(|l| format!("{l}\n")).collect::<String>(),
        )
        .unwrap();
        path
    }

    fn index(&self) -> SessionIndex {
        SessionIndex::new(self.sessions())
    }
}

fn append(path: &Path, text: &str) {
    let mut file = OpenOptions::new().append(true).open(path).unwrap();
    file.write_all(text.as_bytes()).unwrap();
}

/// mtime granularity can hide a rewrite; force a distinct one.
fn bump_mtime(path: &Path, seconds: u64) {
    let file = File::options().write(true).open(path).unwrap();
    let mtime = fs::metadata(path).unwrap().modified().unwrap() + Duration::from_secs(seconds);
    file.set_modified(mtime).unwrap();
}

#[test]
fn a_missing_sessions_directory_is_an_empty_list() {
    let home = Home::new();
    let mut index = home.index();
    assert_eq!(index.refresh().unwrap(), RefreshStats::default());
    assert!(index.entries().is_empty());
}

#[test]
fn sessions_are_found_across_project_directories_and_sorted_by_activity() {
    let home = Home::new();
    home.write(
        "--C--a--",
        "old",
        &[
            header("old", "2026-01-01T00:00:00.000Z"),
            user("viejo", "2026-01-01T00:01:00.000Z"),
        ],
    );
    home.write(
        "--C--b--",
        "new",
        &[
            header("new", "2026-02-01T00:00:00.000Z"),
            user("nuevo", "2026-02-01T00:01:00.000Z"),
        ],
    );
    let mut index = home.index();
    let stats = index.refresh().unwrap();
    assert_eq!((stats.files, stats.rescanned, stats.appended), (2, 2, 0));
    let ids: Vec<_> = index
        .entries()
        .iter()
        .map(|e| e.summary.id.clone())
        .collect();
    assert_eq!(ids, ["new", "old"]);
    assert_eq!(
        index.entries()[0].summary.first_message.as_deref(),
        Some("nuevo")
    );
}

#[test]
fn non_session_files_and_stray_entries_are_ignored() {
    let home = Home::new();
    home.write(
        "--C--a--",
        "notes",
        &[user("sin cabecera", "2026-01-01T00:00:00.000Z")],
    );
    fs::write(home.sessions().join("--C--a--").join("readme.txt"), "x").unwrap();
    fs::write(
        home.sessions().join("loose.jsonl"),
        header("loose", "2026-01-01T00:00:00.000Z"),
    )
    .unwrap();
    let mut index = home.index();
    assert_eq!(index.refresh().unwrap().files, 1); // only the project-level .jsonl
    assert!(index.entries().is_empty());
}

#[test]
fn an_unchanged_file_is_not_read_again() {
    let home = Home::new();
    home.write(
        "--C--a--",
        "s",
        &[
            header("s", "2026-01-01T00:00:00.000Z"),
            user("hola", "2026-01-01T00:01:00.000Z"),
        ],
    );
    let mut index = home.index();
    assert!(index.refresh().unwrap().bytes_read > 0);
    let again = index.refresh().unwrap();
    assert_eq!(
        again,
        RefreshStats {
            files: 1,
            ..RefreshStats::default()
        }
    );
}

#[test]
fn a_grown_file_only_has_its_appended_bytes_read() {
    let home = Home::new();
    let first = [
        header("s", "2026-01-01T00:00:00.000Z"),
        user("hola", "2026-01-01T00:01:00.000Z"),
    ];
    let path = home.write("--C--a--", "s", &first);
    let mut index = home.index();
    index.refresh().unwrap();

    let extra = format!("{}\n", user("segundo", "2026-01-01T00:02:00.000Z"));
    append(&path, &extra);
    let stats = index.refresh().unwrap();
    assert_eq!((stats.rescanned, stats.appended), (0, 1));
    assert_eq!(stats.bytes_read, extra.len() as u64);
    let entry = &index.entries()[0];
    assert_eq!(entry.summary.message_count, 2);
    assert_eq!(entry.summary.first_message.as_deref(), Some("hola"));
}

#[test]
fn a_half_written_line_waits_until_it_is_complete() {
    let home = Home::new();
    let path = home.write("--C--a--", "s", &[header("s", "2026-01-01T00:00:00.000Z")]);
    let mut index = home.index();
    index.refresh().unwrap();

    let line = user("completo", "2026-01-01T00:01:00.000Z");
    let (head, tail) = line.split_at(20);
    append(&path, head);
    index.refresh().unwrap();
    assert_eq!(index.entries()[0].summary.message_count, 0);

    append(&path, &format!("{tail}\n"));
    index.refresh().unwrap();
    assert_eq!(index.entries()[0].summary.message_count, 1);
    assert_eq!(
        index.entries()[0].summary.first_message.as_deref(),
        Some("completo")
    );
}

#[test]
fn a_rewritten_smaller_file_is_rescanned_from_the_start() {
    let home = Home::new();
    let long = [
        header("s", "2026-01-01T00:00:00.000Z"),
        user("uno", "2026-01-01T00:01:00.000Z"),
        user("dos", "2026-01-01T00:02:00.000Z"),
    ];
    let path = home.write("--C--a--", "s", &long);
    let mut index = home.index();
    index.refresh().unwrap();

    home.write("--C--a--", "s", &[header("s", "2026-01-01T00:00:00.000Z")]);
    let stats = index.refresh().unwrap();
    assert_eq!((stats.rescanned, stats.appended), (1, 0));
    assert_eq!(index.entries()[0].summary.message_count, 0);
    assert!(path.exists());
}

#[test]
fn a_same_size_rewrite_with_a_new_mtime_is_rescanned() {
    let home = Home::new();
    let path = home.write(
        "--C--a--",
        "s",
        &[
            header("s", "2026-01-01T00:00:00.000Z"),
            user("aaaa", "2026-01-01T00:01:00.000Z"),
        ],
    );
    let mut index = home.index();
    index.refresh().unwrap();
    home.write(
        "--C--a--",
        "s",
        &[
            header("s", "2026-01-01T00:00:00.000Z"),
            user("bbbb", "2026-01-01T00:01:00.000Z"),
        ],
    );
    bump_mtime(&path, 5);
    index.refresh().unwrap();
    assert_eq!(
        index.entries()[0].summary.first_message.as_deref(),
        Some("bbbb")
    );
}

/// The reference app loaded the chat list once; every refresh must see changes.
#[test]
fn new_and_deleted_files_show_up_on_the_next_refresh() {
    let home = Home::new();
    let first = home.write(
        "--C--a--",
        "one",
        &[header("one", "2026-01-01T00:00:00.000Z")],
    );
    let mut index = home.index();
    index.refresh().unwrap();
    assert_eq!(index.entries().len(), 1);

    home.write(
        "--C--b--",
        "two",
        &[header("two", "2026-01-02T00:00:00.000Z")],
    );
    let stats = index.refresh().unwrap();
    assert_eq!((stats.rescanned, stats.removed), (1, 0));
    assert_eq!(index.entries().len(), 2);

    fs::remove_file(first).unwrap();
    let stats = index.refresh().unwrap();
    assert_eq!(stats.removed, 1);
    let ids: Vec<_> = index
        .entries()
        .iter()
        .map(|e| e.summary.id.clone())
        .collect();
    assert_eq!(ids, ["two"]);
}

#[test]
fn a_session_without_messages_is_ordered_by_creation_time() {
    let home = Home::new();
    home.write(
        "--C--a--",
        "early",
        &[header("early", "2026-01-01T00:00:00.000Z")],
    );
    home.write(
        "--C--a--",
        "late",
        &[header("late", "2026-03-01T00:00:00.000Z")],
    );
    let mut index = home.index();
    index.refresh().unwrap();
    let ids: Vec<_> = index
        .entries()
        .iter()
        .map(|e| e.summary.id.clone())
        .collect();
    assert_eq!(ids, ["late", "early"]);
}

/// A second index over the same home, warmed from `index`'s snapshot: the
/// state a persisted cache brings back on the next run.
fn restored(index: &SessionIndex) -> SessionIndex {
    let mut fresh = SessionIndex::new(index.sessions_dir.clone());
    fresh.restore(index.snapshot());
    fresh
}

#[test]
fn a_snapshot_round_trip_keeps_every_entry_and_the_order() {
    let home = Home::new();
    home.write(
        "--C--a--",
        "old",
        &[
            header("old", "2026-01-01T00:00:00.000Z"),
            user("viejo", "2026-01-01T00:01:00.000Z"),
        ],
    );
    home.write(
        "--C--b--",
        "new",
        &[header("new", "2026-02-01T00:00:00.000Z")],
    );
    let mut index = home.index();
    index.refresh().unwrap();

    let fresh = restored(&index);
    assert_eq!(fresh.entries(), index.entries());
    let ids: Vec<_> = fresh
        .entries()
        .iter()
        .map(|e| e.summary.id.clone())
        .collect();
    assert_eq!(ids, ["new", "old"]);
}

#[test]
fn a_restored_index_reads_nothing_when_the_home_is_unchanged() {
    let home = Home::new();
    home.write(
        "--C--a--",
        "s",
        &[
            header("s", "2026-01-01T00:00:00.000Z"),
            user("hola", "2026-01-01T00:01:00.000Z"),
        ],
    );
    let mut index = home.index();
    index.refresh().unwrap();

    let mut fresh = restored(&index);
    let stats = fresh.refresh().unwrap();
    assert_eq!(
        stats,
        RefreshStats {
            files: 1,
            ..RefreshStats::default()
        }
    );
    assert_eq!(fresh.entries(), index.entries());
}

#[test]
fn a_restored_index_reads_only_the_appended_bytes() {
    let home = Home::new();
    let path = home.write(
        "--C--a--",
        "s",
        &[
            header("s", "2026-01-01T00:00:00.000Z"),
            user("hola", "2026-01-01T00:01:00.000Z"),
        ],
    );
    let mut index = home.index();
    index.refresh().unwrap();

    let extra = format!("{}\n", user("segundo", "2026-01-01T00:02:00.000Z"));
    append(&path, &extra);
    let mut fresh = restored(&index);
    let stats = fresh.refresh().unwrap();
    assert_eq!((stats.rescanned, stats.appended), (0, 1));
    assert_eq!(stats.bytes_read, extra.len() as u64);
    let entry = &fresh.entries()[0];
    assert_eq!(entry.summary.message_count, 2);
    assert_eq!(entry.summary.first_message.as_deref(), Some("hola"));
}

#[test]
fn a_restored_index_forgets_a_deleted_file() {
    let home = Home::new();
    let path = home.write("--C--a--", "s", &[header("s", "2026-01-01T00:00:00.000Z")]);
    let mut index = home.index();
    index.refresh().unwrap();

    fs::remove_file(path).unwrap();
    let mut fresh = restored(&index);
    let stats = fresh.refresh().unwrap();
    assert_eq!(stats.removed, 1);
    assert!(fresh.entries().is_empty());
}

#[test]
fn a_restored_file_rewritten_with_the_same_size_is_rescanned() {
    let home = Home::new();
    let lines = |text: &str| {
        vec![
            header("s", "2026-01-01T00:00:00.000Z"),
            user(text, "2026-01-01T00:01:00.000Z"),
        ]
    };
    let path = home.write("--C--a--", "s", &lines("aaaa"));
    let mut index = home.index();
    index.refresh().unwrap();

    home.write("--C--a--", "s", &lines("bbbb"));
    bump_mtime(&path, 5);
    let mut fresh = restored(&index);
    let stats = fresh.refresh().unwrap();
    assert_eq!((stats.rescanned, stats.appended), (1, 0));
    assert_eq!(
        fresh.entries()[0].summary.first_message.as_deref(),
        Some("bbbb")
    );
}

#[test]
fn a_snapshot_for_another_sessions_directory_is_ignored() {
    let home = Home::new();
    home.write("--C--a--", "s", &[header("s", "2026-01-01T00:00:00.000Z")]);
    let mut index = home.index();
    index.refresh().unwrap();

    // The same files under a second home: the snapshot names the first one.
    let other = Home::new();
    other.write("--C--a--", "s", &[header("s", "2026-01-01T00:00:00.000Z")]);
    let mut elsewhere = SessionIndex::new(other.sessions());
    elsewhere.restore(index.snapshot());
    let stats = elsewhere.refresh().unwrap();
    // Ignored: the files are read again instead of being taken from the cache.
    assert_eq!((stats.files, stats.rescanned), (1, 1));
    assert_eq!(elsewhere.entries().len(), 1);
}

/// A cache that claims bytes it never parsed would silently drop the rest of a
/// file, so such an entry is dropped and the file is scanned again.
#[test]
fn a_snapshot_entry_past_the_end_of_its_file_is_dropped() {
    let home = Home::new();
    let path = home.write("--C--a--", "s", &[header("s", "2026-01-01T00:00:00.000Z")]);
    let mut index = home.index();
    index.refresh().unwrap();
    let mut snapshot = index.snapshot();
    snapshot.files[0].parsed_len = usize::MAX as u64;

    append(
        &path,
        &format!("{}\n", user("hola", "2026-01-01T00:01:00.000Z")),
    );
    let mut fresh = SessionIndex::new(home.sessions());
    fresh.restore(snapshot);
    let stats = fresh.refresh().unwrap();
    assert_eq!(stats.rescanned, 1);
    assert_eq!(fresh.entries()[0].summary.message_count, 1);
}
