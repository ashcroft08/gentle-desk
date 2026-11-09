//! The order of the rows: by last activity, newest first; the folders with no
//! chat after them, in the order the config lists them.

use std::cmp::Ordering;

use super::group::Folder;

/// Orders `folders` in place: the ones with activity by [`Folder::last_activity_ms`]
/// descending and then by [`Folder::key`], the ones without activity by
/// [`Folder::config_rank`] and then by [`Folder::key`]. The two groups never
/// mix: a folder with no chat always comes after one with a chat.
pub(super) fn sort(folders: &mut [Folder]) {
    folders.sort_by(|a, b| match (a.last_activity_ms, b.last_activity_ms) {
        (Some(a_ms), Some(b_ms)) => b_ms.cmp(&a_ms).then_with(|| a.key.cmp(&b.key)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => rank(a).cmp(&rank(b)).then_with(|| a.key.cmp(&b.key)),
    });
}

/// Where a folder with no chat sits among the others with no chat: the order
/// the config lists it in. A folder the config does not hold has no such order
/// and goes after the ones it has; nothing builds one today, since a folder
/// only the sessions report always has a session.
fn rank(folder: &Folder) -> usize {
    folder.config_rank.unwrap_or(usize::MAX)
}
