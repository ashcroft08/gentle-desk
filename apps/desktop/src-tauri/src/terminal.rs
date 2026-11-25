//! "Open in terminal" (0.1.12, S1, S2, T2, T3): the buttons that hand a job to a
//! VISIBLE terminal window, because the shell owns it and the desk only reads.
//!
//! Four jobs travel this way:
//!
//! - profiles: the shell's INTERACTIVE screen, opened with no opening prompt.
//!   The maintainer's first run showed pi's extension erroring on a
//!   prompt-started profiles screen (`saveFirst` undefined — the screen needs
//!   the interactive context pi builds after startup), so the desk opens the
//!   shell and the user names `/gentle:profiles` in the window; the panel's
//!   note already says so. There `n` creates a profile and `u` uses one in the
//!   session (L9), and the desk never writes the file.
//! - login / logout: the shell's own `auth login` / `auth logout`.
//! - update: the shell's own `update`, which forwards to pi's updater (L5), run
//!   from a terminal because S4 lets an update run only with every chat closed.
//! - update extensions: the same command with pi's `--extensions` flag passed
//!   through, so only the installed packages are touched; idempotent, which is
//!   why it needs no check.
//!
//! Every one of them is started with the SAME home flags the chats' children get
//! (the desk's own other arguments are not passed on) and through the resolved
//! launcher, so the terminal reads the home the desk is showing and no `pi` has
//! to be on `PATH`.
//!
//! Which command an action runs is the HOME's (updates-by-home S1, [`command_for_home`]):
//! "Separado" is the shell's own home, so every action is `gentle-shell ...`
//! through the launcher, byte for byte what it always was; "Mi pi" is the
//! user's own pi, so the three Updates actions run `pi ...` DIRECTLY - no
//! launcher, no home flag that belongs to the shell - and "Update Gentle Shell"
//! installs the npm `gentle-pi` version the pane measured against the registry
//! (a stable `x.y.z`, or nothing runs at all).
//!
//! On Windows the window is opened by `cmd` with `CREATE_NEW_CONSOLE`: the new
//! console IS the visible terminal, and the launcher runs inside it. Nothing is
//! written to the process environment (`set_var`), so the desk's own environment
//! stays exactly what it was.
//!
//! The window has to STAY OPEN (L22): the maintainer pressed Update extensions,
//! the shell answered in a fraction of a second and Windows Terminal took the
//! tab away before anything could be read. So the program is never handed to
//! `wt.exe` directly: a one-action wrapper cmd ([`wrapper_script`], in
//! [`wrapper_path`]) runs it, echoes a blank line and waits on `pause`, and the
//! tab wears the action's own name ([`action_label`]) so a window left open is
//! still identifiable. The `CREATE_NEW_CONSOLE` fallback spawns the program
//! itself and never the wrapper: a cmd file is not the program.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use gentle_backend::launcher::{self, Launcher, LauncherEnv};
use gentle_core::i18n::{messages, Language};
use gentle_runtime::updates;

use crate::contract::{HomeChoiceView, TerminalActionView};

/// What one action asks the shell to do: the arguments after the launcher's own
/// and the home flags. Profiles opens the INTERACTIVE shell with no opening
/// prompt: the maintainer's first run showed pi's extension erroring on a
/// prompt-started profiles screen (`saveFirst` undefined — the screen needs the
/// interactive context pi builds after startup), so the desk opens the shell
/// and the user names the command; the panel's note already says so.
pub fn action_args(action: TerminalActionView) -> &'static [&'static str] {
    match action {
        TerminalActionView::Profiles => &[],
        TerminalActionView::Login => &[],
        TerminalActionView::Logout => &[],
        TerminalActionView::Update => &["update"],
        // The shell's own stable upgrade (0.8.1 settings-polish S7): the command
        // that moves a stable install to npm's `latest`, which is the version the
        // pane's shell row is measured against. A separate action and not a flag
        // on `update`, because it is a different command.
        TerminalActionView::UpgradeShell => &["upgrade"],
        // `gentle-shell update` FORWARDS to pi's updater, so pi's own
        // `--extensions` flag passes straight through (L5) and the run touches
        // only the packages the shell has installed, never pi itself.
        TerminalActionView::UpdateExtensions => &["update", "--extensions"],
        // pi's own installer, named explicitly: the maintainer's rule is that an
        // install is never silent, so the pane hands the exact command to a
        // terminal window the user reads.
        TerminalActionView::InstallUsageSources => {
            &["install", "git:github.com/ashcroft08/pi-usage-sources"]
        }
    }
}

