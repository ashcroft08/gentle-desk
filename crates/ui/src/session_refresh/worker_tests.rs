use super::*;
use iced::futures::executor::block_on;
use std::fs;
use std::path::Path;

const HEADER: &str = r#"{"type":"session","version":3,"id":"s1","timestamp":"2026-01-01T00:00:00.000Z","cwd":"C:\\dev\\p"}"#;

fn message(text: &str) -> String {
    format!(
        r#"{{"type":"message","id":"m","timestamp":"2026-01-01T00:01:00.000Z","message":{{"role":"user","content":"{text}"}}}}"#
    )
}

/// A pi home with a `sessions/` directory and one file per session.
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

    fn project(&self, name: &str) -> PathBuf {
        let dir = self.sessions().join(name);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A session file with a header and one user message.
    fn session(&self, project: &str, name: &str, first: &str) -> PathBuf {
        let path = self.project(project).join(name);
        let body = format!("{HEADER}\n{}\n", message(first));
        fs::write(&path, body).unwrap();
        path
    }

    /// Appends `line` and returns the bytes written, as a pass should read them.
    fn append(&self, path: &Path, line: &str) -> u64 {
        let text = format!("{line}\n");
        let mut body = fs::read_to_string(path).unwrap();
        body.push_str(&text);
        fs::write(path, body).unwrap();
        text.len() as u64
    }

    fn index(&self) -> SessionIndex {
        SessionIndex::new(self.sessions())
    }

    fn cache(&self) -> PathBuf {
        self.dir.path().join("data").join("session-index-v1.json")
    }
}

fn index_in(sessions: PathBuf) -> SharedIndex {
    Arc::new(Mutex::new(SessionIndex::new(sessions)))
}

fn item<'a>(refreshed: &'a Refreshed, path: &Path) -> Option<&'a SessionItem> {
    let wanted = path.to_string_lossy();
    refreshed
        .sessions
        .items()
        .iter()
        .find(|item| item.path == wanted)
}

fn titles(refreshed: &Refreshed) -> Vec<String> {
    refreshed
        .sessions
        .items()
        .iter()
        .map(|item| item.title.clone())
        .collect()
}

#[test]
fn a_full_pass_is_full_and_a_paths_pass_is_not() {
    let home = Home::new();
    let a = home.session("--C--p--", "a.jsonl", "uno");
    let shared = Shared::new(index_in(home.sessions()), None);

    let full = block_on(run(Job::Full, shared.clone())).unwrap();
    assert!(full.full);
    assert_eq!(titles(&full), ["uno"]);

    home.append(&a, &message("dos"));
    let paths = block_on(run(Job::Paths(vec![a.clone()]), shared)).unwrap();
    assert!(!paths.full);
    assert_eq!(item(&paths, &a).unwrap().message_count, 2);
}

/// The point of a targeted pass: it looks at the reported file and nothing else.
#[test]
fn a_paths_pass_updates_only_the_reported_path_and_a_full_pass_finds_the_rest() {
    let home = Home::new();
    let a = home.session("--C--p--", "a.jsonl", "a-uno");
    let b = home.session("--C--p--", "b.jsonl", "b-uno");
    let shared = Shared::new(index_in(home.sessions()), None);
    let first = block_on(run(Job::Full, shared.clone())).unwrap();
    assert_eq!(titles(&first), ["a-uno", "b-uno"]);

    // Both cached files grow, and a brand-new session appears.
    home.append(&a, &message("a-dos"));
    home.append(&b, &message("b-dos"));
    let c = home.session("--C--q--", "c.jsonl", "c-uno");

    let targeted = block_on(run(Job::Paths(vec![a.clone()]), shared.clone())).unwrap();
    assert_eq!(item(&targeted, &a).unwrap().message_count, 2);
    // B was not reported, so it is not re-read, and C is not enumerated at all.
    assert_eq!(item(&targeted, &b).unwrap().message_count, 1);
    assert_eq!(titles(&targeted), ["a-uno", "b-uno"]);
    assert!(item(&targeted, &c).is_none());

    let full = block_on(run(Job::Full, shared)).unwrap();
    assert_eq!(item(&full, &a).unwrap().message_count, 2);
    assert_eq!(item(&full, &b).unwrap().message_count, 2);
    assert_eq!(item(&full, &c).unwrap().message_count, 1);
}

