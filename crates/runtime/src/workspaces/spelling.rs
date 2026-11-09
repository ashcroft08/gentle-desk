//! The display parts of a canonical folder path: its segments, and its name.

/// The segments of a canonical path: both separators split, empty parts dropped
/// (a leading separator and a root's trailing one say nothing about the name).
pub(super) fn segments(path: &str) -> Vec<&str> {
    path.split(['\\', '/'])
        .filter(|part| !part.is_empty())
        .collect()
}

/// The name of a folder: the last of its segments, or the whole path for a root
/// such as `C:\`, which has no name of its own.
pub(super) fn name(path: &str, segments: &[&str]) -> String {
    if is_root(path) {
        return path.to_owned();
    }
    match segments.last() {
        Some(last) => (*last).to_owned(),
        None => path.to_owned(),
    }
}

/// A canonical path keeps a trailing separator only for a root: everywhere else
/// [`crate::app_config::canonical_path`] removes it.
fn is_root(path: &str) -> bool {
    path.ends_with(['\\', '/'])
}
