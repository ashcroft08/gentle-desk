//! The git branch the foreground chat's working directory is on
//! (design/composer-dock S2, R3, R4): two `git rev-parse` reads, one answer.
//!
//! The composer's branch chip must never lie and must never be a wall for the
//! user, so this module is built out of two rules:
//!
//! - EVERY failure is `None`. Git not installed, a folder outside any
//!   repository, a bare repository, no permission, an empty answer, a non-zero
//!   exit: all of them are the same answer, which is "there is no branch to
//!   draw", and the composer simply draws no chip (R3).
//! - The answer is read ONCE per working directory and kept. A push happens for
//!   every token a child writes, and a process per push would be absurd: the
//!   [`Branches`] cache holds what the one read answered for each directory, so
//!   the read runs on a worker thread ([`Lookup`]) and every push after it is a
//!   map lookup.
//!
//! The two questions are `rev-parse --is-inside-work-tree` (is there a work tree
//! at all) and `rev-parse --abbrev-ref HEAD` (which branch). A detached HEAD
//! answers `HEAD`, which is not a branch: the short sha is read instead and the
//! chip shows that, so the text is never a word that means "nothing".

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Mutex, MutexGuard};

use crate::contract::BranchView;

/// What `--abbrev-ref` answers when HEAD is not on a branch.
const DETACHED: &str = "HEAD";

/// One `git` invocation's answer: whether it exited zero and what it wrote.
///
/// Public so the decision below can be tested with the exact bytes git writes,
/// without a repository, a child process or a machine that has git.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Answer {
    /// Whether git exited zero.
    pub success: bool,
    /// What git wrote on stdout.
    pub stdout: String,
}

impl Answer {
    /// The one line git answered, trimmed, when it succeeded and said
    /// something. A failed invocation and an empty output are the same thing
    /// here: no answer at all.
    fn line(&self) -> Option<&str> {
        if !self.success {
            return None;
        }
        let line = self.stdout.trim();
        (!line.is_empty()).then_some(line)
    }
}

/// The branch `inside` and `head` describe, or `None` when there is none.
///
/// `sha` is asked only for a detached HEAD, and it is a callback so this rule
/// stays testable without a repository: it is the one case that needs a third
/// read (`rev-parse --short HEAD`), and the case where a wrong answer would put
/// the word "HEAD" on screen.
pub fn decide(
    inside: &Answer,
    head: &Answer,
    sha: impl FnOnce() -> Option<String>,
) -> Option<String> {
    // Only a work tree has a branch: outside one git says `false` (or refuses
    // outright), and a bare repository has branches but no working directory to
    // be on one of. R3's answer is the same in all three cases: nothing.
    if inside.line() != Some("true") {
        return None;
    }
    let head = head.line()?;
    if head != DETACHED {
        return Some(head.to_owned());
    }
    // A detached HEAD answers `HEAD`, which names no branch. The short sha is
    // read instead, so the chip shows where the tree really is; if even that
    // read refuses, there is nothing true to draw.
    sha()
}

/// The branch of the work tree `dir` is in, read now.
///
/// Never an error: every way git can refuse answers `None`, which is what makes
/// the composer draw no chip (R3).
pub fn read(dir: &Path) -> Option<BranchView> {
    let inside = ask(dir, &["rev-parse", "--is-inside-work-tree"]);
    let head = ask(dir, &["rev-parse", "--abbrev-ref", "HEAD"]);
    let name = decide(&inside, &head, || {
        let short = ask(dir, &["rev-parse", "--short", "HEAD"]);
        short.line().map(str::to_owned)
    })?;
    Some(BranchView {
        name,
        path: dir.to_string_lossy().into_owned(),
    })
}

