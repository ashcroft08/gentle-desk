//! Where a reported path sits relative to the sessions directory.
//!
//! A file watcher reports absolute paths as the running process spells them: on
//! Windows it may hand us `\\?\C:\...`, another letter case than the sessions
//! directory has, or `/` instead of `\`. All of those name one place, so both
//! sides are folded before they are compared.
//!
//! This is the notion `SessionIndex::refresh_paths` uses to accept a path (its
//! private `paths.rs`): `<sessions_dir>/<project>/<name>.jsonl`. It is
//! restated here because that module is private and its rule is not to change.

use std::path::{Component, Path};

/// The folded components of `path` below `sessions_dir`, or `None` when the
/// path is not inside it (relative, another root, or one that climbs out).
///
/// One component means a project directory, two mean a file in a project.
pub fn below(sessions_dir: &Path, path: &Path) -> Option<Vec<String>> {
    let (dir_root, dir_parts) = parts(sessions_dir)?;
    let (root, path_parts) = parts(path)?;
    if root != dir_root || path_parts.len() <= dir_parts.len() {
        return None;
    }
    if path_parts[..dir_parts.len()] != dir_parts[..] {
        return None;
    }
    Some(path_parts[dir_parts.len()..].to_vec())
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
            // A path that climbs out of its parent is not below it.
            Component::ParentDir => return None,
            Component::Normal(name) => parts.push(fold(&name.to_string_lossy())),
        }
    }
    Some((root, parts))
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
