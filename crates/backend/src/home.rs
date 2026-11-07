//! Which pi home `gentle-shell` uses, and where its sessions are.
//!
//! Same rules as the reference app: `--home <dir>` is explicit, `--link` reuses
//! a plain pi install (`PI_CODING_AGENT_DIR` or `~/.pi/agent`), `--isolated`
//! uses `~/.gentle-shell/agent`; with no choice at all the home is isolated.

use std::ffi::OsString;
use std::path::PathBuf;

/// The environment the rules read.
#[derive(Debug, Clone, Default)]
pub struct HomeEnv {
    /// The user's home directory (`USERPROFILE` / `HOME`).
    pub user_home: Option<PathBuf>,
    pub pi_coding_agent_dir: Option<PathBuf>,
    pub gentle_shell_home: Option<PathBuf>,
}

impl HomeEnv {
    pub fn from_process() -> Self {
        let var = |name: &str| {
            std::env::var_os(name)
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
        };
        Self {
            user_home: var("USERPROFILE").or_else(|| var("HOME")),
            pi_coding_agent_dir: var("PI_CODING_AGENT_DIR"),
            gentle_shell_home: var("GENTLE_SHELL_HOME"),
        }
    }
}

/// The flags to start `gentle-shell` with: `args` as given, plus `--isolated`
/// (or `--home` from `GENTLE_SHELL_HOME`) when they name no home.
pub fn with_home_flag(mut args: Vec<OsString>, env: &HomeEnv) -> Vec<OsString> {
    if names_a_home(&args) {
        return args;
    }
    match &env.gentle_shell_home {
        Some(dir) => args.extend(["--home".into(), dir.clone().into_os_string()]),
        None => args.push("--isolated".into()),
    }
    args
}

/// The agent directory the flags in `args` select; its `sessions/` holds the sessions.
pub fn agent_dir(args: &[OsString], env: &HomeEnv) -> Option<PathBuf> {
    let flag = |name: &str| args.iter().any(|arg| arg == name);
    let isolated = || Some(env.user_home.as_ref()?.join(".gentle-shell").join("agent"));
    if let Some(position) = args.iter().position(|arg| arg == "--home") {
        return args.get(position + 1).map(PathBuf::from);
    }
    if flag("--link") {
        return env
            .pi_coding_agent_dir
            .clone()
            .or_else(|| Some(env.user_home.as_ref()?.join(".pi").join("agent")));
    }
    if flag("--isolated") {
        return isolated();
    }
    env.gentle_shell_home.clone().or_else(isolated)
}

fn names_a_home(args: &[OsString]) -> bool {
    args.iter()
        .any(|arg| arg == "--home" || arg == "--link" || arg == "--isolated")
}

#[cfg(test)]
mod tests;
