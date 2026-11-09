//! Opening a chat in a folder: the working folder its child starts in, and what
//! a folder that cannot host a child leaves behind (nothing at all).

use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use gentle_core::rpc::Decoded;
use serde_json::Value;

use super::real::{host_for, WAIT};
use super::*;
use fake::FakeSpawner;

/// A child that says where it is running, then stays up like a real one.
const REPORTS_CWD: &str = r#"
console.log(JSON.stringify({ type: 'cwd', cwd: process.cwd() }));
setInterval(() => {}, 1000);
"#;

/// One path in a comparable form: canonical, slash-separated, without the
/// Windows verbatim prefix, and case-folded where the file system ignores case.
fn comparable(path: &Path) -> String {
    let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let text = canonical.to_string_lossy().replace('\\', "/");
    let text = text.strip_prefix("//?/").unwrap_or(text.as_str());
    if cfg!(windows) {
        text.to_lowercase()
    } else {
        text.to_owned()
    }
}

/// Opens `key` and reads the working folder the child reports about itself.
fn reported_cwd(host: &ChatHost, key: ChatKey, folder: Option<&Path>) -> String {
    let opened = host
        .open_in(key.clone(), folder.map(Path::to_path_buf))
        .expect("the chat opens");
    assert_eq!(opened, Opened::Started);
    loop {
        let tagged = host.recv_timeout(WAIT).expect("a tagged event");
        assert_eq!(tagged.key, key);
        let ProcessEvent::Output(Decoded::Unknown(line)) = tagged.event else {
            continue;
        };
        let value: Value = serde_json::from_str(&line).expect("a JSON line");
        if value["type"] == "cwd" {
            return value["cwd"].as_str().expect("a folder").to_owned();
        }
    }
}

#[test]
fn the_child_of_a_chat_opened_in_a_folder_starts_there() {
    let home = tempfile::tempdir().unwrap();
    let folder = tempfile::tempdir().unwrap();
    let host = host_for(REPORTS_CWD, home.path());

    let cwd = reported_cwd(&host, ChatKey::New(1), Some(folder.path()));

    assert_eq!(comparable(Path::new(&cwd)), comparable(folder.path()));
    host.shutdown();
}

#[test]
fn a_chat_opened_without_a_folder_keeps_the_process_folder() {
    let dir = tempfile::tempdir().unwrap();
    let host = host_for(REPORTS_CWD, dir.path());
    let process = std::env::current_dir().unwrap();

    let cwd = reported_cwd(&host, ChatKey::New(1), None);

    assert_eq!(comparable(Path::new(&cwd)), comparable(&process));
    host.shutdown();
}

#[test]
fn a_folder_that_is_not_there_fails_the_open_and_leaves_the_host_as_it_was() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("no-such-folder");
    let spawner = Arc::new(FakeSpawner::new());
    let host = plain(&spawner);
    host.open(saved("a")).unwrap();

    let refused = host
        .open_in(saved("b"), Some(missing.clone()))
        .expect_err("a folder that is not there fails the open");

    assert!(
        refused.to_string().contains(&missing.display().to_string()),
        "the failure names the folder: {refused}"
    );
    assert!(!missing.exists(), "the folder is never created");
    assert_eq!(spawner.spawn_count(&saved("b")), 0, "nothing was started");
    assert_eq!(
        host.live_keys(),
        vec![saved("a")],
        "no slot was left behind"
    );
    assert_eq!(
        host.foreground(),
        Some(saved("a")),
        "still on the other chat"
    );
    assert!(!host.is_alive(&saved("b")));
    assert_eq!(
        host.recv_timeout(Duration::from_millis(200)),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout),
        "no event was published"
    );
}

#[test]
fn a_file_is_not_a_folder_to_open_a_chat_in() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("notes.txt");
    fs::write(&file, "not a folder").unwrap();
    let spawner = Arc::new(FakeSpawner::new());
    let host = plain(&spawner);

    let refused = host
        .open_in(saved("a"), Some(file.clone()))
        .expect_err("a file is not a folder");

    assert!(
        refused.to_string().contains(&file.display().to_string()),
        "the failure names the file: {refused}"
    );
    assert_eq!(spawner.spawn_count(&saved("a")), 0);
    assert_eq!(host.live_keys(), Vec::new());
    assert_eq!(host.foreground(), None);
}

#[test]
fn a_live_chat_ignores_a_folder_it_is_opened_in_again() {
    let dir = tempfile::tempdir().unwrap();
    let first = dir.path().join("first");
    fs::create_dir(&first).unwrap();
    let spawner = Arc::new(FakeSpawner::new());
    let host = plain(&spawner);
    host.open_in(saved("a"), Some(first.clone())).unwrap();

    // Not even looked at: the child is up and keeps the folder it started in,
    // so a folder that is not there cannot fail this open.
    let opened = host
        .open_in(saved("a"), Some(dir.path().join("no-such-folder")))
        .expect("a chat that is already up opens again");

    assert_eq!(opened, Opened::Existing);
    assert_eq!(spawner.spawn_count(&saved("a")), 1, "nothing was respawned");
    assert_eq!(spawner.folders_of(&saved("a")), vec![Some(first)]);
}

#[test]
fn the_spawner_records_the_folder_of_every_child() {
    let dir = tempfile::tempdir().unwrap();
    let (one, two) = (dir.path().join("one"), dir.path().join("two"));
    fs::create_dir(&one).unwrap();
    fs::create_dir(&two).unwrap();
    let spawner = Arc::new(FakeSpawner::new());
    let host = plain(&spawner);

    host.open_in(saved("a"), Some(one.clone())).unwrap();
    host.close(&saved("a"));
    host.open_in(saved("a"), Some(two.clone())).unwrap();
    host.open(saved("b")).unwrap();

    assert_eq!(spawner.folders_of(&saved("a")), vec![Some(one), Some(two)]);
    assert_eq!(spawner.spawn_count(&saved("b")), 1);
    assert_eq!(spawner.folders_of(&saved("b")), vec![None]);
}
