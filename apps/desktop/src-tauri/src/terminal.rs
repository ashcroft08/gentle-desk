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
    launch(&program, &args)
}

/// The arguments Windows Terminal gets: one new window, one tab, then the
/// program and its own arguments after `--`. Pure, so the shape is pinned by a
/// test.
pub fn windows_terminal_args(program: &Path, args: &[OsString]) -> Vec<OsString> {
    let mut argv: Vec<OsString> = ["-w", "-1", "new-tab", "--"]
        .iter()
        .map(OsString::from)
        .collect();
    argv.push(program.as_os_str().to_owned());
    argv.extend(args.iter().cloned());
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
/// starts the program itself, so no `cmd` parses the line and no console
/// handles are inherited from this GUI process. Measured in this app: a child
/// started with `CREATE_NEW_CONSOLE` from the console-less app exits at once
/// with code 0 and prints nothing, while the very same command stays alive in a
/// terminal of its own. When `wt.exe` is missing, the direct spawn with its own
/// console is the fallback.
#[cfg(windows)]
fn launch(program: &Path, args: &[OsString]) -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    /// `CREATE_NEW_CONSOLE`: the fallback console that is visible on its own.
    const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
    /// `DETACHED_PROCESS`: Windows Terminal must not borrow this process's
    /// console, because a GUI process has none to give.
    const DETACHED_PROCESS: u32 = 0x0000_0008;

    if let Some(wt) = windows_terminal() {
        let started = std::process::Command::new(wt)
            .args(windows_terminal_args(program, args))
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
/// ships on Windows, where the console above is what makes the window visible.
#[cfg(not(windows))]
fn launch(program: &Path, args: &[OsString]) -> Result<(), String> {
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

    /// Windows Terminal gets one new window and one tab, then `--` and the
    /// program with its own arguments: nothing is parsed by cmd.
    #[test]
    fn windows_terminal_opens_one_tab_on_the_program() {
        let argv = windows_terminal_args(
            Path::new("C:/Program Files/nodejs/node.exe"),
            &os(&["C:/npm/gentle-shell.mjs", "--link"]),
        );

        assert_eq!(
            argv,
            os(&[
                "-w",
                "-1",
                "new-tab",
                "--",
                "C:/Program Files/nodejs/node.exe",
                "C:/npm/gentle-shell.mjs",
                "--link"
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
