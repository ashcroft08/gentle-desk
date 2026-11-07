//! Turns one raw watcher event into what the session index must look at.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use notify::event::{EventKind, ModifyKind};
use notify::Event;

use super::paths;

/// What one event asks of the index: these paths, or a full pass.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Classified {
    /// Session files the event touched, de-duplicated and sorted.
    pub paths: Vec<PathBuf>,
    /// Do a full pass: the watcher may have missed events.
    pub rescan: bool,
}

impl Classified {
    /// Nothing to do: no path and no full pass.
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty() && !self.rescan
    }
}

/// The effect of `event` on the index of `sessions_dir`.
///
/// - `Access` and `Other` events change nothing (opening a session is not news)
///   unless the watcher flagged them as a rescan.
/// - [`Event::need_rescan`] always means a full pass: events may have been lost.
/// - `Create`, `Modify`, `Remove` and `Any` keep the paths that are session
///   files, `<sessions_dir>/<project>/<name>.jsonl`, the locations
///   [`SessionIndex::refresh_paths`] accepts. On Windows the comparison is
///   case-insensitive because the reported path is folded first.
/// - Removing or renaming a project directory (a path directly under
///   `sessions_dir`) means a full pass: the watcher may not report the files
///   that were inside it.
/// - Anything else is ignored: a session's project is the only directory whose
///   removal hides files from us.
pub fn classify(sessions_dir: &Path, event: &Event) -> Classified {
    let mut out = Classified {
        paths: Vec::new(),
        rescan: event.need_rescan(),
    };
    if matches!(event.kind, EventKind::Access(_) | EventKind::Other) {
        return out;
    }
    let renamed = matches!(event.kind, EventKind::Modify(ModifyKind::Name(_)));
    let removed = matches!(event.kind, EventKind::Remove(_));
    for path in &event.paths {
        match paths::below(sessions_dir, path).as_deref() {
            Some([_, name]) if is_session_name(name) => out.paths.push(path.clone()),
            Some([_]) if renamed || removed => out.rescan = true,
            _ => {}
        }
    }
    out.paths.sort();
    out.paths.dedup();
    out
}

/// `.jsonl`, as `SessionIndex::refresh_paths` accepts it.
fn is_session_name(name: &str) -> bool {
    Path::new(name).extension() == Some(OsStr::new("jsonl"))
}

#[cfg(test)]
mod tests;