/// The home flags `args` carries: `--home <dir>`, `--link` and `--isolated`, in
/// the order they appear.
///
/// The desk's own argument list is what the children are started from
/// (`SettingsPaths::home_args`), so it can also carry an argument that belongs
/// to the app and not to the shell. A terminal gets the home flags only.
pub fn home_flags(args: &[OsString]) -> Vec<OsString> {
    let mut kept = Vec::new();
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.to_str() {
            Some("--home") => {
                kept.push(arg.clone());
                if let Some(dir) = rest.next() {
                    kept.push(dir.clone());
                }
            }
            Some("--link") | Some("--isolated") => kept.push(arg.clone()),
            _ => {}
        }
    }
    kept
}

/// The program and arguments one action runs: the resolved launcher with its own
/// leading arguments (an npm shim resolves to node plus the entry it points at),
/// then the home flags, then the action itself.
pub fn command_for(
    action: TerminalActionView,
    launcher: &Launcher,
    home_args: &[OsString],
) -> (PathBuf, Vec<OsString>) {
    let mut args = launcher.args.clone();
    args.extend(home_flags(home_args));
    args.extend(action_args(action).iter().map(OsString::from));
    (launcher.program.clone(), args)
}

/// Opens a visible terminal window on `action`, on the home the user chose and
/// with the arguments S1's table names ([`command_for_home`]).
///
/// The Separado home's commands keep the home flags the chats' children get; the
/// "Mi pi" home's are `pi` itself, with no launcher and no flag of the shell's.
/// A refusal is ONE line and no window (S2).
pub fn open(
    language: Language,
    action: TerminalActionView,
    home: HomeChoiceView,
    shell_latest: Option<&str>,
    home_args: &[OsString],
) -> Result<(), String> {
    let command =
        command_for_home(action, home, shell_latest).map_err(|refused| match refused {
            Refused::NoStableLatest => messages::terminal_no_stable_latest(language).to_owned(),
        })?;
    let (program, args) = match command.program {
        // "Mi pi": `pi` as this machine's `PATH` answers it, with the table's
        // arguments and nothing else. No launcher is resolved, because none is
        // used, and a `pi` that is not on the `PATH` is cmd's own error, kept
        // visible by the wrapper's `pause` (S5): nothing here looks for another
        // program.
        PI => (
            PathBuf::from(PI),
            command.args.iter().map(OsString::from).collect(),
        ),
        // "Separado", and the shell's own screens: the resolved launcher's own
        // program and leading arguments stand in for the name the table gives
        // (an npm shim resolves to node plus the entry it points at), then the
        // home flags, then the action. Today's bytes, unchanged (S3).
        _ => {
            let launcher = launcher::resolve(&LauncherEnv::from_process()).map_err(|error| {
                messages::terminal_shell_not_found(language, &error.to_string())
            })?;
            command_for(action, &launcher, home_args)
        }
    };
    launch(language, action, &program, &args)
}

/// The `gentle-shell` launcher: the program the "Separado" home's commands are
/// run through, and the one name the "Mi pi" home never uses.
pub const LAUNCHER: &str = "gentle-shell";

/// The user's own pi, run as it is in the "Mi pi" home (S1).
///
/// Deliberately a bare name: the `PATH` is the machine's, and S5 asks the app to
/// let cmd's own error speak when there is nothing to run.
pub const PI: &str = "pi";

/// One logical command of S1's table: the program by NAME, and the arguments
/// that follow it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub program: &'static str,
    pub args: Vec<String>,
}

/// Why an action runs nothing at all (S2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refused {
    /// The pane's Gentle Shell row holds no stable `x.y.z` to install: no
    /// version was measured, or what was measured is not one npm's stable
    /// release can be (a channel build such as `4.9.0-main.abc123`, a `v5.0.0`,
    /// a prerelease, nothing).
    NoStableLatest,
}

