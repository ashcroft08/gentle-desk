//! Which pi home `gentle-shell` uses, and where its sessions are.
//!
//! Same rules as the reference app: `--home <dir>` is explicit, `--link` reuses
//! a plain pi install (`PI_CODING_AGENT_DIR` or `~/.pi/agent`), `--isolated`
//! uses `~/.gentle-shell/agent`; with no choice at all the home is isolated.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Which home the user chose for `gentle-shell` (the welcome screen's two
/// cards): the user's own pi, or a space of its own. Spelled the way the app
/// config stores it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HomeChoice {
    /// `--link`: reuse the plain pi install (`~/.pi/agent`).
    #[serde(rename = "link")]
    Link,
    /// `--isolated`: `~/.gentle-shell/agent`, nothing of pi touched.
    #[serde(rename = "isolated")]
    Isolated,
}

impl HomeChoice {
    /// The choice `name` spells, `None` for anything else.
    pub fn from_name(name: &str) -> Option<HomeChoice> {
        match name {
            "link" => Some(HomeChoice::Link),
            "isolated" => Some(HomeChoice::Isolated),
            _ => None,
        }
    }

    /// The flag `gentle-shell` takes for this choice.
    pub fn flag(self) -> &'static str {
        match self {
            HomeChoice::Link => "--link",
            HomeChoice::Isolated => "--isolated",
        }
    }
}

/// The environment the rules read.
#[derive(Debug, Clone, Default)]
pub struct HomeEnv {
    /// The user's home directory (`USERPROFILE` / `HOME`).
    pub user_home: Option<PathBuf>,
    pub pi_coding_agent_dir: Option<PathBuf>,
    pub gentle_shell_home: Option<PathBuf>,
    /// What the user chose on the welcome screen, if they did. The developer's
    /// `GENTLE_SHELL_HOME` still wins over it.
    pub choice: Option<HomeChoice>,
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
            choice: None,
        }
    }
}

/// The flags to start `gentle-shell` with: `args` as given, plus a home when they
/// name none: `--home` from `GENTLE_SHELL_HOME` (the developer's override), else
/// the user's choice, else `--isolated`.
pub fn with_home_flag(mut args: Vec<OsString>, env: &HomeEnv) -> Vec<OsString> {
    if names_a_home(&args) {
        return args;
    }
    match &env.gentle_shell_home {
        Some(dir) => args.extend(["--home".into(), dir.clone().into_os_string()]),
        None => args.push(env.choice.unwrap_or(HomeChoice::Isolated).flag().into()),
    }
    args
}

/// The folder of the user's own pi when it exists: `PI_CODING_AGENT_DIR`, else
/// `~/.pi/agent`. `exists` is the file system's answer, so only the folder is
/// asked about and nothing in it is read.
pub fn detect_pi(env: &HomeEnv, exists: impl Fn(&Path) -> bool) -> Option<PathBuf> {
    let dir = env
        .pi_coding_agent_dir
        .clone()
        .or_else(|| Some(env.user_home.as_ref()?.join(".pi").join("agent")))?;
    exists(&dir).then_some(dir)
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
