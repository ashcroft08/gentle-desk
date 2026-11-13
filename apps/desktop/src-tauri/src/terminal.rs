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

/// The line `cmd` is given to run `program` and keep the window open afterwards:
/// `/k ""program" "arg" ..."`. The OUTER pair of quotes is what makes cmd keep a
/// quoted program path that holds a space (a node under `Program Files`)
/// intact; every part is quoted and written with Windows separators. A double
/// quote inside an argument would end the quoting early, so it is refused.
pub fn console_line(program: &Path, args: &[OsString]) -> Result<String, String> {
    let mut parts = Vec::with_capacity(args.len() + 1);
    parts.push(program.as_os_str().to_owned());
    parts.extend(args.iter().cloned());
    let mut line = String::from("/k \"");
    for part in parts {
        let text = part.to_string_lossy().replace('/', "\\");
        if text.contains('"') {
            return Err("The terminal could not be opened: an argument holds a quote".to_owned());
        }
        line.push('"');
        line.push_str(&text);
        line.push_str("\" ");
    }
    // The last part leaves a trailing space; the outer quote closes the line.
    line.pop();
    line.push('"');
    Ok(line)
}

/// Starts `program` in a terminal window of its own, answering as soon as the
/// window exists: nothing here waits for the window to close.
///
/// The window is `cmd /k` in a `CREATE_NEW_CONSOLE` console, built with
/// [`console_line`] and `raw_arg` (the standard `args()` quoting is the one cmd
/// mangles). `/k` keeps the window open when the command ends, so its output
/// stays readable: `pi auth login` with no provider used to print one usage line
/// and take the window with it in 0.6 s.
#[cfg(windows)]
fn launch(program: &Path, args: &[OsString]) -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    /// `CREATE_NEW_CONSOLE`: the new console is the visible terminal this
    /// command exists to open.
    const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;

    let line = console_line(program, args)?;
    std::process::Command::new("cmd")
        .raw_arg(line)
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

    /// The window runs the command under `cmd /k` so its output (and any error)
    /// stays readable after it ends. The line is `/k ""program" "arg" ..."` -
    /// the OUTER pair of quotes is what makes cmd keep the quoted program path
    /// (`C:\Program Files\...`) intact; separators are Windows ones.
    #[test]
    fn the_console_line_keeps_a_spaced_program_and_stays_open() {
        let line = console_line(
            Path::new("C:/Program Files/nodejs/node.exe"),
            &os(&["C:/npm/gentle-shell.mjs", "--isolated", "update"]),
        )
        .expect("plain arguments build a line");

        assert_eq!(
            line,
            r#"/k ""C:\Program Files\nodejs\node.exe" "C:\npm\gentle-shell.mjs" "--isolated" "update"""#
        );
    }

    /// A double quote inside an argument would end the quoting early and let the
    /// rest run as a second command, so it is refused instead of escaped.
    #[test]
    fn an_argument_with_a_quote_is_refused() {
        let refused = console_line(Path::new("C:/node.exe"), &os(&["a\"b"]));
        assert!(refused.is_err());
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
