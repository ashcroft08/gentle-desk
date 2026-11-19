//! The files a chat's message carries (design/composer-dock T4, S3, R5, R6).
//!
//! Two rules live here, and the web side only draws their answer:
//!
//! - Images still travel as images (R6): a path whose extension is one of
//!   [`crate::images::Mime`]'s own is `Image`, and the composer hands it to the
//!   command that already reads and serves them.
//! - Every other file travels as its PATH: it becomes a chip whose whole path the
//!   message carries at the end, one per line, and the agent opens it with its
//!   own tools.
//!
//! A path that is a folder, is missing, or cannot be read is refused with the
//! reason instead of becoming a chip. A file outside the chat's folder is NOT
//! refused (R5): it attaches anyway, and `outside` only makes the composer say
//! the agent may not be able to open it.

use std::collections::HashSet;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::contract::{AttachmentKindView, AttachmentRejectView, AttachmentView};
use crate::images::Mime;
use crate::picker::FilePicker;

/// What each path the user chose or dropped is, in the order they named them.
///
/// `folder` is the foreground chat's working directory, when there is one: a
/// path that is not under it is marked `outside`. A run of the same path twice
/// answers once (the first wins), because a message never carries a file twice.
pub fn describe(paths: &[String], folder: Option<&str>) -> Vec<AttachmentView> {
    let root = folder.and_then(|folder| std::fs::canonicalize(folder).ok());
    let mut seen: HashSet<PathBuf> = HashSet::new();
    let mut views = Vec::new();
    for named in paths {
        let Some(view) = one(named, root.as_deref(), &mut seen) else {
            continue;
        };
        views.push(view);
    }
    views
}

/// One path, or `None` when the same file was already named: the de-duplication
/// is by the canonical path, so two spellings of one file answer once.
fn one(named: &str, root: Option<&Path>, seen: &mut HashSet<PathBuf>) -> Option<AttachmentView> {
    let path = Path::new(named);
    let (kind, canonical) = classify(path);
    if let Some(canonical) = canonical.as_ref() {
        if !seen.insert(canonical.clone()) {
            return None;
        }
    }
    let outside = match (root, canonical.as_deref()) {
        (Some(root), Some(canonical)) => !under(root, canonical),
        // A chat with no folder to be outside of, and a path that was refused
        // before it could be resolved, are never "outside": saying so would name
        // a comparison nobody made.
        _ => false,
    };
    let shown = canonical
        .as_deref()
        .map(displayable)
        .unwrap_or_else(|| named.to_owned());
    Some(AttachmentView {
        path: shown,
        name: name_of(path),
        kind,
        outside,
    })
}

/// What one path is, and the canonical path when it resolved. The order is the
/// rule: a folder named `notes.png` is a folder, and a file that cannot be opened
/// is refused before its extension is looked at.
fn classify(path: &Path) -> (AttachmentKindView, Option<PathBuf>) {
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return (
                AttachmentKindView::Rejected(AttachmentRejectView::NotFound),
                None,
            )
        }
        Err(_) => {
            return (
                AttachmentKindView::Rejected(AttachmentRejectView::Unreadable),
                None,
            )
        }
    };
    if metadata.is_dir() {
        return (
            AttachmentKindView::Rejected(AttachmentRejectView::IsDirectory),
            None,
        );
    }
    // Opened, never read: what the rule asks is whether the agent's own tools
    // will be able to read it at all.
    if std::fs::File::open(path).is_err() {
        return (
            AttachmentKindView::Rejected(AttachmentRejectView::Unreadable),
            None,
        );
    }
    let Ok(canonical) = std::fs::canonicalize(path) else {
        return (
            AttachmentKindView::Rejected(AttachmentRejectView::Unreadable),
            None,
        );
    };
    let kind = if is_image(&canonical) {
        AttachmentKindView::Image
    } else {
        AttachmentKindView::File
    };
    (kind, Some(canonical))
}

/// Whether `path` is one of the image types a message carries. The rule is the
/// one `attach_images` already applies ([`Mime::of_extension`]), never a second
/// list that could drift from it.
fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| Mime::of_extension(extension).is_some())
}

/// The file's own name: the last segment, which is what the chip draws.
fn name_of(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| displayable(path))
}

/// A path as the app shows it: Windows' `canonicalize` answers the verbatim form
/// (`\\?\C:\...`, `\\?\UNC\server\share`), and the app names paths as the user
/// does everywhere else.
pub fn displayable(path: &Path) -> String {
    let text = path.to_string_lossy();
    if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{rest}");
    }
    if let Some(rest) = text.strip_prefix(r"\\?\") {
        return rest.to_owned();
    }
    text.into_owned()
}

/// Whether `path` is `folder` or lives under it. Windows compares paths without
/// regard to case; every other platform keeps the exact spelling.
fn under(folder: &Path, path: &Path) -> bool {
    let folder = comparable(folder);
    let path = comparable(path);
    if path == folder {
        return true;
    }
    let separator = std::path::MAIN_SEPARATOR;
    let prefix = if folder.ends_with(separator) {
        folder
    } else {
        format!("{folder}{separator}")
    };
    path.starts_with(&prefix)
}

/// One canonical path as the comparison sees it: with no verbatim prefix, and
/// folded to one case on Windows.
fn comparable(path: &Path) -> String {
    let text = displayable(path);
    if cfg!(windows) {
        text.to_lowercase()
    } else {
        text
    }
}

