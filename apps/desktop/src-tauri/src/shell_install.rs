//! Installing gentle-shell when the machine has none (shell-choice S4).
//!
//! The app never installs anything on its own: this runs only after the user
//! pressed "Install gentle-shell" on the permission screen. The install is one
//! command behind a small seam, so the tests run `node` instead of `npm` and the
//! real window can be checked with a stand-in (`GENTLE_DESK_NPM`).
//!
//! The command's output is read to the end by two threads (a full pipe would
//! stall it) and each line reaches the caller's sink as it arrives, so the
//! screen can say what npm is doing instead of only that it is running. Nothing
//! is kept per line: the sink takes it and the line is dropped. The only buffers
//! here are bounded - the tail of the output, which a failure reports, and the
//! line being read, which a command that never ends a line cannot grow.

use std::ffi::OsString;
use std::io::Read;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

/// What runs the install, handing each line the command prints to the sink as
/// it arrives. `Err` is the sentence the permission screen shows.
pub trait Installer: Send + Sync {
    /// Runs the install to its end. The sink lives no longer than this call: the
    /// threads that read the command's pipes are scoped to it, so nothing here
    /// outlives the install.
    fn install(&self, on_line: &LineSink<'_>) -> Result<(), String>;
}

/// What the install thread does with one line of the command's output: the
/// screen shows the last one, so nothing here keeps them. Both pipes of the
/// command feed the same sink, from their own threads.
pub type LineSink<'a> = dyn Fn(&str) + Send + Sync + 'a;

/// How much of the command's output is kept, per stream.
const KEPT_BYTES: usize = 64 * 1024;
/// The longest failure message: the end of the output, cut to this many bytes.
const MESSAGE_BYTES: usize = 2_000;
/// The longest one line is held before its start is dropped: the end of a line
/// is what says what happened, so that is the end that is kept. A command that
/// prints a megabyte without a newline costs this much and no more.
const LINE_BYTES: usize = 2_000;
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
    fn install(&self, on_line: &LineSink<'_>) -> Result<(), String> {
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
        // The two pipes are read on threads of their own, which is what keeps a
        // full pipe from stalling the command; each line they complete reaches
        // the sink as it is read, never at the end.
        let (status, output) = std::thread::scope(|scope| {
            let out = child
                .stdout
                .take()
                .map(|reader| scope.spawn(|| drain(reader, on_line)));
            let err = child
                .stderr
                .take()
                .map(|reader| scope.spawn(|| drain(reader, on_line)));

            let status = wait(&mut child)?;
            let mut output = Vec::new();
            for handle in [err, out].into_iter().flatten() {
                output.extend(handle.join().unwrap_or_default());
                output.push(b'\n');
            }
            Ok::<_, String>((status, output))
        })?;
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

/// Waits for the command to end, stopping it when it takes longer than
/// [`TIMEOUT`].
fn wait(child: &mut Child) -> Result<ExitStatus, String> {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(50)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("the install did not finish in 20 minutes and was stopped".to_owned());
            }
            Err(error) => return Err(format!("could not wait for the install: {error}")),
        }
    }
}

/// Reads a stream to its end, handing every line it completes to the sink and
/// keeping only the last bytes for a failure message.
fn drain<R: Read>(mut reader: R, on_line: &LineSink<'_>) -> Vec<u8> {
    let mut kept: Vec<u8> = Vec::new();
    let mut lines = LineNormaliser::default();
    let mut chunk = [0u8; 8192];
    while let Ok(read) = reader.read(&mut chunk) {
        if read == 0 {
            break;
        }
        kept.extend_from_slice(&chunk[..read]);
        if kept.len() > KEPT_BYTES {
            kept.drain(..kept.len() - KEPT_BYTES);
        }
        for line in lines.push(&chunk[..read]) {
            on_line(&line);
        }
    }
    // A command that ends mid-line still printed that line.
    for line in lines.finish() {
        on_line(&line);
    }
    kept
}

/// The normaliser between a command's output and a screen: it turns the bytes
/// as they are read into the lines a person can read.
///
/// npm redraws one line with carriage returns, so a `\r` that is not followed by
/// `\n` means "this line starts again": what came before it is dropped, and
/// `\r\n` (or a bare `\n`) ends the line. Empty lines are answered as nothing,
/// and one line is never held past [`LINE_BYTES`].
#[derive(Debug, Default)]
pub struct LineNormaliser {
    /// The line so far, without its end.
    line: Vec<u8>,
    /// A `\r` was read and what follows it has not decided yet whether it ends
    /// the line (`\n`) or starts it again: held, so a read may split the two.
    after_cr: bool,
}

