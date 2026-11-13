//! What the Settings dialog reads from the machine (0.1.12): gentle-pi's agent
//! profiles, where the shell's authentication stands, and whether the three
//! pieces this machine runs have newer versions published.
//!
//! Every read is a SEAM of its own - a file reader, a per-provider check, a
//! version probe and one GET - so the bridge can be driven windowless and no
//! test of it reaches the network, a launcher or the user's own files. The rules
//! those seams feed are pure and live in `gentle_runtime` (`profiles`, `auth`,
//! `updates`); this module only turns their answers into the contract's views
//! and supplies the real implementations.
//!
//! Nothing here writes anything: gentle-desk reads gentle-pi's files and never
//! edits them, which is the standing decision of the settings roadmap.

use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use gentle_backend::home::HomeChoice;
use gentle_backend::launcher::{self, LauncherEnv};

mod auth;
mod paths;
mod profiles;
mod updates;
mod web;

pub use auth::view as auth_view;
pub use paths::SettingsPaths;
pub use profiles::view as profiles_view;
pub use updates::{view as updates_view, PI_REGISTRY};

/// Reads one file of the machine. `Ok(None)` is a file that is not there - an
/// auth store nobody has written yet is not a failure - and `Err` is a file that
/// is there and could not be read.
pub type FileRead = Arc<dyn Fn(&Path) -> Result<Option<String>, String> + Send + Sync>;

/// What one `gentle-shell auth check --provider <id> --json --no-refresh` run
/// printed: its exit code (the shell's own contract) and its output. The
/// provider id is the only thing that travels in; nothing of the store does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderAnswer {
    pub exit_code: i32,
    pub stdout: String,
}

/// Runs the shell's own check for one provider. A run that could not start at
/// all is the `Err`, which the pane draws as that provider's error.
pub type ProviderProbe = Arc<dyn Fn(&str) -> Result<ProviderAnswer, String> + Send + Sync>;

/// Runs `gentle-shell --version` and answers what it printed: the line for the
/// shell and the line for the pi it runs, both read by
/// [`gentle_runtime::updates::parse_version_output`].
pub type VersionProbe = Arc<dyn Fn() -> Result<String, String> + Send + Sync>;

/// One GET of an https address, answering the body. Used for the release
/// manifest (the desk's own version) and for the npm registry (pi's).
pub type WebGet = Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>;

/// How long one GET of the update check may take, connection included: the
/// check is best effort, so a server that does not answer in time leaves that
/// row unknown instead of holding the check open.
const WEB_TIMEOUT: Duration = Duration::from_secs(8);

/// Everything the Settings pane needs to read the machine, gathered once per run:
/// the three paths it reads, and the four seams that answer. The bridge owns one
/// value, so a test replaces any single seam and everything else stays real.
pub struct SettingsSeams {
    /// The files this run reads: `profiles.json`, its sibling
    /// `profile-fallbacks.json` (L13) and `auth.json`.
    pub paths: SettingsPaths,
    pub files: FileRead,
    /// One `auth check` per provider.
    pub probe: ProviderProbe,
    /// `gentle-shell --version`, the one process that names both versions.
    pub versions: VersionProbe,
    /// The release manifest and the registry, one GET each.
    pub web: WebGet,
}

impl SettingsSeams {
    /// The real seams of this run: the files and flags the home choice resolves,
    /// the launcher this machine has, and one bounded GET.
    pub fn new(choice: Option<HomeChoice>) -> Self {
        let paths = paths::resolve(choice);
        let home_args = Arc::new(paths.home_args.clone());
        Self {
            paths,
            files: file_read(),
            probe: provider_probe(Arc::clone(&home_args)),
            versions: version_probe(home_args),
            web: web_get(),
        }
    }
}

/// The real file reader: the file's text, `None` when there is no such file.
pub fn file_read() -> FileRead {
    Arc::new(|path: &Path| match std::fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("{} could not be read: {error}", path.display())),
    })
}

