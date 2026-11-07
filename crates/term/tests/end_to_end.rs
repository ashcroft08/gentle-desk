//! End to end: the real `gentle-term` binary against the stand-in rpc child.
//!
//! `GENTLE_SHELL_BIN` points at `tools/fake-gentle-shell/fake.mjs`, a plain
//! Node script that speaks pi's JSONL rpc without any model, so these tests are
//! deterministic, cost nothing and never touch the network. Every case asserts
//! the whole observable contract: exit code, stdout and stderr.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// The stand-in child, relative to the repository root.
const FAKE: &str = "tools/fake-gentle-shell/fake.mjs";

/// A hang must fail the test instead of blocking the suite forever.
const TIMEOUT: Duration = Duration::from_secs(60);

/// One finished run of `gentle-term` and everything it printed.
struct Finished {
    status: ExitStatus,
    stdout: String,
    stderr: String,
}

impl Finished {
    fn code(&self) -> Option<i32> {
        self.status.code()
    }
}

/// `crates/term` -> the repository root.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the workspace root is two levels above crates/term")
        .to_path_buf()
}

/// Absolute path of the stand-in child.
fn fake_shell() -> String {
    repo_root().join(FAKE).to_str().expect("utf-8 path").into()
}

/// Runs the built binary with `args` and `GENTLE_SHELL_BIN` set to `bin`.
fn run_term(bin: &str, args: &[&str]) -> Finished {
    let mut child = Command::new(env!("CARGO_BIN_EXE_gentle-term"))
        .args(args)
        .env("GENTLE_SHELL_BIN", bin)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("gentle-term starts");

    let stdout = child.stdout.take().expect("stdout is piped");
    let stderr = child.stderr.take().expect("stderr is piped");
    // Both pipes are drained while the child runs, so it can never block on a full pipe.
    let out_reader = thread::spawn(move || read_all(stdout));
    let err_reader = thread::spawn(move || read_all(stderr));

    let status = wait_with_timeout(&mut child, TIMEOUT);
    let stdout = out_reader.join().expect("the stdout reader survives");
    let stderr = err_reader.join().expect("the stderr reader survives");
    Finished {
        status,
        stdout,
        stderr,
    }
}

fn read_all(mut pipe: impl Read) -> String {
    let mut text = String::new();
    pipe.read_to_string(&mut text)
        .expect("the pipe is readable");
    text
}

fn wait_with_timeout(child: &mut Child, timeout: Duration) -> ExitStatus {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait().expect("the child status is queryable") {
            return status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("gentle-term did not finish within {timeout:?}: it hung");
        }
        thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn a_plain_prompt_prints_the_echoed_reply_and_exits_zero() {
    let finished = run_term(&fake_shell(), &["hola"]);

    assert_eq!(finished.code(), Some(0), "stderr: {}", finished.stderr);
    assert_eq!(finished.stdout, "echo: hola\n");
    assert_eq!(finished.stderr, "");
}

#[test]
fn forwarded_flags_reach_the_child_and_never_become_prompt_words() {
    let finished = run_term(
        &fake_shell(),
        &[
            "--isolated",
            "--provider",
            "fake",
            "--model",
            "fake",
            "hola",
        ],
    );

    assert_eq!(finished.code(), Some(0), "stderr: {}", finished.stderr);
    assert_eq!(finished.stdout, "echo: hola\n");
    assert_eq!(finished.stderr, "");
}

#[test]
fn a_provider_failure_exits_one_and_never_prints_the_provider_text() {
    let finished = run_term(&fake_shell(), &["error"]);

    assert_eq!(finished.code(), Some(1), "stdout: {}", finished.stdout);
    assert!(
        finished
            .stderr
            .contains("model error: quota exceeded (fake)"),
        "stderr: {}",
        finished.stderr
    );
    assert!(
        finished.stderr.contains("rate limit reached (fake)"),
        "stderr: {}",
        finished.stderr
    );
    assert!(
        !finished.stdout.contains("Provider says no."),
        "stdout: {}",
        finished.stdout
    );
    assert_eq!(finished.stdout, "");
}

#[test]
fn dialogs_are_cancelled_and_the_child_sees_three_cancelled_answers() {
    let finished = run_term(&fake_shell(), &["dialog"]);

    assert_eq!(finished.code(), Some(0), "stderr: {}", finished.stderr);
    for id in ["d1", "d2", "d3"] {
        let line = format!("cancelled extension dialog: {id}");
        assert!(
            finished.stderr.contains(&line),
            "missing {line:?} in stderr: {}",
            finished.stderr
        );
    }
    // The reply echoes the raw answers, so this pins the exact wire shape the
    // client sends for a cancelled dialog (`serde_json`'s map sorts the keys).
    let expected = concat!(
        "RESPONSES: [",
        r#"{"cancelled":true,"id":"d1","type":"extension_ui_response"},"#,
        r#"{"cancelled":true,"id":"d2","type":"extension_ui_response"},"#,
        r#"{"cancelled":true,"id":"d3","type":"extension_ui_response"}"#,
        "]\n",
    );
    assert_eq!(finished.stdout, expected);
}

#[test]
fn a_missing_gentle_shell_bin_fails_before_spawning_anything() {
    let missing = repo_root().join("tools/fake-gentle-shell/does-not-exist.mjs");
    let missing = missing.to_str().expect("utf-8 path");

    let finished = run_term(missing, &["hola"]);

    assert_eq!(finished.code(), Some(1), "stdout: {}", finished.stdout);
    assert!(
        finished.stderr.contains("GENTLE_SHELL_BIN points to"),
        "stderr: {}",
        finished.stderr
    );
    assert!(
        finished.stderr.contains("does-not-exist.mjs"),
        "stderr: {}",
        finished.stderr
    );
    assert_eq!(finished.stdout, "");
}

#[test]
fn a_flood_turn_streams_all_three_hundred_messages_in_order() {
    let finished = run_term(&fake_shell(), &["flood"]);

    assert_eq!(finished.code(), Some(0), "stderr: {}", finished.stderr);
    let headings: Vec<&str> = finished
        .stdout
        .lines()
        .filter(|line| line.starts_with("## flood message "))
        .collect();
    assert_eq!(headings.len(), 300, "stdout: {}", finished.stdout);
    assert_eq!(headings[0], "## flood message 0");
    assert_eq!(headings[299], "## flood message 299");
}