/// S1's table: the command `action` runs on `home`, before this machine's
/// launcher and home flags are resolved.
///
/// "Separado" is the shell's own home, so every action is the shell's own
/// command, run through the launcher: the same arguments [`action_args`] names,
/// which is what makes those lines byte for byte what they were (S3).
///
/// "Mi pi" is the user's own pi, so the Updates actions name `pi` directly and
/// carry no home flag of the shell's. "Update Gentle Shell" is
/// `pi install npm:gentle-pi@<latest>`, and `<latest>` is the version the pane's
/// own row measured against npm's stable `latest` (settings-polish S6) - taken
/// only while it is a plain stable `x.y.z` by the very rule that row applies
/// ([`stable_latest`]). Anything else is [`Refused::NoStableLatest`]: no
/// `@latest`, and never `gentle-shell upgrade` as a fallback.
///
/// The shell's own screens (profiles, sign in, sign out, install usage sources)
/// are not in S1's table: they keep the launcher on both homes.
pub fn command_for_home(
    action: TerminalActionView,
    home: HomeChoiceView,
    shell_latest: Option<&str>,
) -> Result<Invocation, Refused> {
    // The shell's own command for `action`: the arguments today's `action_args`
    // names, run through the launcher. Both the Separado arm and the shell's own
    // screens answer this, so those lines never drift from what they were.
    let shell = || Invocation {
        program: LAUNCHER,
        args: action_args(action)
            .iter()
            .map(|arg| (*arg).to_owned())
            .collect(),
    };

    if home == HomeChoiceView::Isolated {
        return Ok(shell());
    }

    match action {
        // "Mi pi": the user's own pi, with pi's own flags and no home flag of the
        // shell's, because `--link` and `--isolated` are not pi's arguments.
        TerminalActionView::Update => Ok(pi(&["update"])),
        TerminalActionView::UpdateExtensions => Ok(pi(&["update", "--extensions"])),
        // The shell package itself, as npm's stable release names it. The
        // version is the one the pane MEASURED, so a row that has none runs
        // nothing: `@latest` would install whatever npm says when the window
        // opens, and `gentle-shell upgrade` is the other home's command.
        TerminalActionView::UpgradeShell => {
            let Some(latest) = stable_latest(shell_latest) else {
                return Err(Refused::NoStableLatest);
            };
            Ok(pi(&["install", &format!("npm:gentle-pi@{latest}")]))
        }
        // The shell's own screens are not in S1's table, so they keep the
        // launcher on both homes.
        TerminalActionView::Profiles
        | TerminalActionView::Login
        | TerminalActionView::Logout
        | TerminalActionView::InstallUsageSources => Ok(shell()),
    }
}

/// One `pi` command of the table: the user's own pi, then exactly `args`.
fn pi(args: &[&str]) -> Invocation {
    Invocation {
        program: PI,
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
    }
}

/// The version `latest` names, when it is a plain stable `x.y.z` by the rule the
/// Updates pane's own row applies (`gentle_runtime::updates::parse`, the one
/// `stable_row` calls), trimmed. Anything else - a channel build, a `v` prefix,
/// a prerelease, an empty string, nothing measured - is no version to install.
fn stable_latest(latest: Option<&str>) -> Option<String> {
    let latest = latest?.trim();
    updates::parse(latest).map(|_| latest.to_owned())
}

/// The plain label one action's window and tab wear, in the pane's own words and
/// in `language`: the user pressed a button named this, so the window says the
/// same thing. No file, no product's command line.
pub fn action_label(language: Language, action: TerminalActionView) -> &'static str {
    match action {
        TerminalActionView::Profiles => messages::terminal_action_profiles(language),
        // The buttons are pi's own `/login` and `/logout` inside the shell, so
        // the labels are the words those buttons would wear.
        TerminalActionView::Login => messages::terminal_action_login(language),
        TerminalActionView::Logout => messages::terminal_action_logout(language),
        TerminalActionView::Update => messages::terminal_action_update(language),
        TerminalActionView::UpgradeShell => messages::terminal_action_upgrade_shell(language),
        TerminalActionView::UpdateExtensions => {
            messages::terminal_action_update_extensions(language)
        }
        TerminalActionView::InstallUsageSources => {
            messages::terminal_action_install_usage_sources(language)
        }
    }
}

/// The wrapper's file name for `action`: one file per action, so two windows
/// opened in a row never race on one script.
fn action_slug(action: TerminalActionView) -> &'static str {
    match action {
        TerminalActionView::Profiles => "profiles",
        TerminalActionView::Login => "login",
        TerminalActionView::Logout => "logout",
        TerminalActionView::Update => "update",
        TerminalActionView::UpgradeShell => "upgrade-shell",
        TerminalActionView::UpdateExtensions => "update-extensions",
        TerminalActionView::InstallUsageSources => "install-usage-sources",
    }
}

/// Where `action`'s wrapper cmd is written: the app's own temp directory, never
/// beside the app or the user's files. Its content is [`wrapper_script`], and
/// the file is rewritten on every run.
pub fn wrapper_path(action: TerminalActionView) -> PathBuf {
    std::env::temp_dir().join(format!("gentle-desk-{}.cmd", action_slug(action)))
}