/// One `git` invocation in `dir`, with no console window opened over the app.
///
/// Every failure - git not installed, no permission, not a repository, a
/// non-zero exit - is the same empty answer, which is what keeps the caller
/// free of error handling.
fn ask(dir: &Path, args: &[&str]) -> Answer {
    let mut command = Command::new("git");
    command
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // No console window flashing over the app.
        command.creation_flags(0x0800_0000);
    }
    match command.output() {
        Ok(output) if output.status.success() => Answer {
            success: true,
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        },
        // A refusal says nothing, and a failure to spawn at all - no git on this
        // machine - is the same answer as git answering "not a repository".
        _ => Answer::default(),
    }
}

/// What the cache knows about one working directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lookup {
    /// The answer, which itself may be "there is no branch here".
    Read(Option<BranchView>),
    /// A read is already running for it: nothing to start, nothing to draw yet.
    Waiting,
    /// Nobody has asked yet: the caller starts the one read.
    Missing,
}

/// The branch of every working directory this run has looked at.
///
/// One entry per directory, `None` included, for the whole run: a directory git
/// had nothing to say about is not asked again, and the read itself happens off
/// the window's thread, so no push ever waits for a process. Nothing invalidates
/// an entry while the app runs - a `git checkout` in another window shows up the
/// next time the app starts.
#[derive(Debug, Default)]
pub struct Branches {
    inner: Mutex<Inner>,
}

#[derive(Debug, Default)]
struct Inner {
    /// The answer per working directory, "no branch" included.
    answers: HashMap<String, Option<BranchView>>,
    /// The directories a read is running for right now.
    asking: HashSet<String>,
}

impl Branches {
    /// The answer for `dir`, claiming the one read when nobody has asked yet.
    pub fn lookup(&self, dir: &str) -> Lookup {
        let mut inner = self.inner();
        if let Some(answer) = inner.answers.get(dir) {
            return Lookup::Read(answer.clone());
        }
        // Only the caller that puts the claim in starts the read; every push
        // that arrives while it runs only waits for it.
        if !inner.asking.insert(dir.to_owned()) {
            return Lookup::Waiting;
        }
        Lookup::Missing
    }

    /// Stores what the read answered for `dir` and releases the claim, so the
    /// next lookup is a hit whatever the answer was.
    pub fn record(&self, dir: &str, answer: Option<BranchView>) {
        let mut inner = self.inner();
        inner.asking.remove(dir);
        inner.answers.insert(dir.to_owned(), answer);
    }

    /// The cache, a poisoned mutex included: nothing here may panic because a
    /// thread died holding it.
    fn inner(&self) -> MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    /// A git that answered `stdout` and exited zero.
    fn ok(stdout: &str) -> Answer {
        Answer {
            success: true,
            stdout: stdout.to_owned(),
        }
    }

    /// A git that refused: not installed, not a repository, no permission.
    fn refused() -> Answer {
        Answer::default()
    }

