//! The environment check (0.9.0 general T2, S2-S6): one row per tool this app
//! and the shell depend on - Gentle Shell, pi, node, git - each with the state
//! it is in and the version it printed.
//!
//! Every fact is a read the app already knows how to run (S3): the shell's and
//! pi's versions are the Updates rows' own, taken from the SAME one
//! `gentle-shell --version` run, so the page never starts that probe a second
//! time; node and git are two bounded `--version` runs of the shape
//! `settings::run` already has - `without_console`, stdin null, an 8 s budget,
//! never the network, and never on the window's thread, because the whole
//! snapshot is gathered on a blocking worker.
//!
//! This module owns the two rules that turn a tool's words into a version, and
//! the mapping from a probe's answer to a row. What it never carries is the raw
//! answer: no path, no command line and no stderr travels up (S5), only the
//! token the tool printed and one of the three states the web words itself.

use std::io::ErrorKind;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gentle_core::i18n::Language;

use super::ToolProbe;
use crate::contract::{GeneralView, ToolKind, ToolRow, ToolState, UpdatesView};

/// How long one tool's own `--version` may take: the same 8 s budget the version
/// probe and the update check's GETs work to. A run that outlives it is killed
/// and read as unreadable (S3), so a wedged tool can never hold the dialog open.
pub const TOOL_TIMEOUT: Duration = Duration::from_secs(8);

/// How often the bounded run looks at the child it started: often enough that a
/// fast answer is not rounded up to a visible delay, rare enough that two probes
/// cost nothing while they wait.
const POLL: Duration = Duration::from_millis(10);

/// What one bounded `--version` run answered (S2, S5).
///
/// The three states are the seam's whole vocabulary: no message, no exit code
/// and no output travels with them, because none of those could be drawn without
/// naming the machine's paths and the tool's own words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolAnswer {
    /// The tool answered with the version it printed.
    Installed(String),
    /// The tool is not on this machine.
    Missing,
    /// It ran - or could not - and printed nothing this app can read as a
    /// version, a timeout included.
    Unreadable,
}

/// The real probe: one bounded `node --version` / `git --version` run each,
/// without a console window and without a stdin to wait on (S3). The shell and
/// pi are never asked here: their rows are the versions read the Updates rows
/// already made, and asking would start that launch a second time.
pub fn probe() -> ToolProbe {
    Arc::new(|tool: ToolKind, _language: Language| match tool {
        ToolKind::Node => read("node", TOOL_TIMEOUT, &["--version"], node_version),
        ToolKind::Git => read("git", TOOL_TIMEOUT, &["--version"], git_version),
        // Never asked by the load; answered honestly if it ever were.
        ToolKind::GentleShell | ToolKind::Pi => ToolAnswer::Unreadable,
    })
}

/// The environment check's slice (S2-S6), from the two facts the load already
/// has: the versions the Updates rows were drawn from, and one bounded probe per
/// tool that prints its own.
///
/// `launcher_found` is the shell view's own answer - the launcher the bridge
/// resolved - so the check never resolves it again: a machine where it does not
/// resolve has no shell, and the pi it carries comes with it.
///
/// `now_ms` is the load's ONE reading of the clock (S4): the same instant the
/// Updates rows were worded from, so the two pages can never disagree about when
/// they last read the machine.
pub fn view(
    updates: &UpdatesView,
    launcher_found: bool,
    tools: &ToolProbe,
    language: Language,
    now_ms: u64,
) -> GeneralView {
    GeneralView {
        tools: vec![
            carried(
                ToolKind::GentleShell,
                updates.shell.current.as_deref(),
                launcher_found,
            ),
            carried(ToolKind::Pi, updates.pi.current.as_deref(), launcher_found),
            row(ToolKind::Node, tools.as_ref()(ToolKind::Node, language)),
            row(ToolKind::Git, tools.as_ref()(ToolKind::Git, language)),
        ],
        checked_ms: Some(now_ms as f64),
    }
}

