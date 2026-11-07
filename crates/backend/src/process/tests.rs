//! These tests drive real child processes (`node` running small scripts that
//! stand in for `gentle-shell`), so the pipes, threads and kill paths are the
//! production ones.

use super::*;
use gentle_core::rpc::RpcEvent;
use std::fs;
use std::path::Path;

const TIMEOUT: Duration = Duration::from_secs(10);

fn script_launcher(dir: &Path, source: &str) -> Launcher {
    let script = dir.join("fake.mjs");
    fs::write(&script, source).unwrap();
    Launcher {
        program: "node".into(),
        args: vec![script.into()],
    }
}

fn next(process: &RpcProcess) -> ProcessEvent {
    process.events().recv_timeout(TIMEOUT).expect("an event")
}

#[test]
fn stdout_lines_are_decoded_in_order_and_end_with_closed() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = script_launcher(
        dir.path(),
        r#"process.stdout.write('{"type":"agent_start"}\r\nnot json\n{"type":"agent_settled"}');"#,
    );
    let process = RpcProcess::spawn(&launcher, &SpawnOptions::default()).unwrap();
    assert_eq!(
        next(&process),
        ProcessEvent::Output(Decoded::Event(RpcEvent::AgentStart))
    );
    assert_eq!(
        next(&process),
        ProcessEvent::Output(Decoded::Unknown("not json".into()))
    );
    // The last line has no trailing newline: flushed at EOF.
    assert_eq!(
        next(&process),
        ProcessEvent::Output(Decoded::Event(RpcEvent::AgentSettled))
    );
    assert_eq!(next(&process), ProcessEvent::Closed);
}

#[test]
fn a_multibyte_character_split_across_writes_survives() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = script_launcher(
        dir.path(),
        r#"
const bytes = Buffer.from('{"type":"x","t":"añ€"}\n');
const cut = bytes.indexOf(0xe2) + 1; // inside the 3-byte euro sign
process.stdout.write(bytes.subarray(0, cut));
setTimeout(() => process.stdout.write(bytes.subarray(cut)), 100);
"#,
    );
    let process = RpcProcess::spawn(&launcher, &SpawnOptions::default()).unwrap();
    assert_eq!(
        next(&process),
        ProcessEvent::Output(Decoded::Unknown(r#"{"type":"x","t":"añ€"}"#.into()))
    );
}

#[test]
fn commands_reach_stdin_and_mode_rpc_is_passed() {
    let dir = tempfile::tempdir().unwrap();
    // Echo argv and every stdin line back as JSON strings on stdout.
    let launcher = script_launcher(
        dir.path(),
        r#"
console.log(JSON.stringify({ type: 'argv', args: process.argv.slice(2) }));
process.stdin.setEncoding('utf8');
let buf = '';
process.stdin.on('data', (d) => {
  buf += d;
  let i;
  while ((i = buf.indexOf('\n')) >= 0) {
    console.log(JSON.stringify({ type: 'echo', line: buf.slice(0, i) }));
    buf = buf.slice(i + 1);
  }
});
"#,
    );
    let options = SpawnOptions {
        args: vec!["--isolated".into()],
        ..SpawnOptions::default()
    };
    let mut process = RpcProcess::spawn(&launcher, &options).unwrap();
    let ProcessEvent::Output(Decoded::Unknown(argv)) = next(&process) else {
        panic!("argv line is an unmodeled event")
    };
    assert_eq!(
        argv,
        r#"{"type":"argv","args":["--mode","rpc","--isolated"]}"#
    );

    process
        .send(&RpcCommand::Prompt {
            message: "hola".into(),
        })
        .unwrap();
    let ProcessEvent::Output(Decoded::Unknown(echo)) = next(&process) else {
        panic!("echo line is an unmodeled event")
    };
    assert_eq!(
        echo,
        r#"{"type":"echo","line":"{\"message\":\"hola\",\"type\":\"prompt\"}"}"#
    );
}

#[test]
fn stderr_is_drained_as_its_own_events() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = script_launcher(dir.path(), "console.error('boom');");
    let process = RpcProcess::spawn(&launcher, &SpawnOptions::default()).unwrap();
    let mut seen = Vec::new();
    loop {
        match next(&process) {
            ProcessEvent::Closed => break,
            other => seen.push(other),
        }
    }
    assert_eq!(seen, vec![ProcessEvent::Stderr("boom".into())]);
}

#[test]
fn shutdown_closes_stdin_and_a_cooperative_child_exits_gracefully() {
    let dir = tempfile::tempdir().unwrap();
    // Exits when stdin ends, like pi.
    let launcher = script_launcher(
        dir.path(),
        "process.stdin.resume(); process.stdin.on('end', () => process.exit(0));",
    );
    let process = RpcProcess::spawn(&launcher, &SpawnOptions::default()).unwrap();
    let outcome = process.shutdown(TIMEOUT).unwrap();
    assert!(matches!(outcome, Shutdown::Graceful(status) if status.success()));
}

/// Spawns a grandchild that outlives its parent unless the tree is killed,
/// reports both pids, then ignores stdin EOF.
const STUBBORN_TREE: &str = r#"
import { spawn } from 'node:child_process';
const grandchild = spawn(process.execPath, ['-e', 'setInterval(() => {}, 1000)'], { stdio: 'ignore', detached: true });
console.log(JSON.stringify({ type: 'pids', child: process.pid, grandchild: grandchild.pid }));
setInterval(() => {}, 1000);
"#;

fn pids(process: &RpcProcess) -> (u32, u32) {
    let ProcessEvent::Output(Decoded::Unknown(line)) = next(process) else {
        panic!("pids line")
    };
    let value: serde_json::Value = serde_json::from_str(&line).unwrap();
    (
        value["child"].as_u64().unwrap() as u32,
        value["grandchild"].as_u64().unwrap() as u32,
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
    let deadline = Instant::now() + TIMEOUT;
    while alive(pid) {
        assert!(Instant::now() < deadline, "process {pid} is still running");
        thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn shutdown_kills_a_stubborn_child_and_its_descendants() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = script_launcher(dir.path(), STUBBORN_TREE);
    let process = RpcProcess::spawn(&launcher, &SpawnOptions::default()).unwrap();
    let (child, grandchild) = pids(&process);
    assert!(alive(child) && alive(grandchild));

    let outcome = process.shutdown(Duration::from_millis(300)).unwrap();
    assert_eq!(outcome, Shutdown::Killed);
    assert_gone(child);
    assert_gone(grandchild);
}

#[test]
fn dropping_the_process_leaves_no_orphans() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = script_launcher(dir.path(), STUBBORN_TREE);
    let process = RpcProcess::spawn(&launcher, &SpawnOptions::default()).unwrap();
    let (child, grandchild) = pids(&process);
    drop(process);
    assert_gone(child);
    assert_gone(grandchild);
}

#[test]
fn spawning_a_missing_program_is_an_error() {
    let launcher = Launcher {
        program: "definitely-not-a-real-program".into(),
        args: vec![],
    };
    assert!(RpcProcess::spawn(&launcher, &SpawnOptions::default()).is_err());
}
