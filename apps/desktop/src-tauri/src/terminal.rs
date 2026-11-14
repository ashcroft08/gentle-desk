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

use crate::contract::TerminalActionView;

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

/// Opens a visible terminal window on `action`, with the home flags the chats'
/// children get.
pub fn open(action: TerminalActionView, home_args: &[OsString]) -> Result<(), String> {
    let launcher = launcher::resolve(&LauncherEnv::from_process()).map_err(|error| {
        format!("The terminal could not be opened: gentle-shell could not be found ({error})")
    })?;
    let (program, args) = command_for(action, &launcher, home_args);
    launch(action, &program, &args)
}

/// The plain label one action's window and tab wear, in the pane's own words:
/// the user pressed a button named this, so the window says the same thing.
/// ASCII, no file, no product's command line.
pub fn action_label(action: TerminalActionView) -> &'static str {
    match action {
        TerminalActionView::Profiles => "Profiles",
        // The buttons are pi's own `/login` and `/logout` inside the shell, so
        // the labels are the words those buttons would wear.
        TerminalActionView::Login => "Sign in",
        TerminalActionView::Logout => "Sign out",
        TerminalActionView::Update => "Update",
        TerminalActionView::UpdateExtensions => "Update extensions",
        TerminalActionView::InstallUsageSources => "Install usage sources",
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
pub fn wrapper_script(program: &Path, args: &[OsString]) -> Result<String, String> {
    let mut line = quote_part(&program.to_string_lossy())?;
    for arg in args {
        line.push(' ');
        line.push_str(&quote_part(&arg.to_string_lossy())?);
    }

    Ok(format!("@echo off\r\n{line}\r\necho.\r\npause\r\n"))
}

/// One part of the wrapper's command line, in double quotes, with the
/// separators cmd wants.
fn quote_part(part: &str) -> Result<String, String> {
    if part.contains('"') {
        return Err(format!(
            "The terminal could not be opened: {part} carries a double quote, which a cmd line cannot pass on"
        ));
    }
    Ok(format!("\"{}\"", part.replace('/', "\\")))
}

/// The arguments Windows Terminal gets: one new window, one tab wearing the
/// action's label, then the wrapper cmd after `--`. Pure, so the shape is
/// pinned by a test.
pub fn windows_terminal_args(action: TerminalActionView, wrapper: &Path) -> Vec<OsString> {
    let mut argv: Vec<OsString> = ["-w", "-1", "new-tab", "--title", action_label(action), "--"]
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
fn launch(action: TerminalActionView, program: &Path, args: &[OsString]) -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    /// `CREATE_NEW_CONSOLE`: the fallback console that is visible on its own.
    const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
    /// `DETACHED_PROCESS`: Windows Terminal must not borrow this process's
    /// console, because a GUI process has none to give.
    const DETACHED_PROCESS: u32 = 0x0000_0008;

    if let Some(wt) = windows_terminal() {
        let script = wrapper_script(program, args)?;
        let wrapper = wrapper_path(action);
        std::fs::write(&wrapper, script).map_err(|error| {
            format!(
                "The terminal could not be opened: {} could not be written ({error})",
                wrapper.display()
            )
        })?;
        let started = std::process::Command::new(wt)
            .args(windows_terminal_args(action, &wrapper))
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
        .map_err(|error| format!("The terminal could not be opened: {error}"))
}

/// Everywhere else there is no console flag to set: the launcher is started as
/// it is and inherits the terminal the app was started from. The desktop app
/// ships on Windows, where the console above is what makes the window visible,
/// so no wrapper is written here.
#[cfg(not(windows))]
fn launch(_action: TerminalActionView, program: &Path, args: &[OsString]) -> Result<(), String> {
    std::process::Command::new(program)
        .args(args)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("The terminal could not be opened: {error}"))
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
        let script = wrapper_script(Path::new("C:/node/node.exe"), &os(&["update"])).unwrap();

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
        let error = wrapper_script(Path::new("C:/node/node.exe"), &os(&["C:/a\"b.mjs"]))
            .expect_err("an argument with a double quote is refused");
        assert!(
            error.contains("double quote"),
            "the reason names what cannot be passed on: {error}"
        );
        assert!(
            wrapper_script(Path::new("C:/no\"de/node.exe"), &[]).is_err(),
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
            TerminalActionView::UpdateExtensions,
            TerminalActionView::InstallUsageSources,
        ];

        let labels: Vec<&str> = actions.iter().copied().map(action_label).collect();
        assert_eq!(
            labels,
            vec![
                "Profiles",
                "Sign in",
                "Sign out",
                "Update",
                "Update extensions",
                "Install usage sources"
            ]
        );
        for label in labels {
            assert!(label.is_ascii() && !label.is_empty(), "{label:?}");
        }
    }

    /// Windows Terminal gets one new window, one tab with the action's title,
    /// then `--` and the WRAPPER cmd: nothing is parsed by this process, and
    /// cmd is what keeps the window open after the program exits.
    #[test]
    fn windows_terminal_opens_one_titled_tab_on_the_wrapper() {
        let argv = windows_terminal_args(
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
}
