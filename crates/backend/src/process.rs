//! The `gentle-shell --mode rpc` child process.
//!
//! Threads (std only, no async runtime): one reads stdout in raw chunks, feeds
//! the bytes to [`LineSplitter`] and forwards each decoded line; one drains
//! stderr so the child can never block on a full pipe. Both report through a
//! single channel, so the owner consumes one ordered stream of
//! [`ProcessEvent`]s and never touches a pipe.
//!
//! Closing never orphans anything: stdin is closed first (pi exits on EOF),
//! and a child that does not leave within the grace period is killed together
//! with its descendants. `Drop` does the same.
//!
//! Writing is its own handle ([`RpcWriter`]) because a pipe write blocks when the
//! child stops reading: an owner can hand one out and write with no lock of its
//! own held across the write.

use std::ffi::OsString;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use gentle_core::rpc::{decode_line, Decoded, LineSplitter, RpcCommand};

use crate::launcher::Launcher;

mod tree;
mod writer;

pub use writer::RpcWriter;

use tree::{hide_console_window, kill_tree};
use writer::stdin_closed;

/// How to start the child beyond the resolved launcher.
#[derive(Debug, Clone, Default)]
pub struct SpawnOptions {
    /// Extra arguments after `--mode rpc` (e.g. `--isolated`).
    pub args: Vec<OsString>,
    /// Extra environment variables for the child.
    pub env: Vec<(OsString, OsString)>,
    pub cwd: Option<PathBuf>,
}

/// One item of the child's ordered output stream.
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessEvent {
    /// A stdout line, decoded.
    Output(Decoded),
    /// A stderr line (diagnostics, never protocol).
    Stderr(String),
    /// stdout reached EOF: the child is gone or closing. Always the last event.
    Closed,
}

/// How [`RpcProcess::shutdown`] ended the child.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shutdown {
    /// It exited by itself after stdin was closed.
    Graceful(ExitStatus),
    /// It outlived the grace period and was killed with its descendants.
    Killed,
}

pub struct RpcProcess {
    child: Child,
    /// Shared with every [`RpcWriter`] taken from this process: a write and the
    /// end-of-file that [`RpcProcess::stop`] sends must not race on the pipe.
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    events: Receiver<ProcessEvent>,
    readers: Vec<JoinHandle<()>>,
}

impl RpcProcess {
    pub fn spawn(launcher: &Launcher, options: &SpawnOptions) -> io::Result<Self> {
        let mut command = Command::new(&launcher.program);
        command
            .args(&launcher.args)
            .args(["--mode", "rpc"])
            .args(&options.args)
            .envs(options.env.iter().map(|(k, v)| (k, v)))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(cwd) = &options.cwd {
            command.current_dir(cwd);
        }
        hide_console_window(&mut command);

        let mut child = command.spawn()?;
        let stdin = child.stdin.take();
        let stdout = child.stdout.take().expect("stdout is piped");
        let stderr = child.stderr.take().expect("stderr is piped");

        let (tx, events) = mpsc::channel();
        let out_tx = tx.clone();
        let stdout_reader = thread::spawn(move || {
            pump(stdout, |line| {
                let _ = out_tx.send(ProcessEvent::Output(decode_line(&line)));
            });
            let _ = out_tx.send(ProcessEvent::Closed);
        });
        let stderr_reader = thread::spawn(move || {
            pump(stderr, |line| {
                let _ = tx.send(ProcessEvent::Stderr(line));
            });
        });

        Ok(Self {
            child,
            stdin: Arc::new(Mutex::new(stdin)),
            events,
            readers: vec![stdout_reader, stderr_reader],
        })
    }

    /// The ordered output stream.
    pub fn events(&self) -> &Receiver<ProcessEvent> {
        &self.events
    }

    /// Hands the output stream to another owner (a UI subscription, say).
    /// What stays behind is a closed stream, so there is never a second consumer.
    pub fn take_events(&mut self) -> Receiver<ProcessEvent> {
        std::mem::replace(&mut self.events, mpsc::channel().1)
    }

    /// Writes one command line to the child's stdin.
    pub fn send(&mut self, command: &RpcCommand) -> io::Result<()> {
        self.writer().ok_or_else(stdin_closed)?.send(command)
    }

    /// A handle that writes command lines on its own, or `None` once stdin is
    /// closed. The caller writes with the handle and not with this struct, so the
    /// pipe write never happens under a lock it holds for other reasons.
    pub fn writer(&self) -> Option<RpcWriter> {
        let stdin = self.stdin.lock().unwrap_or_else(|p| p.into_inner());
        stdin.as_ref()?;
        Some(RpcWriter::new(Arc::clone(&self.stdin)))
    }

    /// Closes stdin, waits up to `grace` for a clean exit, then kills the
    /// child's whole process tree.
    pub fn shutdown(mut self, grace: Duration) -> io::Result<Shutdown> {
        self.stop(grace)
    }

    fn stop(&mut self, grace: Duration) -> io::Result<Shutdown> {
        // EOF on stdin: pi's cue to exit. Taking the slot waits for a write that
        // is still in flight, so a half-written line is never sent after it.
        self.stdin.lock().unwrap_or_else(|p| p.into_inner()).take();
        let deadline = Instant::now() + grace;
        let outcome = loop {
            if let Some(status) = self.child.try_wait()? {
                break Shutdown::Graceful(status);
            }
            if Instant::now() >= deadline {
                kill_tree(&mut self.child);
                self.child.wait()?;
                break Shutdown::Killed;
            }
            thread::sleep(Duration::from_millis(10));
        };
        // Pipes close with the child, so the readers end on their own.
        for reader in self.readers.drain(..) {
            let _ = reader.join();
        }
        Ok(outcome)
    }
}

impl Drop for RpcProcess {
    fn drop(&mut self) {
        if self.readers.is_empty() {
            return; // already shut down
        }
        let _ = self.stop(Duration::from_secs(2));
    }
}

/// Reads `source` to EOF in raw chunks, calling `on_line` per complete line.
fn pump(mut source: impl Read, mut on_line: impl FnMut(String)) {
    let mut splitter = LineSplitter::new();
    let mut chunk = [0u8; 8192];
    loop {
        match source.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => splitter
                .push(&chunk[..n])
                .into_iter()
                .for_each(&mut on_line),
        }
    }
    splitter.flush().into_iter().for_each(&mut on_line);
}

#[cfg(test)]
mod tests;