/// The wrapper cmd's content: honest quoting (every part in double quotes,
/// separators turned into backslashes), then the blank line and `pause` that
/// keep the window open after a command that exits in a fraction of a second.
///
/// Pure, and it answers the content rather than writing it, so the shape is
/// pinned by tests. A part carrying a double quote is REFUSED: cmd would end
/// the quoted part there and run whatever follows (the rule the earlier
/// `console_line` had). Every line is CRLF, cmd's own ending, and the content
/// is ASCII.
pub fn wrapper_script(
    language: Language,
    program: &Path,
    args: &[OsString],
) -> Result<String, String> {
    let mut line = quote_part(language, &program.to_string_lossy())?;
    for arg in args {
        line.push(' ');
        line.push_str(&quote_part(language, &arg.to_string_lossy())?);
    }

    Ok(format!("@echo off\r\n{line}\r\necho.\r\npause\r\n"))
}

/// One part of the wrapper's command line, in double quotes, with the
/// separators cmd wants.
fn quote_part(language: Language, part: &str) -> Result<String, String> {
    if part.contains('"') {
        return Err(messages::terminal_part_has_quote(language, part));
    }
    Ok(format!("\"{}\"", part.replace('/', "\\")))
}

/// The arguments Windows Terminal gets: one new window, one tab wearing the
/// action's label, then the wrapper cmd after `--`. Pure, so the shape is
/// pinned by a test.
pub fn windows_terminal_args(
    language: Language,
    action: TerminalActionView,
    wrapper: &Path,
) -> Vec<OsString> {
    let mut argv: Vec<OsString> = [
        "-w",
        "-1",
        "new-tab",
        "--title",
        action_label(language, action),
        "--",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    argv.push(wrapper.as_os_str().to_owned());
    argv
}

/// `wt.exe` on this machine: the Windows Terminal this user has, when it is
/// installed. `LOCALAPPDATA`'s alias is the documented place; `PATH` is looked
/// at too, because a machine may have it either way.
fn windows_terminal() -> Option<PathBuf> {
    if let Some(base) = std::env::var_os("LOCALAPPDATA") {
        let candidate = PathBuf::from(base).join("Microsoft/WindowsApps/wt.exe");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join("wt.exe"))
        .find(|candidate| candidate.is_file())
}

/// Opens `program` in a terminal window, answering as soon as the window
/// exists: nothing here waits for the window to close.
///
/// Windows Terminal is used when it is installed: it owns a real terminal and
/// starts the program itself, so no `cmd` of this process parses the line and
/// no console handles are inherited from this GUI process. Measured in this
/// app: a child started with `CREATE_NEW_CONSOLE` from the console-less app
/// exits at once with code 0 and prints nothing, while the very same command
/// stays alive in a terminal of its own. When `wt.exe` is missing, the direct
/// spawn with its own console is the fallback.
///
/// Windows Terminal closes a tab the moment its command exits, which is why the
/// program is handed over INSIDE a wrapper cmd: `pause` holds the window open
/// and the output stays readable (L22). The wrapper is a cmd file, so the
/// fallback never uses it - it spawns `program` itself.
#[cfg(windows)]
fn launch(
    language: Language,
    action: TerminalActionView,
    program: &Path,
    args: &[OsString],
) -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    /// `CREATE_NEW_CONSOLE`: the fallback console that is visible on its own.
    const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
    /// `DETACHED_PROCESS`: Windows Terminal must not borrow this process's
    /// console, because a GUI process has none to give.
    const DETACHED_PROCESS: u32 = 0x0000_0008;

    if let Some(wt) = windows_terminal() {
        let script = wrapper_script(language, program, args)?;
        let wrapper = wrapper_path(action);
        std::fs::write(&wrapper, script).map_err(|error| {
            messages::terminal_wrapper_unwritable(
                language,
                &wrapper.display().to_string(),
                &error.to_string(),
            )
        })?;
        let started = std::process::Command::new(wt)
            .args(windows_terminal_args(language, action, &wrapper))
            .creation_flags(DETACHED_PROCESS)
            .spawn();
        if started.is_ok() {
            return Ok(());
        }
    }

    std::process::Command::new(program)
        .args(args)
        .creation_flags(CREATE_NEW_CONSOLE)
        .spawn()
        .map(|_| ())
        .map_err(|error| messages::terminal_open_failed(language, &error.to_string()))
}