/// The paths the human picked, as the web side receives them: what the dialog
/// answered, and the empty list for a cancelled pick.
///
/// The picker runs on the caller's thread, which must not be the window's and
/// must hold no lock: a dialog can stay open for minutes.
pub fn pick(picker: &dyn FilePicker, title: &str) -> Vec<String> {
    picker
        .pick_files(title)
        .iter()
        .map(|path| displayable(path))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One real file of `dir`, written with `bytes`: the rules below are about
    /// what is on the disk, so nothing here fakes a metadata.
    fn file(dir: &Path, name: &str, bytes: &[u8]) -> String {
        let path = dir.join(name);
        std::fs::write(&path, bytes).unwrap();
        path.to_string_lossy().into_owned()
    }

    fn kind_of(view: &AttachmentView) -> &AttachmentKindView {
        &view.kind
    }

    #[test]
    fn an_image_is_decided_by_the_rule_attach_images_uses() {
        let dir = tempfile::tempdir().unwrap();
        let png = file(dir.path(), "shot.PNG", b"x");
        let notes = file(dir.path(), "notes.txt", b"x");

        let views = describe(&[png.clone(), notes.clone()], None);

        assert_eq!(views[0].kind, AttachmentKindView::Image);
        assert_eq!(views[0].name, "shot.PNG");
        assert_eq!(views[1].kind, AttachmentKindView::File);
        assert_eq!(views[1].name, "notes.txt");
        // The whole path, absolute, and no verbatim prefix on it.
        assert!(views[0].path.ends_with("shot.PNG"), "got {}", views[0].path);
        assert!(
            Path::new(&views[0].path).is_absolute(),
            "got {}",
            views[0].path
        );
        assert!(!views[0].path.starts_with(r"\\?\"), "got {}", views[0].path);
    }

    #[test]
    fn a_directory_and_a_missing_file_are_refused_with_their_reason() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("assets");
        std::fs::create_dir(&folder).unwrap();
        let missing = dir.path().join("gone.md");

        let views = describe(
            &[
                folder.to_string_lossy().into_owned(),
                missing.to_string_lossy().into_owned(),
            ],
            None,
        );

        assert_eq!(
            kind_of(&views[0]),
            &AttachmentKindView::Rejected(AttachmentRejectView::IsDirectory)
        );
        assert_eq!(views[0].name, "assets");
        assert_eq!(
            kind_of(&views[1]),
            &AttachmentKindView::Rejected(AttachmentRejectView::NotFound)
        );
        assert_eq!(views[1].name, "gone.md");
        // A refused path is never "outside": no comparison was made.
        assert!(!views[0].outside && !views[1].outside);
    }

    #[test]
    fn a_file_outside_the_chat_folder_is_inside_when_it_is_under_it() {
        let chat = tempfile::tempdir().unwrap();
        let inside = file(chat.path(), "inside.md", b"x");
        let elsewhere = tempfile::tempdir().unwrap();
        let outside = file(elsewhere.path(), "outside.md", b"x");
        let folder = chat.path().to_string_lossy().into_owned();

        let views = describe(&[inside.clone(), outside.clone()], Some(&folder));

        assert!(!views[0].outside, "{} is under {}", inside, folder);
        assert!(views[1].outside, "{} is not under {}", outside, folder);
    }

    #[test]
    fn a_chat_with_no_folder_has_nothing_outside_it() {
        let dir = tempfile::tempdir().unwrap();
        let notes = file(dir.path(), "notes.md", b"x");

        let views = describe(&[notes], None);

        assert!(!views[0].outside);
    }

    #[test]
    fn the_same_file_twice_is_attached_once() {
        let dir = tempfile::tempdir().unwrap();
        let notes = file(dir.path(), "notes.md", b"x");
        let same = dir.path().join(".").join("notes.md");

        let views = describe(&[notes, same.to_string_lossy().into_owned()], None);

        assert_eq!(views.len(), 1, "got {views:?}");
        assert_eq!(views[0].name, "notes.md");
    }

    #[cfg(windows)]
    #[test]
    fn windows_paths_compare_without_regard_to_case() {
        let chat = tempfile::tempdir().unwrap();
        let notes = file(chat.path(), "notes.md", b"x");
        let folder = chat.path().to_string_lossy().into_owned();

        // The same folder, spelled in another case, is the same folder.
        let views = describe(&[notes], Some(&folder.to_uppercase()));

        assert!(!views[0].outside, "{} under {}", views[0].path, folder);
    }

    #[cfg(windows)]
    #[test]
    fn a_verbatim_folder_and_a_plain_path_are_the_same_directory() {
        let chat = tempfile::tempdir().unwrap();
        let notes = file(chat.path(), "notes.md", b"x");
        // What the platform's own canonicalize answers on Windows: the form a
        // caller could hand this rule, and the one it must not leak onward.
        let verbatim = std::fs::canonicalize(chat.path()).unwrap();
        assert!(
            verbatim.to_string_lossy().starts_with(r"\\?\"),
            "no verbatim form to compare: {}",
            verbatim.display()
        );
        let folder = verbatim.to_string_lossy().into_owned();

        let views = describe(&[notes], Some(&folder));

        assert!(!views[0].outside, "{} under {}", views[0].path, folder);
        assert!(!views[0].path.starts_with(r"\\?\"), "got {}", views[0].path);
    }
}
