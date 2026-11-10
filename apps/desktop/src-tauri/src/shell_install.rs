//! Installing gentle-shell when the machine has none (shell-choice S4).
//!
//! The app never installs anything on its own: this runs only after the user
//! pressed "Install gentle-shell" on the permission screen. The install is one
//! command behind a small seam, so the tests run `node` instead of `npm` and the
//! real window can be checked with a stand-in (`GENTLE_DESK_NPM`).
//!
//! The command's output is read to the end by two threads (a full pipe would
//! stall it) but only its tail is kept, and a failure is reported as the exit
//! code plus the last lines, bounded, because that is the part that says why.

use std::ffi::OsString;
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// What runs the install. `Err` is the sentence the permission screen shows.
pub trait Installer: Send + Sync {
    fn install(&self) -> Result<(), String>;
}

/// How much of the command's output is kept, per stream.
const KEPT_BYTES: usize = 64 * 1024;
/// The longest failure message: the end of the output, cut to this many bytes.
const MESSAGE_BYTES: usize = 2_000;
/// An install that has not ended by now is stopped and reported.
const TIMEOUT: Duration = Duration::from_secs(20 * 60);

/// Runs one command to the end.
pub struct CommandInstaller {
    program: OsString,
    args: Vec<OsString>,
}

impl CommandInstaller {
    pub fn new<S: Into<OsString>, I: IntoIterator<Item = A>, A: Into<OsString>>(
        program: S,
        args: I,
    ) -> Self {
        Self {
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
        }
    }

    /// `npm install -g gentle-pi`, or the stand-in `GENTLE_DESK_NPM` names (a
    /// `.mjs`/`.js` file is run with `node`).
    pub fn npm() -> Self {
        let args = ["install", "-g", "gentle-pi"];
        match std::env::var_os("GENTLE_DESK_NPM").filter(|value| !value.is_empty()) {
            Some(path) => {
                let is_script = std::path::Path::new(&path)
                    .extension()
                    .is_some_and(|ext| ext == "mjs" || ext == "js");
                if is_script {
                    let mut all = vec![path];
                    all.extend(args.iter().map(OsString::from));
                    Self::new("node", all)
                } else {
                    Self::new(path, args)
                }
            }
            None => Self::new(if cfg!(windows) { "npm.cmd" } else { "npm" }, args),
        }
    }
}

impl Installer for CommandInstaller {
    fn install(&self) -> Result<(), String> {
        let mut command = Command::new(&self.program);
        command
            .args(&self.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // No console window flashing over the app.
            command.creation_flags(0x0800_0000);
        }
        let mut child = command.spawn().map_err(|error| {
            format!(
                "could not start {}: {error}",
                self.program.to_string_lossy()
            )
        })?;
        let out = child.stdout.take().map(drain);
        let err = child.stderr.take().map(drain);

        let deadline = Instant::now() + TIMEOUT;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(50))
                }
                Ok(None) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(
                        "the install did not finish in 20 minutes and was stopped".to_owned()
                    );
                }
                Err(error) => return Err(format!("could not wait for the install: {error}")),
            }
        };
        let mut output = Vec::new();
        for handle in [err, out].into_iter().flatten() {
            output.extend(handle.join().unwrap_or_default());
            output.push(b'\n');
        }
        if status.success() {
            return Ok(());
        }
        let code = status.code().map_or_else(
            || "no exit code".to_owned(),
            |code| format!("exit code {code}"),
        );
        Err(format!(
            "the install ended with {code}\n{}",
            tail_of(&output)
        ))
    }
}

/// Reads a stream to its end on its own thread, keeping only the last bytes.
fn drain<R: Read + Send + 'static>(mut reader: R) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut kept: Vec<u8> = Vec::new();
        let mut chunk = [0u8; 8192];
        while let Ok(read) = reader.read(&mut chunk) {
            if read == 0 {
                break;
            }
            kept.extend_from_slice(&chunk[..read]);
            if kept.len() > KEPT_BYTES {
                kept.drain(..kept.len() - KEPT_BYTES);
            }
        }
        kept
    })
}

/// The end of `output` as text, at most [`MESSAGE_BYTES`], cut on a character.
fn tail_of(output: &[u8]) -> String {
    let text = String::from_utf8_lossy(output);
    let text = text.trim();
    if text.len() <= MESSAGE_BYTES {
        return text.to_owned();
    }
    let mut start = text.len() - MESSAGE_BYTES;
    while !text.is_char_boundary(start) {
        start += 1;
    }
    text[start..].to_owned()
}
