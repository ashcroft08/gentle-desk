//! Resolution of the `gentle-shell` launcher.
//!
//! On Windows the global install is an npm shim (`gentle-shell.cmd`) that
//! cannot be executed without a shell by runtimes that do not go through
//! `cmd.exe` (Node failed with EINVAL in the reference app). Even where it can
//! be spawned, `cmd.exe` becomes the child and `node` a grandchild, so
//! closing the session would leave `node` orphaned. So a `.cmd` shim is
//! resolved to `node` plus the real JS entry it points at, and only falls back
//! to `cmd.exe /c` when the shim cannot be understood.

use std::ffi::{OsStr, OsString};
use std::fmt;
use std::path::{Path, PathBuf};

/// A program plus the leading arguments needed to run `gentle-shell` with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Launcher {
    pub program: PathBuf,
    pub args: Vec<OsString>,
}

/// The environment inputs resolution depends on.
#[derive(Debug, Clone, Default)]
pub struct LauncherEnv {
    /// `GENTLE_SHELL_BIN`: a JS entry, a shim or a directly executable file.
    pub gentle_shell_bin: Option<OsString>,
    /// The `PATH` variable.
    pub path: Option<OsString>,
    /// Folders searched after `PATH`, in order: on Windows the npm global
    /// folder, which is where `npm install -g` puts the shim and which is not
    /// usually on `PATH`.
    pub extra_dirs: Vec<PathBuf>,
}

impl LauncherEnv {
    pub fn from_process() -> Self {
        Self {
            gentle_shell_bin: std::env::var_os("GENTLE_SHELL_BIN").filter(|v| !v.is_empty()),
            path: std::env::var_os("PATH"),
            extra_dirs: extra_dirs_from(std::env::var_os("APPDATA").as_deref(), cfg!(windows)),
        }
    }
}

/// The folders searched after `PATH`. On Windows that is the npm global folder
/// (`%APPDATA%\npm`), where npm puts `gentle-shell.cmd`; elsewhere there is no
/// such folder, so the list stays empty.
fn extra_dirs_from(appdata: Option<&OsStr>, windows: bool) -> Vec<PathBuf> {
    if !windows {
        return Vec::new();
    }
    appdata
        .map(Path::new)
        .filter(|dir| !dir.as_os_str().is_empty())
        .map(|dir| vec![dir.join("npm")])
        .unwrap_or_default()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LauncherError {
    /// `GENTLE_SHELL_BIN` names a file that does not exist.
    BinMissing(PathBuf),
    /// Nothing on `PATH`, in the extra folders, and no `GENTLE_SHELL_BIN`.
    NotFound,
}

impl fmt::Display for LauncherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BinMissing(path) => write!(
                f,
                "GENTLE_SHELL_BIN points to {}, which does not exist",
                path.display()
            ),
            Self::NotFound => write!(
                f,
                "gentle-shell launcher not found: looked in the folders of PATH and in the \
                 npm global folder (%APPDATA%\\npm on Windows). Install it with \
                 `npm install -g gentle-pi`, or set GENTLE_SHELL_BIN to its full path"
            ),
        }
    }
}

impl std::error::Error for LauncherError {}

/// Resolves the launcher for the current platform.
pub fn resolve(env: &LauncherEnv) -> Result<Launcher, LauncherError> {
    resolve_for(env, cfg!(windows))
}

/// Same as [`resolve`] with the platform explicit, so both branches are testable anywhere.
pub fn resolve_for(env: &LauncherEnv, windows: bool) -> Result<Launcher, LauncherError> {
    if let Some(bin) = &env.gentle_shell_bin {
        let bin = PathBuf::from(bin);
        if !bin.is_file() {
            return Err(LauncherError::BinMissing(bin));
        }
        return Ok(launcher_for_file(&bin, env, windows));
    }
    find_launcher(env, windows).ok_or(LauncherError::NotFound)
}

/// `PATH` first, then the extra folders, each looked in for the platform's
/// launcher names. A hit goes through the same shim handling either way.
fn find_launcher(env: &LauncherEnv, windows: bool) -> Option<Launcher> {
    let names: &[&str] = if windows {
        &["gentle-shell.cmd", "gentle-shell.exe"]
    } else {
        &["gentle-shell"]
    };
    let on_path = env
        .path
        .as_deref()
        .map(std::env::split_paths)
        .into_iter()
        .flatten();
    let extra = env.extra_dirs.iter().cloned();
    for dir in on_path.chain(extra).filter(|d| !d.as_os_str().is_empty()) {
        for name in names {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(launcher_for_file(&candidate, env, windows));
            }
        }
    }
    None
}

fn launcher_for_file(file: &Path, env: &LauncherEnv, windows: bool) -> Launcher {
    let ext = file
        .extension()
        .and_then(OsStr::to_str)
        .map(str::to_ascii_lowercase);
    match ext.as_deref() {
        Some("mjs" | "js" | "cjs") => node_launcher(file, env, windows, None),
        Some("cmd" | "bat") if windows => resolve_shim(file, env),
        _ => Launcher {
            program: file.to_path_buf(),
            args: Vec::new(),
        },
    }
}

/// `node <entry>`; `node` is the `node.exe` next to the shim when there is one
/// (npm shims prefer it), else the first `node` on `PATH`, else bare `node`.
fn node_launcher(
    entry: &Path,
    env: &LauncherEnv,
    windows: bool,
    beside: Option<&Path>,
) -> Launcher {
    let node_name = if windows { "node.exe" } else { "node" };
    let program = beside
        .map(|dir| dir.join(node_name))
        .filter(|p| p.is_file())
        .or_else(|| {
            let path = env.path.as_deref()?;
            std::env::split_paths(path)
                .map(|dir| dir.join(node_name))
                .find(|p| p.is_file())
        })
        .unwrap_or_else(|| PathBuf::from("node"));
    Launcher {
        program,
        args: vec![entry.into()],
    }
}

fn resolve_shim(shim: &Path, env: &LauncherEnv) -> Launcher {
    let dir = shim.parent().unwrap_or(Path::new("."));
    if let Some(entry) = std::fs::read_to_string(shim)
        .ok()
        .and_then(|text| shim_js_entry(&text))
        .map(|relative| dir.join(relative))
        .filter(|entry| entry.is_file())
    {
        return node_launcher(&entry, env, true, Some(dir));
    }
    Launcher {
        program: PathBuf::from("cmd.exe"),
        args: vec!["/c".into(), shim.into()],
    }
}

/// The JS entry an npm `.cmd` shim runs: the quoted `%dp0%\...` token with a
/// JS extension, relative to the shim's directory.
fn shim_js_entry(text: &str) -> Option<String> {
    text.split('"')
        .filter_map(|token| token.strip_prefix("%dp0%"))
        .map(|rest| rest.trim_start_matches(['\\', '/']))
        .find(|rest| {
            let lower = rest.to_ascii_lowercase();
            [".mjs", ".js", ".cjs"].iter().any(|e| lower.ends_with(e))
        })
        .map(str::to_owned)
}

#[cfg(test)]
mod tests;
