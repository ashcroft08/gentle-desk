//! These tests drive real child processes (`node` running a small script that
//! stands in for `gentle-shell`), so the spawn command line, the tagged event
//! stream and the shutdown of a whole tree are the production ones.

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use gentle_core::rpc::Decoded;
use serde_json::{json, Value};

use super::*;
use crate::chat_host::{ChatHost, Limits, ProcessSpawner, SystemClock};
use crate::launcher::Launcher;
use crate::process::ProcessEvent;

/// How long a test waits for a real child to speak, or for a process to go away.
/// Generous on purpose: the child is a cold `node` start, and on a loaded CI runner
/// (two main-branch runs failed here with a Timeout at 10 s, while the same tests
/// passed on pull-request runs) a start can take longer than it ever does locally.
pub(super) const WAIT: Duration = Duration::from_secs(30);

/// A child that reports its own pid, its grandchild's pid and its arguments,
/// then ignores stdin EOF: only the whole tree being killed stops it.
const STUBBORN_TREE: &str = r#"
import { spawn } from 'node:child_process';
const grandchild = spawn(process.execPath, ['-e', 'setInterval(() => {}, 1000)'], { stdio: 'ignore', detached: true });
console.log(JSON.stringify({ type: 'pids', child: process.pid, grandchild: grandchild.pid }));
console.log(JSON.stringify({ type: 'argv', args: process.argv.slice(2) }));
setInterval(() => {}, 1000);
"#;

fn real_host(dir: &Path) -> ChatHost {
    host_for(STUBBORN_TREE, dir)
}

/// A host whose children are `node` running `script`, written in `dir`.
pub(super) fn host_for(script: &str, dir: &Path) -> ChatHost {
    let file = dir.join("fake.mjs");
    fs::write(&file, script).unwrap();
    let launcher = Launcher {
        program: "node".into(),
        args: vec![file.into()],
    };
    ChatHost::with_parts(
        Limits::default(),
        Arc::new(ProcessSpawner::new(launcher)),
        Arc::new(SystemClock),
    )
    .with_grace(Duration::from_millis(300))
}

/// Opens `key` and reads that child's pids and command line from its own stream.
fn open_and_read(host: &ChatHost, key: ChatKey) -> (u32, u32, Value) {
    assert_eq!(host.open(key.clone()).unwrap(), Opened::Started);
    let (mut pids, mut argv) = (None, None);
    while pids.is_none() || argv.is_none() {
        let tagged = host.recv_timeout(WAIT).expect("a tagged event");
        assert_eq!(tagged.key, key);
        let ProcessEvent::Output(Decoded::Unknown(line)) = tagged.event else {
            continue;
        };
        let value: Value = serde_json::from_str(&line).unwrap();
        match value["type"].as_str() {
            Some("pids") => pids = Some(value),
            Some("argv") => argv = Some(value),
            _ => {}
        }
    }
    let pids = pids.unwrap();
    (
        pids["child"].as_u64().unwrap() as u32,
        pids["grandchild"].as_u64().unwrap() as u32,
        argv.unwrap(),
    )
}

fn alive(pid: u32) -> bool {
    #[cfg(windows)]
    {
        let out = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH"])
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).contains(&pid.to_string())
    }
    #[cfg(not(windows))]
    {
        Path::new(&format!("/proc/{pid}")).exists()
    }
}

fn assert_gone(pid: u32) {
    let deadline = Instant::now() + WAIT;
    while alive(pid) {
        assert!(Instant::now() < deadline, "process {pid} is still running");
        thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn a_saved_chat_passes_its_session_on_the_command_line() {
    let dir = tempfile::tempdir().unwrap();
    let host = real_host(dir.path());
    let session = dir.path().join("session.jsonl");

    let (child, grandchild, argv) = open_and_read(&host, ChatKey::Saved(session.clone()));

    assert_eq!(
        argv["args"],
        json!(["--mode", "rpc", "--session", session.to_string_lossy()])
    );
    assert!(alive(child) && alive(grandchild));
    host.shutdown();
    assert_gone(child);
    assert_gone(grandchild);
}

#[test]
fn a_new_chat_is_spawned_without_a_session() {
    let dir = tempfile::tempdir().unwrap();
    let host = real_host(dir.path());

    let (child, grandchild, argv) = open_and_read(&host, ChatKey::New(1));

    assert_eq!(argv["args"], json!(["--mode", "rpc"]));
    host.shutdown();
    assert_gone(child);
    assert_gone(grandchild);
}

#[test]
fn shutdown_stops_every_child_and_its_descendants() {
    let dir = tempfile::tempdir().unwrap();
    let host = real_host(dir.path());
    let (child_a, grandchild_a, _) = open_and_read(&host, ChatKey::Saved(dir.path().join("a")));
    let (child_b, grandchild_b, _) = open_and_read(&host, ChatKey::Saved(dir.path().join("b")));

    host.shutdown();

    for pid in [child_a, grandchild_a, child_b, grandchild_b] {
        assert_gone(pid);
    }
}

#[test]
fn dropping_the_host_leaves_no_child_behind() {
    let dir = tempfile::tempdir().unwrap();
    let host = real_host(dir.path());
    let (child, grandchild, _) = open_and_read(&host, ChatKey::Saved(dir.path().join("a")));

    drop(host);

    assert_gone(child);
    assert_gone(grandchild);
}
