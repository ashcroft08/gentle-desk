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

use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use gentle_core::rpc::{decode_line, encode_command, Decoded, LineSplitter, RpcCommand};

use crate::launcher::Launcher;

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
    stdin: Option<ChildStdin>,
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
            stdin,
            events,
            readers: vec![stdout_reader, stderr_reader],
        })
    }

    /// The ordered output stream.
    pub fn events(&self) -> &Receiver<ProcessEvent> {
        &self.events
    }

    /// Writes one command line to the child's stdin.
    pub fn send(&mut self, command: &RpcCommand) -> io::Result<()> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "stdin is closed"))?;
        stdin.write_all(encode_command(command).as_bytes())?;
        stdin.flush()
    }

    /// Closes stdin, waits up to `grace` for a clean exit, then kills the
    /// child's whole process tree.
    pub fn shutdown(mut self, grace: Duration) -> io::Result<Shutdown> {
        self.stop(grace)
    }

    fn stop(&mut self, grace: Duration) -> io::Result<Shutdown> {
        self.stdin = None; // EOF on stdin: pi's cue to exit
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

#[cfg(windows)]
fn hide_console_window(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn hide_console_window(_command: &mut Command) {}

/// Kills the child and its descendants. `Child::kill` alone leaves
/// grandchildren (pi extensions, provider bridges) running on Windows.
fn kill_tree(child: &mut Child) {
    #[cfg(windows)]
    {
        let mut taskkill = Command::new("taskkill");
        taskkill
            .args(["/T", "/F", "/PID", &child.id().to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        hide_console_window(&mut taskkill);
        let _ = taskkill.status();
    }
    let _ = child.kill(); // no-op if taskkill already did it
}

#[cfg(test)]
mod tests;
