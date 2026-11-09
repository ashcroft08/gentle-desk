//! Link-level proof of the rfd folder picker (Q4, W3c).
//!
//! The Tauri dialog plugin was dropped because a test binary that linked it
//! failed to load with `STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139)`: it imports
//! comctl32 v6's `TaskDialogIndirect` and a test exe carries no application
//! manifest. This test binary is the check instead: it names the real picker,
//! so its `pick_folder` is linked, and it must simply START and pass.
//!
//! It never opens a dialog: the trait object is built and handed to a seam that
//! stores it, which is enough for the linker to keep the real implementation.
//! The same binary is what `objdump -p` is read for the comctl32 imports.

use std::path::PathBuf;
use std::sync::Arc;

use gentle_desk_lib::picker::{FolderPicker, NativeFolderPicker};

/// A fake picker, so the real one is never asked to open a window here.
struct Fixed(Option<PathBuf>);

impl FolderPicker for Fixed {
    fn pick_folder(&self, _title: &str) -> Option<PathBuf> {
        self.0.clone()
    }
}

/// The seam a workspace flow takes: a trait object, whatever built it.
fn as_seam(picker: Arc<dyn FolderPicker>) -> Arc<dyn FolderPicker> {
    picker
}

#[test]
fn the_real_folder_picker_links_into_a_test_binary() {
    // The real picker, built and handed over as the seam the flows take: its
    // `pick_folder`, and with it rfd, is linked into this executable.
    let real = as_seam(Arc::new(NativeFolderPicker::new()));
    // Never called: the dialog needs a human. What is proven is that this binary
    // loads and runs with rfd linked.
    assert!(Arc::strong_count(&real) >= 1);

    // The seam answers a fixed folder for a test, exactly as the flows use it.
    let fake = as_seam(Arc::new(Fixed(Some(PathBuf::from("C:\\dev\\repo")))));
    assert_eq!(
        fake.pick_folder("Choose a workspace folder"),
        Some(PathBuf::from("C:\\dev\\repo"))
    );
}
