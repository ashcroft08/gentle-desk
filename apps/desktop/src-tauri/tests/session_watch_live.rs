//! The one process-level test of the watcher (T8b): the real `notify` watcher on
//! a temp pi home, the real passes, and the list the run pushes.
//!
//! A session file the terminal writes while the app runs must appear in the
//! sidebar by itself, and deleting it must take it away again. Every wait polls
//! with the live deadline and never sleeps a fixed time: Windows/NTFS hands the
//! notice of a change over seconds late (about 3 s on a developer machine, over
//! 10 s on a CI runner, see `gentle_backend::session_watch`), so a short fixed
//! wait would be the flakiest thing here.

mod session_seam;
mod watch_seam;

use std::fs;

use session_seam::Home;
use watch_seam::{session_file, wired_real};

/// The deadline a close gives its thread stops here.
const CLOSE: std::time::Duration = std::time::Duration::from_secs(5);

#[test]
fn a_session_file_the_terminal_writes_appears_without_a_restart() {
    let home = Home::new();
    // The watcher needs the directory to exist: a home whose sessions folder is
    // not there yet is the retry rule of `session_watch_wiring.rs`.
    fs::create_dir_all(home.sessions()).expect("the sessions directory");
    let wired = wired_real(&home);
    wired.service.boot();
    // The first pass settles the empty list, and the real watcher is up.
    wired.wait_total(0);

    home.session("p", "terminal.jsonl", "written by the terminal");
    wired.wait_total(1);

    fs::remove_file(session_file(&home, "p", "terminal.jsonl")).expect("the session file");
    wired.wait_total(0);

    assert!(wired.service.stop(CLOSE), "the watcher thread ended");
}
