//! Where a session file may live, and the spelling the cache uses for it.
//!
//! A file watcher reports absolute paths as the running process sees them: on
//! Windows it may hand us `\\?\C:\...`, a different case than the sessions
//! directory has, or `/` instead of `\`. All of those name one place, so every
//! reported path is folded before it is compared.

use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, PathBuf};

/// `<sessions_dir>/<project>/<name>.jsonl` as the cache spells it, plus the
/// folded form that makes two spellings of one file compare equal.
///
/// The key is always spelled from `sessions_dir`, so it is the key `refresh`
/// uses; a path whose project or file name differs from disk in letter case
/// (Windows) still gets its own spelling until the next full pass drops it.
///
/// `None` for anything that is not a session file location: a relative path,
/// another directory, another depth, another extension.
pub fn session_key(sessions_dir: &Path, path: &Path) -> Option<(PathBuf, String)> {
    let (dir_root, dir_parts) = parts(sessions_dir)?;
    let (root, file_parts) = parts(path)?;
    if root != dir_root || file_parts.len() != dir_parts.len() + 2 {
        return None;
    }
    if file_parts[..dir_parts.len()] != dir_parts[..] {
        return None;
    }
    let names = names(path);
    let [project, name] = names.get(dir_parts.len()..)? else {
        return None;
    };
    // `session_files` matches the extension exactly: keep one rule for both.
    if Path::new(name).extension() != Some(OsStr::new("jsonl")) {
        return None;
    }
    let folded = format!("{root}/{}", file_parts.join("/"));
    Some((sessions_dir.join(project).join(name), folded))
}

/// An absolute path as a root key plus its folded components, so two spellings
/// of one place compare equal.
fn parts(path: &Path) -> Option<(String, Vec<String>)> {
    if !path.is_absolute() {
        return None;
    }
    let mut root = String::new();
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => root = fold(&plain(&prefix.as_os_str().to_string_lossy())),
            Component::RootDir | Component::CurDir => {}
            // A path that climbs out of its parent is not a session file.
            Component::ParentDir => return None,
            Component::Normal(name) => parts.push(fold(&name.to_string_lossy())),
        }
    }
    Some((root, parts))
}

/// The `Normal` components of `path`, in order: its spelling on disk.
fn names(path: &Path) -> Vec<OsString> {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(name) => Some(name.to_os_string()),
            _ => None,
        })
        .collect()
}

/// `\\?\C:` and `\??\C:` mean `C:`, `\\?\UNC\srv\share` means `\\srv\share`.
fn plain(prefix: &str) -> String {
    let prefix = prefix
        .strip_prefix(r"\\?\")
        .or_else(|| prefix.strip_prefix(r"\??\"))
        .unwrap_or(prefix);
    let share = prefix
        .strip_prefix("UNC")
        .or_else(|| prefix.strip_prefix("unc"));
    match share {
        Some(share) => format!(r"\\{}", share.trim_start_matches(['\\', '/'])),
        None => prefix.to_string(),
    }
}

/// Windows paths are case-insensitive: `C:\Home` and `c:\home` are one place.
#[cfg(windows)]
fn fold(text: &str) -> String {
    text.to_lowercase()
}

#[cfg(not(windows))]
fn fold(text: &str) -> String {
    text.to_string()
}

#[cfg(test)]
mod tests;