impl LineNormaliser {
    /// Feeds one chunk of output and answers the lines it completed, in order,
    /// empty ones left out.
    pub fn push(&mut self, chunk: &[u8]) -> Vec<String> {
        let mut lines = Vec::new();
        for &byte in chunk {
            if self.after_cr {
                self.after_cr = false;
                if byte != b'\n' {
                    // A redraw: what was on the line before the `\r` is gone,
                    // which is what npm means by it.
                    self.line.clear();
                }
            }
            match byte {
                b'\r' => self.after_cr = true,
                b'\n' => self.ended(&mut lines),
                byte => {
                    self.line.push(byte);
                    if self.line.len() > LINE_BYTES {
                        self.trim();
                    }
                }
            }
        }
        lines
    }

    /// Answers the line that never ended, if there is one, and forgets it: a
    /// command that stops mid-line still printed what it was saying.
    pub fn finish(&mut self) -> Vec<String> {
        let mut lines = Vec::new();
        self.after_cr = false;
        self.ended(&mut lines);
        lines
    }

    /// Ends the line being read, if it holds anything, and answers it.
    fn ended(&mut self, lines: &mut Vec<String>) {
        if self.line.is_empty() {
            return;
        }
        let line = String::from_utf8_lossy(&self.line).into_owned();
        self.line.clear();
        let line = line.trim_end();
        if !line.is_empty() {
            lines.push(line.to_owned());
        }
    }

    /// Drops the start of a line past [`LINE_BYTES`], on a character boundary, so
    /// a line that never ends costs a bounded buffer.
    fn trim(&mut self) {
        let mut start = self.line.len() - LINE_BYTES;
        while start < self.line.len() && (self.line[start] & 0xC0) == 0x80 {
            start += 1;
        }
        self.line.drain(..start);
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Feeds the chunks in order and answers every line they completed, so a
    /// test reads as one feed however the reads happened to split it.
    fn feed(normaliser: &mut LineNormaliser, chunks: &[&str]) -> Vec<String> {
        let mut lines = Vec::new();
        for chunk in chunks {
            lines.extend(normaliser.push(chunk.as_bytes()));
        }
        lines
    }

    #[test]
    fn whole_lines_come_out_in_order() {
        let mut normaliser = LineNormaliser::default();

        assert_eq!(
            feed(&mut normaliser, &["one\ntwo\nthr", "ee\n"]),
            ["one", "two", "three"]
        );
        assert!(normaliser.finish().is_empty());
    }

    /// npm redraws one line with carriage returns, so a `\r` that is not
    /// followed by `\n` starts the line again instead of ending it.
    #[test]
    fn a_carriage_return_redraws_the_line_it_does_not_end_it() {
        let mut normaliser = LineNormaliser::default();

        assert_eq!(
            feed(
                &mut normaliser,
                &["npm WARN 10%\rnpm WARN 50%\rnpm WARN 100%\n"]
            ),
            ["npm WARN 100%"]
        );
    }

    /// `\r\n` is an end of line, not a redraw, even when a read splits it.
    #[test]
    fn a_carriage_return_and_a_newline_end_the_line() {
        let mut normaliser = LineNormaliser::default();

        assert_eq!(
            feed(&mut normaliser, &["one\r\ntwo\r", "\nthree\r\n"]),
            ["one", "two", "three"]
        );
    }

    #[test]
    fn empty_lines_are_not_lines() {
        let mut normaliser = LineNormaliser::default();

        assert_eq!(feed(&mut normaliser, &["\n\n\ntwo\n\n", "\r\n\n"]), ["two"]);
        assert!(normaliser.finish().is_empty());
    }

    /// One line is never held past [`LINE_BYTES`]: a command that prints without
    /// ever ending a line costs a bounded buffer, and the end of the line is the
    /// part that says what happened.
    #[test]
    fn a_line_that_never_ends_is_bounded() {
        let mut normaliser = LineNormaliser::default();
        let mut lines = Vec::new();
        for _ in 0..100 {
            lines.extend(normaliser.push(&[b'x'; 1_000]));
        }

        assert!(lines.is_empty(), "no line ended, so none came out");
        let line = normaliser.finish().pop().expect("the line never ended");
        assert_eq!(line.len(), LINE_BYTES, "100 KB of one line, kept bounded");
        assert!(line.bytes().all(|byte| byte == b'x'));
    }

    /// A command that stops mid-line still printed that line.
    #[test]
    fn a_last_line_that_never_ended_is_still_reported() {
        let mut normaliser = LineNormaliser::default();

        assert_eq!(feed(&mut normaliser, &["one\nnpm ERR! 42"]), ["one"]);
        assert_eq!(normaliser.finish(), ["npm ERR! 42"]);
        assert!(normaliser.finish().is_empty(), "it is answered once");
    }

    /// A read may split a multi-byte character; the line still comes out whole.
    #[test]
    fn a_character_split_across_reads_is_not_lost() {
        let mut normaliser = LineNormaliser::default();
        let line = "caf\u{e9}\n".as_bytes();

        assert!(normaliser.push(&line[..4]).is_empty());
        assert_eq!(normaliser.push(&line[4..]), ["caf\u{e9}"]);
    }
}
