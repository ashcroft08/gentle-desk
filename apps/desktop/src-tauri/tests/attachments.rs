//! The clip's command (design/composer-dock T4, S3), windowless: a fake file
//! picker answers instead of a human, so what is asserted is what the command
//! hands the page - the paths a pick named, and nothing at all for a cancel.
//!
//! The dialog itself is never opened here. The real picker's own link is pinned
//! by `tests/picker_link.rs`, which builds it beside its fake.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use gentle_desk_lib::attachments::pick;
use gentle_desk_lib::picker::FilePicker;

/// A file picker a test answers with: the files it was told, or a cancel. It
/// records every title the clip asked for, so the words the dialog opens with
/// are checked as well.
struct FakeFilePicker {
    answer: Vec<PathBuf>,
    titles: Mutex<Vec<String>>,
}

impl FakeFilePicker {
    fn picking(paths: &[&str]) -> Self {
        Self {
            answer: paths.iter().map(PathBuf::from).collect(),
            titles: Mutex::new(Vec::new()),
        }
    }

    fn cancelled() -> Self {
        Self {
            answer: Vec::new(),
            titles: Mutex::new(Vec::new()),
        }
    }

    fn calls(&self) -> usize {
        lock(&self.titles).len()
    }
}

impl FilePicker for FakeFilePicker {
    fn pick_files(&self, title: &str) -> Vec<PathBuf> {
        lock(&self.titles).push(title.to_owned());
        self.answer.clone()
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

#[test]
fn a_cancelled_pick_answers_no_path_at_all() {
    let picker = FakeFilePicker::cancelled();

    let picked = pick(&picker, "Attach files");

    assert_eq!(picker.calls(), 1, "the one dialog of the clip");
    assert!(picked.is_empty(), "a cancel attaches nothing, got {picked:?}");
}

#[test]
fn a_pick_answers_the_paths_the_dialog_named() {
    let picker = FakeFilePicker::picking(&["C:\\dev\\repo\\src\\header.tsx", "C:\\dev\\repo\\plan.md"]);

    let picked = pick(&picker, "Attach files");

    assert_eq!(
        picked,
        vec![
            "C:\\dev\\repo\\src\\header.tsx".to_owned(),
            "C:\\dev\\repo\\plan.md".to_owned()
        ]
    );
}