/// One row whose version the Updates read already answered (S3).
fn carried(tool: ToolKind, version: Option<&str>, launcher_found: bool) -> ToolRow {
    match version {
        Some(version) => installed(tool, version),
        // The launcher is not there: the shell, and the pi it runs, are missing
        // from this machine rather than unreadable.
        None if !launcher_found => missing(tool),
        // It is there and its one run printed no version for this tool, which is
        // exactly a version that could not be read (S5).
        None => unreadable(tool),
    }
}

/// One row a bounded probe answered.
fn row(tool: ToolKind, answer: ToolAnswer) -> ToolRow {
    match answer {
        ToolAnswer::Installed(version) if !version.trim().is_empty() => {
            installed(tool, version.trim())
        }
        // An empty version is not an installed one: the row says so instead of
        // drawing a blank beside "installed".
        ToolAnswer::Installed(_) | ToolAnswer::Unreadable => unreadable(tool),
        ToolAnswer::Missing => missing(tool),
    }
}

fn installed(tool: ToolKind, version: &str) -> ToolRow {
    ToolRow {
        tool,
        state: ToolState::Installed,
        version: Some(version.to_owned()),
    }
}

fn missing(tool: ToolKind) -> ToolRow {
    ToolRow {
        tool,
        state: ToolState::Missing,
        version: None,
    }
}

fn unreadable(tool: ToolKind) -> ToolRow {
    ToolRow {
        tool,
        state: ToolState::Unreadable,
        version: None,
    }
}

/// One bounded `<program> <args>` run, read by `version`'s own rule.
///
/// A program that is not there is [`ToolAnswer::Missing`] - the one spawn failure
/// that is a fact about the machine rather than about the run - and everything
/// else that goes wrong (another spawn error, a non-zero exit, a budget outrun,
/// words that are not a version) is [`ToolAnswer::Unreadable`]. Nothing of the
/// failure is carried: no error string, no path, no stderr (S5).
fn read(
    program: &str,
    budget: Duration,
    args: &[&str],
    version: fn(&str) -> Option<String>,
) -> ToolAnswer {
    let mut command = std::process::Command::new(program);
    command
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    // The same guard every read Settings starts on the user's behalf uses: a
    // probe never flashes a console window over the app.
    super::without_console(&mut command);
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) if error.kind() == ErrorKind::NotFound => return ToolAnswer::Missing,
        Err(_) => return ToolAnswer::Unreadable,
    };
    let deadline = Instant::now() + budget;
    let finished = loop {
        match child.try_wait() {
            Ok(Some(_)) => break true,
            Ok(None) if Instant::now() >= deadline => break false,
            Ok(None) => std::thread::sleep(POLL),
            // A child whose state cannot be read is a run that will not answer.
            Err(_) => break false,
        }
    };
    if !finished {
        let _ = child.kill();
        let _ = child.wait();
        return ToolAnswer::Unreadable;
    }
    // The child has exited, so reading its pipes cannot block: this only drains
    // what it already wrote.
    let Ok(output) = child.wait_with_output() else {
        return ToolAnswer::Unreadable;
    };
    if !output.status.success() {
        return ToolAnswer::Unreadable;
    }
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    match version(&text) {
        Some(version) => ToolAnswer::Installed(version),
        None => ToolAnswer::Unreadable,
    }
}

/// node's rule: `node --version` prints the version and nothing else, so the
/// first word of the first line that has one IS the version - kept exactly as
/// node printed it, `v` and all (`v24.11.1`), because that is the answer the
/// page draws.
///
/// What counts as a version: an optional leading `v`, then ASCII digits and dots
/// only, starting with a digit and carrying at least one dot. Anything else - a
/// shell's "command not found", a flag's usage text, an empty answer - is no
/// version, and the row says it could not be read (S5).
fn node_version(text: &str) -> Option<String> {
    let token = text
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())?
        .split_whitespace()
        .next()?;
    let digits = token.strip_prefix('v').unwrap_or(token);
    let shaped = digits.starts_with(|c: char| c.is_ascii_digit())
        && digits.contains('.')
        && digits.chars().all(|c| c.is_ascii_digit() || c == '.');
    shaped.then(|| token.to_owned())
}