#[test]
fn a_paths_pass_after_a_restored_index_reads_only_the_appended_bytes() {
    let home = Home::new();
    let a = home.session("--C--p--", "a.jsonl", "uno");
    let first = block_on(run(Job::Full, Shared::new(index_in(home.sessions()), None))).unwrap();
    assert!(first.stats.bytes_read > 0);

    // A later run starts warm from the cache instead of reading the file again.
    let mut restored = home.index();
    restored.restore(warm_snapshot(&home));
    let shared = Shared::new(Arc::new(Mutex::new(restored)), None);

    let appended = home.append(&a, &message("dos"));
    let targeted = block_on(run(Job::Paths(vec![a.clone()]), shared)).unwrap();
    assert_eq!(targeted.stats.bytes_read, appended);
    assert_eq!(item(&targeted, &a).unwrap().message_count, 2);
}

/// The snapshot of the home as it is now, read by one full pass.
fn warm_snapshot(home: &Home) -> gentle_backend::session_snapshot::IndexSnapshot {
    let mut index = home.index();
    index.refresh().unwrap();
    index.snapshot()
}

#[test]
fn the_first_change_writes_the_cache_a_second_change_within_the_interval_does_not_and_flush_does() {
    let home = Home::new();
    let a = home.session("--C--p--", "a.jsonl", "uno");
    let cache = home.cache();
    let shared = Shared::new(index_in(home.sessions()), Some(cache.clone()));

    block_on(run(Job::Full, shared.clone())).unwrap();
    let written = fs::read_to_string(&cache).expect("the first change wrote the cache");

    home.append(&a, &message("dos"));
    block_on(run(Job::Paths(vec![a.clone()]), shared.clone())).unwrap();
    assert_eq!(
        fs::read_to_string(&cache).unwrap(),
        written,
        "a change within the interval must not rewrite a 50 000-chat document"
    );

    flush(&shared);
    let flushed = fs::read_to_string(&cache).unwrap();
    assert_ne!(flushed, written, "flush writes what the throttle held back");
}

#[test]
fn a_pass_that_changed_nothing_writes_nothing() {
    let home = Home::new();
    home.session("--C--p--", "a.jsonl", "uno");
    let cache = home.cache();
    let shared = Shared::new(index_in(home.sessions()), Some(cache.clone()));
    block_on(run(Job::Full, shared.clone())).unwrap();

    // Anything the pass wrote would replace this marker.
    fs::write(&cache, "untouched").unwrap();
    let unchanged = block_on(run(Job::Full, shared.clone())).unwrap();
    assert_eq!(unchanged.stats.appended + unchanged.stats.rescanned, 0);
    assert_eq!(fs::read_to_string(&cache).unwrap(), "untouched");

    // Nor does a flush with nothing dirty.
    flush(&shared);
    assert_eq!(fs::read_to_string(&cache).unwrap(), "untouched");
}

#[test]
fn no_cache_path_writes_nothing() {
    let home = Home::new();
    home.session("--C--p--", "a.jsonl", "uno");
    let shared = Shared::new(index_in(home.sessions()), None);
    let refreshed = block_on(run(Job::Full, shared.clone())).unwrap();
    assert_eq!(refreshed.sessions.items().len(), 1);
    flush(&shared);
    assert!(!home.dir.path().join("data").exists());
}

#[test]
fn an_unwritable_cache_location_does_not_fail_the_pass() {
    let home = Home::new();
    home.session("--C--p--", "a.jsonl", "uno");
    // A file where the cache directory should be: the save cannot win.
    let blocked = home.dir.path().join("data");
    fs::write(&blocked, "not a directory").unwrap();
    let shared = Shared::new(
        index_in(home.sessions()),
        Some(blocked.join("session-index-v1.json")),
    );
    let refreshed = block_on(run(Job::Full, shared)).unwrap();
    assert_eq!(refreshed.sessions.items().len(), 1);
}