/// Everywhere else there is no console flag to set: the launcher is started as
/// it is and inherits the terminal the app was started from. The desktop app
/// ships on Windows, where the console above is what makes the window visible,
/// so no wrapper is written here.
#[cfg(not(windows))]
fn launch(
    language: Language,
    _action: TerminalActionView,
    program: &Path,
    args: &[OsString],
) -> Result<(), String> {
    std::process::Command::new(program)
        .args(args)
        .spawn()
        .map(|_| ())
        .map_err(|error| messages::terminal_open_failed(language, &error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A launcher as the npm shim resolves on Windows: node plus the entry.
    fn launcher() -> Launcher {
        Launcher {
            program: PathBuf::from("C:/node/node.exe"),
            args: vec![OsString::from("C:/npm/gentle-shell.mjs")],
        }
    }

    fn os(args: &[&str]) -> Vec<OsString> {
        args.iter().map(OsString::from).collect()
    }

    /// Each button names the shell's own command. Profiles opens the shell
    /// INTERACTIVE with no opening prompt: pi's extension errors on a
    /// prompt-started profiles screen (the maintainer's run:
    /// `saveFirst` undefined), so the user names the command in the window.
    #[test]
    fn every_action_names_the_shells_own_command() {
        assert_eq!(
            action_args(TerminalActionView::Profiles) as &[&str],
            &[] as &[&str]
        );
        // `pi auth login|logout` REQUIRE a provider (`Usage: pi auth login
        // <provider>`, exit 1 in 0.6 s - the window blinked away). Sign-in and
        // sign-out are pi's own `/login` and `/logout` inside the shell, which
        // ask for the provider themselves, so these open the shell too.
        assert_eq!(
            action_args(TerminalActionView::Login) as &[&str],
            &[] as &[&str]
        );
        assert_eq!(
            action_args(TerminalActionView::Logout) as &[&str],
            &[] as &[&str]
        );
        assert_eq!(action_args(TerminalActionView::Update), &["update"]);
        // S7: the shell's own stable upgrade is a SECOND action, not a flag on
        // the update: it runs `gentle-shell upgrade` and nothing else.
        assert_eq!(action_args(TerminalActionView::UpgradeShell), &["upgrade"]);
        // The maintainer's rule: an install is explicit and never silent, so the
        // button hands pi the exact command the pane's note names.
        assert_eq!(
            action_args(TerminalActionView::InstallUsageSources),
            &["install", "git:github.com/ashcroft08/pi-usage-sources"]
        );
    }

    /// Extensions are updated by the SAME shell command with pi's own flag
    /// appended: `gentle-shell update` forwards to pi's updater, so
    /// `--extensions` passes straight through and only the packages the shell
    /// has installed are touched (`--self` stays pi's default).
    #[test]
    fn update_extensions_is_the_shells_update_with_pis_own_flag() {
        assert_eq!(
            action_args(TerminalActionView::UpdateExtensions) as &[&str],
            &["update", "--extensions"] as &[&str]
        );
    }

    /// The command line is the launcher's own arguments, then the home flags,
    /// then the action: the same shape every other shell run of the pane has.
    #[test]
    fn the_command_is_the_launcher_then_the_home_flags_then_the_action() {
        let (program, args) = command_for(
            TerminalActionView::Update,
            &launcher(),
            &os(&["--isolated"]),
        );

        assert_eq!(program, PathBuf::from("C:/node/node.exe"));
        assert_eq!(
            args,
            os(&["C:/npm/gentle-shell.mjs", "--isolated", "update"])
        );
    }

    /// A home the user named explicitly reaches the terminal untouched, so the
    /// sign-in the terminal writes is the one the chats already read.
    #[test]
    fn a_home_flag_chooses_the_same_home_the_children_read() {
        let (_, args) = command_for(
            TerminalActionView::Profiles,
            &launcher(),
            &os(&["--home", "C:/h", "--link"]),
        );

        assert_eq!(
            args,
            os(&["C:/npm/gentle-shell.mjs", "--home", "C:/h", "--link"]),
            "the flags travel as the children get them: `--home` is the one the shell reads first"
        );
    }

    /// The wrapper cmd runs: the program and every argument, each in double
    /// quotes (a path with a space is exactly what broke the earlier attempts)
    /// and with forward slashes turned into backslashes, which cmd wants.
    #[test]
    fn the_wrapper_quotes_every_part_and_turns_slashes_into_backslashes() {
        let script = wrapper_script(
            Language::En,
            Path::new("C:/Program Files/nodejs/node.exe"),
            &os(&[
                "C:/npm/gentle-shell.mjs",
                "--home",
                "C:/Users/josep/.pi/agent",
            ]),
        )
        .expect("a quoted line is written");

        assert_eq!(
            script,
            "@echo off\r\n\
             \"C:\\Program Files\\nodejs\\node.exe\" \"C:\\npm\\gentle-shell.mjs\" \
             \"--home\" \"C:\\Users\\josep\\.pi\\agent\"\r\n\
             echo.\r\n\
             pause\r\n"
        );
    }

    /// The window has to stay open after a command that exits in a fraction of
    /// a second, so the wrapper ends with the blank line and `pause`. Every
    /// line ends in CRLF, cmd's own ending, and nothing outside ASCII.
    #[test]
    fn the_wrapper_waits_for_a_key_before_it_closes() {
        let script = wrapper_script(
            Language::En,
            Path::new("C:/node/node.exe"),
            &os(&["update"]),
        )
        .unwrap();

        assert_eq!(
            script.split("\r\n").collect::<Vec<_>>(),
            vec![
                "@echo off",
                "\"C:\\node\\node.exe\" \"update\"",
                "echo.",
                "pause",
                ""
            ],
            "`pause` is the last thing cmd reads, and the only line ending is CRLF"
        );
        assert!(script.is_ascii(), "a cmd file is ASCII: {script:?}");
    }

    /// A double quote in a part cannot be passed on: cmd would end the quoted
    /// part there and run whatever follows. The same rule the earlier
    /// `console_line` had, refused rather than mangled.
    #[test]
    fn a_part_with_a_double_quote_is_refused() {
        let error = wrapper_script(
            Language::En,
            Path::new("C:/node/node.exe"),
            &os(&["C:/a\"b.mjs"]),
        )
        .expect_err("an argument with a double quote is refused");
        assert!(
            error.contains("double quote"),
            "the reason names what cannot be passed on: {error}"
        );
        assert!(
            wrapper_script(Language::En, Path::new("C:/no\"de/node.exe"), &[]).is_err(),
            "the program itself is checked too"
        );
    }

    /// Every action gets its own cmd file in the app's temp directory, so two
    /// windows never race on one file and nothing is written beside the app.
    #[test]
    fn each_action_writes_its_own_cmd_in_the_temp_directory() {
        let temp = std::env::temp_dir();
        let expected = [
            (TerminalActionView::Profiles, "gentle-desk-profiles.cmd"),
            (TerminalActionView::Login, "gentle-desk-login.cmd"),
            (TerminalActionView::Logout, "gentle-desk-logout.cmd"),
            (TerminalActionView::Update, "gentle-desk-update.cmd"),
            (
                TerminalActionView::UpgradeShell,
                "gentle-desk-upgrade-shell.cmd",
            ),
            (
                TerminalActionView::UpdateExtensions,
                "gentle-desk-update-extensions.cmd",
            ),
            (
                TerminalActionView::InstallUsageSources,
                "gentle-desk-install-usage-sources.cmd",
            ),
        ];

        let mut seen = std::collections::BTreeSet::new();
        for (action, name) in expected {
            let path = wrapper_path(action);
            assert_eq!(path.parent(), Some(temp.as_path()), "{action:?}");
            assert_eq!(
                path.file_name().and_then(|name| name.to_str()),
                Some(name),
                "one file per action, named after it"
            );
            assert!(
                seen.insert(path),
                "{action:?} shares a file with another action"
            );
        }
    }

    /// The tab says what ran, in the pane's own words: a plain label, one per
    /// action, so a window left open over lunch is still identifiable.
    #[test]
    fn every_action_has_a_plain_tab_label() {
        let actions = [
            TerminalActionView::Profiles,
            TerminalActionView::Login,
            TerminalActionView::Logout,
            TerminalActionView::Update,
            TerminalActionView::UpgradeShell,
            TerminalActionView::UpdateExtensions,
            TerminalActionView::InstallUsageSources,
        ];

        let labels: Vec<&str> = actions
            .iter()
            .copied()
            .map(|action| action_label(Language::En, action))
            .collect();
        assert_eq!(
            labels,
            vec![
                "Profiles",
                "Sign in",
                "Sign out",
                "Update",
                "Update Gentle Shell",
                "Update extensions",
                "Install usage sources"
            ]
        );
        for label in labels {
            assert!(label.is_ascii() && !label.is_empty(), "{label:?}");
        }
        // The escalation says the product's whole name in both languages, because
        // this is the first time the pane asks for that name on the machine.
        assert_eq!(
            action_label(Language::Es, TerminalActionView::UpgradeShell),
            "Actualizar Gentle Shell"
        );
    }

    /// Windows Terminal gets one new window, one tab with the action's title,
    /// then `--` and the WRAPPER cmd: nothing is parsed by this process, and
    /// cmd is what keeps the window open after the program exits.
    #[test]
    fn windows_terminal_opens_one_titled_tab_on_the_wrapper() {
        let argv = windows_terminal_args(
            Language::En,
            TerminalActionView::UpdateExtensions,
            Path::new("C:/tmp/gentle-desk-update-extensions.cmd"),
        );

        assert_eq!(
            argv,
            os(&[
                "-w",
                "-1",
                "new-tab",
                "--title",
                "Update extensions",
                "--",
                "C:/tmp/gentle-desk-update-extensions.cmd"
            ])
        );
    }

    /// The desk's own arguments are not the shell's: only the home flags travel.
    #[test]
    fn only_the_home_flags_of_the_desks_arguments_travel() {
        assert_eq!(
            home_flags(&os(&["--fullscreen", "--link", "--daemon", "--isolated"])),
            os(&["--link", "--isolated"])
        );
        assert_eq!(home_flags(&os(&["--home"])), os(&["--home"]));
        assert!(home_flags(&os(&["--fullscreen"])).is_empty());
    }

    /// A launcher whose program is `gentle-shell` itself, so the Separado lines
    /// of the table read exactly as the table does. What a real npm shim
    /// resolves to (node plus the entry) is the test above's.
    fn plain_launcher() -> Launcher {
        Launcher {
            program: PathBuf::from(LAUNCHER),
            args: Vec::new(),
        }
    }

    /// The line one action really runs on this machine, read out of the wrapper
    /// cmd: the resolved launcher and the home flags for the Separado home, `pi`
    /// and nothing else for "Mi pi".
    fn command_line(
        action: TerminalActionView,
        home: HomeChoiceView,
        latest: Option<&str>,
        launcher: &Launcher,
        home_args: &[OsString],
    ) -> String {
        let command = command_for_home(action, home, latest).expect("a runnable command");
        let (program, args) = if command.program == LAUNCHER {
            command_for(action, launcher, home_args)
        } else {
            (
                PathBuf::from(command.program),
                command.args.iter().map(OsString::from).collect(),
            )
        };
        let script = wrapper_script(Language::En, &program, &args).expect("a quotable line");
        script
            .split("\r\n")
            .nth(1)
            .expect("the wrapper names its command")
            .to_owned()
    }

    /// The table of updates-by-home S1, byte for byte and for both homes: the
    /// Separado home is the shell's own, so these are today's commands; "Mi pi"
    /// is the user's own pi, named directly, with no shell home flag, and
    /// "Update Gentle Shell" installs the stable version the pane measured.
    #[test]
    fn the_updates_commands_follow_the_home() {
        let latest = Some("5.0.0");
        let launcher = plain_launcher();
        let expected = [
            (
                TerminalActionView::UpdateExtensions,
                "\"gentle-shell\" \"update\" \"--extensions\"",
                "\"pi\" \"update\" \"--extensions\"",
            ),
            (
                TerminalActionView::Update,
                "\"gentle-shell\" \"update\"",
                "\"pi\" \"update\"",
            ),
            (
                TerminalActionView::UpgradeShell,
                "\"gentle-shell\" \"upgrade\"",
                "\"pi\" \"install\" \"npm:gentle-pi@5.0.0\"",
            ),
        ];

        for (action, separado, mi_pi) in expected {
            assert_eq!(
                command_line(action, HomeChoiceView::Isolated, latest, &launcher, &[]),
                separado,
                "{action:?} on Separado"
            );
            assert_eq!(
                command_line(action, HomeChoiceView::Link, latest, &launcher, &[]),
                mi_pi,
                "{action:?} on Mi pi"
            );
        }
    }

    /// S2: `pi install npm:gentle-pi@<latest>` only with the stable `x.y.z` the
    /// pane measured - the very rule the shell row applies. Anything else runs
    /// nothing, says the one typed refusal, and never falls back to
    /// `gentle-shell upgrade` or to `@latest`.
    #[test]
    fn a_latest_that_is_not_a_stable_version_refuses_the_upgrade() {
        for latest in [
            None,
            Some("4.9.0-main.abc123"),
            Some("v5.0.0"),
            Some(""),
            Some("   "),
        ] {
            assert_eq!(
                command_for_home(
                    TerminalActionView::UpgradeShell,
                    HomeChoiceView::Link,
                    latest
                ),
                Err(Refused::NoStableLatest),
                "{latest:?} is no version to install"
            );
        }
    }

    /// The measured version is used as the row measured it, trimmed: no `v`, no
    /// `@latest`, no tag, nothing derived from a string the web sends.
    #[test]
    fn an_accepted_latest_is_the_plain_stable_version() {
        assert_eq!(
            command_for_home(
                TerminalActionView::UpgradeShell,
                HomeChoiceView::Link,
                Some(" 5.0.0 ")
            ),
            Ok(Invocation {
                program: PI,
                args: vec!["install".to_owned(), "npm:gentle-pi@5.0.0".to_owned()],
            })
        );
    }

    /// S3: the Separado commands are today's, byte for byte - the resolved
    /// launcher's own program and leading arguments, then the home flags the
    /// children get, then the action - and no latest is needed for them.
    #[test]
    fn the_separado_home_keeps_todays_lines() {
        let launcher = launcher();
        let home_args = os(&["--home", "C:/h", "--link"]);
        let prefix =
            "\"C:\\node\\node.exe\" \"C:\\npm\\gentle-shell.mjs\" \"--home\" \"C:\\h\" \"--link\"";
        let expected = [
            (
                TerminalActionView::UpdateExtensions,
                format!("{prefix} \"update\" \"--extensions\""),
            ),
            (TerminalActionView::Update, format!("{prefix} \"update\"")),
            (
                TerminalActionView::UpgradeShell,
                format!("{prefix} \"upgrade\""),
            ),
        ];

        for (action, line) in expected {
            assert_eq!(
                command_line(
                    action,
                    HomeChoiceView::Isolated,
                    None,
                    &launcher,
                    &home_args
                ),
                line,
                "{action:?} runs today's line on the Separado home"
            );
            assert_eq!(
                command_for_home(action, HomeChoiceView::Isolated, None),
                Ok(Invocation {
                    program: LAUNCHER,
                    args: action_args(action)
                        .iter()
                        .map(|arg| (*arg).to_owned())
                        .collect(),
                }),
                "{action:?} is the shell's own command, unchanged"
            );
        }
    }

    /// S1: in "Mi pi" the launcher is not used at all - no `gentle-shell`, no
    /// home flag that belongs to the shell - and the tab still waits on `pause`,
    /// exactly as it does today (S3).
    #[test]
    fn the_mi_pi_wrapper_names_pi_alone() {
        let command = command_for_home(
            TerminalActionView::UpgradeShell,
            HomeChoiceView::Link,
            Some("5.0.0"),
        )
        .expect("a measured stable version is a command");
        let args: Vec<OsString> = command.args.iter().map(OsString::from).collect();
        let script = wrapper_script(Language::En, Path::new(command.program), &args)
            .expect("a quotable line");

        assert!(!script.contains("gentle-shell"), "{script:?}");
        for flag in ["--home", "--link", "--isolated"] {
            assert!(
                !script.contains(flag),
                "{flag} belongs to the shell, not to pi: {script:?}"
            );
        }
        assert_eq!(
            script,
            "@echo off\r\n\"pi\" \"install\" \"npm:gentle-pi@5.0.0\"\r\necho.\r\npause\r\n"
        );
    }

    /// S1's table names the three Updates actions: the shell's own screens keep
    /// the launcher on both homes (S3).
    #[test]
    fn the_shells_own_screens_keep_the_launcher_on_both_homes() {
        for action in [
            TerminalActionView::Profiles,
            TerminalActionView::Login,
            TerminalActionView::Logout,
            TerminalActionView::InstallUsageSources,
        ] {
            for home in [HomeChoiceView::Isolated, HomeChoiceView::Link] {
                assert_eq!(
                    command_for_home(action, home, None),
                    Ok(Invocation {
                        program: LAUNCHER,
                        args: action_args(action)
                            .iter()
                            .map(|arg| (*arg).to_owned())
                            .collect(),
                    }),
                    "{action:?} on {home:?}"
                );
            }
        }
    }

    /// The refusal is one honest line and NOTHING else: no window, no wrapper
    /// written, no `gentle-shell upgrade` run instead (S2).
    #[test]
    fn the_refusal_opens_nothing() {
        let wrapper = wrapper_path(TerminalActionView::UpgradeShell);
        let before = std::fs::read(&wrapper).ok();

        let error = open(
            Language::En,
            TerminalActionView::UpgradeShell,
            HomeChoiceView::Link,
            Some("4.9.0-main.abc123"),
            &[],
        )
        .expect_err("a channel build's version is no stable latest");
        assert_eq!(error, messages::terminal_no_stable_latest(Language::En));

        let spanish = open(
            Language::Es,
            TerminalActionView::UpgradeShell,
            HomeChoiceView::Link,
            None,
            &[],
        )
        .expect_err("nothing measured is nothing to install");
        assert_eq!(spanish, messages::terminal_no_stable_latest(Language::Es));

        assert_eq!(
            std::fs::read(&wrapper).ok(),
            before,
            "no wrapper cmd is written for a refusal"
        );
    }
}