/// git's rule: `git --version` prints `git version 2.51.0.windows.1`, so the
/// version is what follows git's own name, cut at the first character that is
/// not a digit or a dot: the build tag is the platform's, not the version's, and
/// the row draws `2.51.0`.
///
/// What counts as a version: at least one dot among the digits, so `git version`
/// alone, a usage line, another program's words or an empty answer are no
/// version at all.
fn git_version(text: &str) -> Option<String> {
    let rest = text.trim_start().strip_prefix("git version ")?;
    let token = rest.split_whitespace().next()?;
    let leading: String = token
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let version = leading.trim_end_matches('.');
    (version.contains('.') && version.starts_with(|c: char| c.is_ascii_digit()))
        .then(|| version.to_owned())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::contract::{UpdatesView, VersionRowView};

    /// The environment check's rows as (tool, state, version), the shape every
    /// assertion here compares.
    fn rows(view: &GeneralView) -> Vec<(ToolKind, ToolState, Option<String>)> {
        view.tools
            .iter()
            .map(|row| (row.tool, row.state, row.version.clone()))
            .collect()
    }

    /// A versions read's answer with the two rows the Updates pane draws from it.
    fn updates(shell: Option<&str>, pi: Option<&str>) -> UpdatesView {
        UpdatesView {
            shell: VersionRowView {
                current: shell.map(str::to_owned),
                ..VersionRowView::default()
            },
            pi: VersionRowView {
                current: pi.map(str::to_owned),
                ..VersionRowView::default()
            },
            ..UpdatesView::default()
        }
    }

    /// One check's answer over the seams a test names.
    fn check(
        updates: &UpdatesView,
        launcher_found: bool,
        node: ToolAnswer,
        git: ToolAnswer,
    ) -> GeneralView {
        let tools: ToolProbe = Arc::new(move |tool: ToolKind, _language: Language| match tool {
            ToolKind::Node => node.clone(),
            ToolKind::Git => git.clone(),
            other => panic!("the environment check never spawns {other:?}"),
        });
        view(
            updates,
            launcher_found,
            &tools,
            Language::En,
            1_700_000_000_000,
        )
    }

    /// node's rule: the first word it prints, `v` and all, because that is what
    /// node answered and what the page draws (`v24.11.1`).
    #[test]
    fn node_prints_the_version_with_its_own_prefix() {
        for (text, expected) in [
            ("v24.11.1\n", Some("v24.11.1")),
            ("24.11.1", Some("24.11.1")),
            ("  v24.11.1  \n", Some("v24.11.1")),
            ("\n\nv24.11.1\nsecond line\n", Some("v24.11.1")),
        ] {
            assert_eq!(node_version(text).as_deref(), expected, "{text:?}");
        }
    }

    /// And anything that is not a version is no version at all: the row says it
    /// could not be read instead of drawing words as a version.
    #[test]
    fn node_output_that_is_no_version_reads_as_nothing() {
        for text in [
            "",
            "\n\n",
            "v",
            "24",
            "vv24.11.1",
            "node: command not found",
            "'node' is not recognized as an internal or external command",
            "v24.11.1-rc.1 (experimental)",
        ] {
            assert_eq!(
                node_version(text),
                None,
                "a version is digits and dots, not {text:?}"
            );
        }
        // A pre-release tag is words after the number, and a version is the
        // number: `v24.11.1-rc.1` carries none of these.
        assert_eq!(node_version("v24.11.1\n"), Some("v24.11.1".to_owned()));
    }

    /// git's rule: its words, then the version, and only the leading dotted
    /// number: the Windows build prints `2.51.0.windows.1` and the row draws
    /// `2.51.0`.
    #[test]
    fn git_prints_its_name_then_the_version() {
        for (text, expected) in [
            ("git version 2.51.0.windows.1\n", Some("2.51.0")),
            ("git version 2.51.0\n", Some("2.51.0")),
            ("git version 2.51\n", Some("2.51")),
            ("  git version 2.51.0.windows.1  ", Some("2.51.0")),
        ] {
            assert_eq!(git_version(text).as_deref(), expected, "{text:?}");
        }
    }

    #[test]
    fn git_output_that_is_no_version_reads_as_nothing() {
        for text in [
            "",
            "\n",
            "git version",
            "git version ",
            "git version unknown",
            "git version .",
            "Git version 2.51.0",
            "git: command not found",
            "warning: git found at C:\\Program Files\\Git\\bin\\git.exe",
        ] {
            assert_eq!(
                git_version(text),
                None,
                "a version is digits and dots after git's own name, not {text:?}"
            );
        }
    }

    /// S2: an answered probe is the row's version, and it is the token the tool
    /// printed, never a re-shaped one.
    #[test]
    fn a_probe_that_answered_draws_installed_with_its_version() {
        let view = check(
            &updates(Some("4.9.0"), Some("1.6.0")),
            true,
            ToolAnswer::Installed("v24.11.1".to_owned()),
            ToolAnswer::Installed("2.51.0".to_owned()),
        );

        assert_eq!(
            rows(&view),
            vec![
                (
                    ToolKind::GentleShell,
                    ToolState::Installed,
                    Some("4.9.0".to_owned())
                ),
                (ToolKind::Pi, ToolState::Installed, Some("1.6.0".to_owned())),
                (
                    ToolKind::Node,
                    ToolState::Installed,
                    Some("v24.11.1".to_owned())
                ),
                (
                    ToolKind::Git,
                    ToolState::Installed,
                    Some("2.51.0".to_owned())
                ),
            ],
            "one row per tool, in the tools' own order"
        );
    }

    /// S5: the two failures are different states, and neither carries a version.
    #[test]
    fn a_tool_that_is_not_there_is_missing_and_one_that_did_not_answer_is_unreadable() {
        let missing = check(
            &updates(Some("4.9.0"), Some("1.6.0")),
            true,
            ToolAnswer::Missing,
            ToolAnswer::Missing,
        );
        assert_eq!(missing.tools[2].state, ToolState::Missing);
        assert_eq!(missing.tools[2].version, None);
        assert_eq!(missing.tools[3].state, ToolState::Missing);

        // A probe that ran out of its budget cannot be told apart from one that
        // answered nonsense: both are the same "could not be read" state.
        let unreadable = check(
            &updates(Some("4.9.0"), Some("1.6.0")),
            true,
            ToolAnswer::Unreadable,
            ToolAnswer::Unreadable,
        );
        assert_eq!(unreadable.tools[2].state, ToolState::Unreadable);
        assert_eq!(unreadable.tools[2].version, None);
        assert_eq!(unreadable.tools[3].state, ToolState::Unreadable);
    }

    /// S3: the shell's and pi's rows are the versions the Updates read already
    /// answered, and each row follows its own version - one tool missing its
    /// version does not blank the other.
    #[test]
    fn the_shell_and_pi_rows_follow_the_versions_read() {
        let view = check(
            &updates(Some("4.9.0"), None),
            true,
            ToolAnswer::Installed("v24.11.1".to_owned()),
            ToolAnswer::Installed("2.51.0".to_owned()),
        );

        assert_eq!(view.tools[0].state, ToolState::Installed);
        assert_eq!(view.tools[0].version.as_deref(), Some("4.9.0"));
        assert_eq!(
            view.tools[1].state,
            ToolState::Unreadable,
            "the run printed the shell's version and nothing for pi"
        );
        assert_eq!(view.tools[1].version, None);
    }

    /// The one failure that means "this machine does not have the shell": the
    /// launcher the bridge already resolved is not there, which is the same
    /// answer the version probe cannot get past.
    #[test]
    fn a_machine_without_the_launcher_is_missing_and_never_unreadable() {
        let view = check(
            &updates(None, None),
            false,
            ToolAnswer::Installed("v24.11.1".to_owned()),
            ToolAnswer::Installed("2.51.0".to_owned()),
        );

        assert_eq!(view.tools[0].state, ToolState::Missing);
        assert_eq!(view.tools[0].version, None);
        assert_eq!(view.tools[1].state, ToolState::Missing);
        assert_eq!(view.tools[1].version, None);
        assert_eq!(
            view.tools[2].state,
            ToolState::Installed,
            "node has nothing to do with the shell's launcher"
        );
    }

    /// S4: the row that says when the check last ran is the load's own instant.
    #[test]
    fn the_check_carries_the_instant_it_read() {
        let view = check(
            &updates(Some("4.9.0"), Some("1.6.0")),
            true,
            ToolAnswer::Missing,
            ToolAnswer::Missing,
        );

        assert_eq!(view.checked_ms, Some(1_700_000_000_000.0));
    }

    /// S5: the wire shape carries the version token and nothing of the machine.
    #[test]
    fn a_row_never_carries_a_path_or_a_tools_stderr() {
        let view = check(
            &updates(Some("4.9.0"), Some("1.6.0")),
            true,
            ToolAnswer::Installed("v24.11.1".to_owned()),
            ToolAnswer::Installed("2.51.0".to_owned()),
        );

        let carried = serde_json::to_string(&view).expect("the rows are serializable");
        assert_eq!(
            carried,
            concat!(
                r#"{"tools":["#,
                r#"{"tool":"GentleShell","state":"Installed","version":"4.9.0"},"#,
                r#"{"tool":"Pi","state":"Installed","version":"1.6.0"},"#,
                r#"{"tool":"Node","state":"Installed","version":"v24.11.1"},"#,
                r#"{"tool":"Git","state":"Installed","version":"2.51.0"}"#,
                r#"],"checked_ms":1700000000000.0}"#,
            ),
            "the four rows, state and version only"
        );
    }

    /// The one bounded run every real probe is: a program that is not on this
    /// machine answers Missing, and not an error string that would have to be
    /// drawn (S5).
    #[test]
    fn a_program_that_is_not_there_is_missing() {
        assert_eq!(
            read(
                "gentle-desk-no-such-tool-anywhere",
                TOOL_TIMEOUT,
                &["--version"],
                node_version
            ),
            ToolAnswer::Missing
        );
    }

    /// The budget is a budget: a run that does not answer inside it is killed
    /// and reads as unreadable, and the check never waits on the machine (S3,
    /// S4). The test binary is a program this machine certainly has, so the run
    /// starts and is cut short rather than being missing.
    #[test]
    fn a_run_that_outlives_its_budget_is_unreadable_and_does_not_hang() {
        let program = std::env::current_exe()
            .expect("the test binary is a path")
            .display()
            .to_string();
        let started = std::time::Instant::now();

        let answer = read(
            &program,
            std::time::Duration::from_millis(0),
            &["--version"],
            node_version,
        );

        assert_eq!(
            answer,
            ToolAnswer::Unreadable,
            "no version in its budget, whatever it did print"
        );
        assert!(
            started.elapsed() < std::time::Duration::from_secs(20),
            "the probe ended inside its own deadline"
        );
    }

    /// A run that finished is read through the tool's own rule, and only its
    /// standard output is read: what the rule answers is the row's version.
    /// The test binary is the one program every machine running these tests has,
    /// and `--list` is its own way of printing something and exiting well.
    #[test]
    fn a_run_that_finished_is_read_through_the_tools_own_rule() {
        let program = std::env::current_exe()
            .expect("the test binary is a path")
            .display()
            .to_string();

        let answer = read(&program, TOOL_TIMEOUT, &["--list"], |text| {
            text.contains(": test").then(|| "1.2.3".to_owned())
        });

        assert_eq!(answer, ToolAnswer::Installed("1.2.3".to_owned()));
    }

    /// And a run that exits badly is unreadable even when it printed something:
    /// a tool that refused to answer is not a row with a version.
    #[test]
    fn a_run_that_exits_badly_is_unreadable_even_when_it_printed_something() {
        let program = std::env::current_exe()
            .expect("the test binary is a path")
            .display()
            .to_string();

        let answer = read(
            &program,
            TOOL_TIMEOUT,
            &["--this-flag-does-not-exist"],
            |_text| Some("9.9.9".to_owned()),
        );

        assert_eq!(answer, ToolAnswer::Unreadable);
    }
}