/// The real per-provider check: the launcher this machine resolves, run with the
/// home flags the children get and the shell's own `auth check`.
///
/// `--no-refresh` is part of the command, not a courtesy: without it the check
/// would make pi refresh an OAuth token, and the terminal owns that.
pub fn provider_probe(home_args: Arc<Vec<OsString>>) -> ProviderProbe {
    Arc::new(move |provider: &str| {
        let output = run(&provider_args(&home_args, provider))?;
        Ok(ProviderAnswer {
            // A signal-terminated run has no code: -1 is no exit code the shell
            // uses, so the pane reports it as an error.
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        })
    })
}

/// The command line one provider's check runs: the home flags, then the shell's
/// own check with `--no-refresh`.
pub fn provider_args(home_args: &[OsString], provider: &str) -> Vec<OsString> {
    let mut args = home_args.to_vec();
    for arg in [
        "auth",
        "check",
        "--provider",
        provider,
        "--json",
        "--no-refresh",
    ] {
        args.push(OsString::from(arg));
    }
    args
}

/// The real version probe: `gentle-shell --version`, which prints the shell's
/// version and the version of the pi it runs, with the same home flags.
pub fn version_probe(home_args: Arc<Vec<OsString>>) -> VersionProbe {
    Arc::new(move || {
        let mut args = (*home_args).clone();
        args.push(OsString::from("--version"));
        let output = run(&args)?;
        let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
        // The launcher may split its own output between the streams; both are
        // read so a warning line cannot hide the versions.
        text.push('\n');
        text.push_str(&String::from_utf8_lossy(&output.stderr));
        Ok(text)
    })
}

/// The real GET: one bounded request through the same TLS stack the updater
/// plugin already brings.
pub fn web_get() -> WebGet {
    Arc::new(|url: &str| web::get(url, WEB_TIMEOUT))
}

/// Runs the resolved `gentle-shell` with `args`, answering what it printed.
fn run(args: &[OsString]) -> Result<std::process::Output, String> {
    let launcher = launcher::resolve(&LauncherEnv::from_process())
        .map_err(|error| format!("gentle-shell could not be resolved: {error}"))?;
    std::process::Command::new(&launcher.program)
        .args(&launcher.args)
        .args(args)
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|error| format!("gentle-shell could not run: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_that_is_not_there_is_no_file_and_not_a_failure() {
        let dir = std::env::temp_dir().join(format!("gentle-desk-seams-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("the scratch directory is creatable");
        let missing = dir.join("nothing-here.json");
        let present = dir.join("here.json");
        std::fs::write(&present, "{}").expect("the scratch file is writable");

        let read = file_read();
        assert_eq!(read(&missing), Ok(None));
        assert_eq!(read(&present), Ok(Some("{}".to_owned())));
        // A folder is not a file: that is a read that failed, not a missing one.
        assert!(read(&dir).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The command line one provider's check runs, pinned without starting
    /// anything: the home flags first, then the shell's own check, and
    /// `--no-refresh` always.
    #[test]
    fn the_provider_check_asks_the_shell_with_no_refresh() {
        let home = vec![OsString::from("--isolated")];

        assert_eq!(
            provider_args(&home, "anthropic"),
            vec![
                OsString::from("--isolated"),
                OsString::from("auth"),
                OsString::from("check"),
                OsString::from("--provider"),
                OsString::from("anthropic"),
                OsString::from("--json"),
                OsString::from("--no-refresh"),
            ]
        );
    }

    #[test]
    fn the_provider_check_passes_the_home_flags_it_was_given() {
        // A linked home is the same command with the flag that names it, so the
        // check always reads the store the chats read.
        let home = vec![OsString::from("--home"), OsString::from("C:/h")];
        let args = provider_args(&home, "openai-codex");
        assert_eq!(&args[..2], &home[..]);
        assert_eq!(args[3], OsString::from("check"));
        assert_eq!(args[5], OsString::from("openai-codex"));
    }
}