    /// The sha read of a detached HEAD, and whether it was asked at all.
    fn sha<'a>(
        answer: Option<&'static str>,
        asked: &'a Cell<bool>,
    ) -> impl FnOnce() -> Option<String> + 'a {
        move || {
            asked.set(true);
            answer.map(str::to_owned)
        }
    }

    /// The two reads a branch on `name` makes.
    fn on_branch(name: &str) -> Option<String> {
        let asked = Cell::new(false);
        decide(&ok("true\n"), &ok(&format!("{name}\n")), sha(None, &asked))
    }

    #[test]
    fn a_branch_is_its_name_and_the_sha_is_never_asked_for() {
        let asked = Cell::new(false);
        let branch = decide(&ok("true\n"), &ok("feat/composer-dock\n"), sha(None, &asked));

        assert_eq!(branch.as_deref(), Some("feat/composer-dock"));
        assert!(!asked.get(), "a branch needs no third read");
    }

    #[test]
    fn a_slash_in_the_branch_name_is_kept_as_git_spells_it() {
        assert_eq!(on_branch("release/0.7.0").as_deref(), Some("release/0.7.0"));
    }

    #[test]
    fn a_detached_head_draws_the_short_sha_instead_of_the_word_head() {
        // `HEAD` is what `--abbrev-ref` answers on a detached HEAD: it names no
        // branch, so the chip shows the sha rather than a word that means none.
        let asked = Cell::new(false);
        let branch = decide(&ok("true\n"), &ok("HEAD\n"), sha(Some("743f4fe"), &asked));

        assert_eq!(branch.as_deref(), Some("743f4fe"));
        assert!(asked.get(), "the sha is exactly what a detached HEAD needs");
    }

    #[test]
    fn a_detached_head_with_no_sha_to_read_draws_nothing() {
        let asked = Cell::new(false);

        assert_eq!(decide(&ok("true"), &ok("HEAD"), sha(None, &asked)), None);
        assert!(asked.get());
    }

    #[test]
    fn a_directory_outside_a_work_tree_has_no_branch() {
        // A bare repository answers `false`, and R3's rule is the same one R4's
        // is: absent, never a placeholder.
        let asked = Cell::new(false);

        assert_eq!(decide(&ok("false\n"), &ok("main\n"), sha(None, &asked)), None);
        assert!(!asked.get());
    }

    #[test]
    fn a_refused_read_is_no_branch() {
        let asked = Cell::new(false);

        // No git on this machine, no permission, not a repository: whichever of
        // the two reads refused, the answer is the same.
        assert_eq!(decide(&refused(), &ok("main\n"), sha(None, &asked)), None);
        assert_eq!(decide(&ok("true\n"), &refused(), sha(None, &asked)), None);
        assert!(!asked.get());
    }

    #[test]
    fn an_empty_answer_is_no_branch() {
        let asked = Cell::new(false);

        assert_eq!(decide(&ok("true\n"), &ok("   \n"), sha(None, &asked)), None);
        assert_eq!(decide(&ok("\n"), &ok("main\n"), sha(None, &asked)), None);
    }

    #[test]
    fn one_directory_is_read_once_and_never_asked_again() {
        let cache = Branches::default();
        let dir = "/dev/proyectos/gentle-desk";
        let branch = BranchView {
            name: "feat/composer-dock".to_owned(),
            path: dir.to_owned(),
        };

        // The first push of a chat claims the read; the pushes that follow it
        // only wait, so one chat open never starts a second process.
        assert_eq!(cache.lookup(dir), Lookup::Missing);
        assert_eq!(cache.lookup(dir), Lookup::Waiting);

        cache.record(dir, Some(branch.clone()));
        assert_eq!(cache.lookup(dir), Lookup::Read(Some(branch)));

        // The answer is reused for every push after it, and no read starts.
        assert_eq!(cache.lookup(dir), Lookup::Read(Some(BranchView {
            name: "feat/composer-dock".to_owned(),
            path: dir.to_owned(),
        })));
    }

    #[test]
    fn a_directory_with_no_branch_is_also_answered_once() {
        let cache = Branches::default();
        let dir = "/dev/proyectos/outside";

        assert_eq!(cache.lookup(dir), Lookup::Missing);
        cache.record(dir, None);

        // "No branch here" is an answer too: it is not asked again on every
        // push, which is what a chat outside a repository would cost.
        assert_eq!(cache.lookup(dir), Lookup::Read(None));
        assert_eq!(cache.lookup(dir), Lookup::Read(None));
    }

    #[test]
    fn two_directories_keep_their_own_answers() {
        let cache = Branches::default();
        let one = "/dev/one";
        let two = "/dev/two";

        assert_eq!(cache.lookup(one), Lookup::Missing);
        assert_eq!(cache.lookup(two), Lookup::Missing);
        cache.record(one, Some(BranchView {
            name: "main".to_owned(),
            path: one.to_owned(),
        }));

        assert_eq!(
            cache.lookup(one),
            Lookup::Read(Some(BranchView {
                name: "main".to_owned(),
                path: one.to_owned(),
            }))
        );
        // The second one is still being read: its claim was its own.
        assert_eq!(cache.lookup(two), Lookup::Waiting);
    }
}
