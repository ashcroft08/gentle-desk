//! "Open in terminal" (0.1.12, S1, S2, T2, T3): the buttons that hand a job to a
//! VISIBLE terminal window, because the shell owns it and the desk only reads.
//!
//! Four jobs travel this way:
//!
//! - profiles: the shell's interactive screen, `/gentle:profiles`, named as the
//!   opening prompt. The screen is a TUI its own extension draws, reachable only
//!   by naming that command once the terminal is up, so the command is passed to
//!   the launcher as its positional prompt: that is the smallest thing that
//!   opens the right screen (no new subcommand, and no keystrokes typed into
//!   someone's window after it opens). There `n` creates a profile and `u` uses
//!   one in the session (L9), and the desk never writes the file.
//! - login / logout: the shell's own `auth login` / `auth logout`.
//! - update: the shell's own `update`, which forwards to pi's updater (L5), run
//!   from a terminal because S4 lets an update run only with every chat closed.
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
/// and the home flags.
pub fn action_args(action: TerminalActionView) -> &'static [&'static str] {
    match action {
        TerminalActionView::Profiles => &["/gentle:profiles"],
        TerminalActionView::Login => &["auth", "login"],
        TerminalActionView::Logout => &["auth", "logout"],
        TerminalActionView::Update => &["update"],
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

/// Starts `program` in a terminal window of its own, answering as soon as the
/// window exists: nothing here waits for the window to close.
#[cfg(windows)]
fn launch(program: &Path, args: &[OsString]) -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    /// `CREATE_NEW_CONSOLE`: the new console is the visible terminal this
    /// command exists to open.
    const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;

    std::process::Command::new("cmd")
        .arg("/c")
        .arg(program)
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

    /// Each button names the shell's own command, and the profiles screen is the
    /// slash command the shell already answers to.
    #[test]
    fn every_action_names_the_shells_own_command() {
        assert_eq!(
            action_args(TerminalActionView::Profiles),
            &["/gentle:profiles"]
        );
        assert_eq!(action_args(TerminalActionView::Login), &["auth", "login"]);
        assert_eq!(action_args(TerminalActionView::Logout), &["auth", "logout"]);
        assert_eq!(action_args(TerminalActionView::Update), &["update"]);
    }

    /// The command line is the launcher's own arguments, then the home flags,
    /// then the action: the same shape every other shell run of the pane has.
    #[test]
    fn the_command_is_the_launcher_then_the_home_flags_then_the_action() {
        let (program, args) =
            command_for(TerminalActionView::Login, &launcher(), &os(&["--isolated"]));

        assert_eq!(program, PathBuf::from("C:/node/node.exe"));
        assert_eq!(
            args,
            os(&["C:/npm/gentle-shell.mjs", "--isolated", "auth", "login"])
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
            os(&[
                "C:/npm/gentle-shell.mjs",
                "--home",
                "C:/h",
                "--link",
                "/gentle:profiles"
            ]),
            "the flags travel as the children get them: `--home` is the one the shell reads first"
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
