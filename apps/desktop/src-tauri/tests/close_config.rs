//! Windowless tests of the config writer's close step (T8a): the queue is
//! drained and the thread finishes before the process goes away, so a snapshot
//! queued when the window closes still reaches the file.
//!
//! A temp directory holds the file; the user's own config is never touched.

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use gentle_desk_lib::config::ConfigWriter;
use gentle_runtime::app_config::{self, AppConfig};

/// How long a test waits for the writer, which has no deadline of its own here.
const LIMIT: Duration = Duration::from_secs(10);

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// A config naming one workspace, so two snapshots are distinguishable.
fn config(path: &str) -> AppConfig {
    let mut config = AppConfig::default();
    assert!(config.add_workspace(path));
    config
}

/// Waits until the file holds the config of `path`, failing instead of hanging.
fn wait_for_file(file: &Path, path: &str) {
    let deadline = Instant::now() + LIMIT;
    loop {
        let loaded = app_config::load(file);
        if loaded.config == config(path) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "the config file never held {path}: {:?}",
            loaded.config
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn a_flush_writes_the_last_of_several_queued_snapshots() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("config.json");
    let writer = ConfigWriter::spawn(Some(file.clone()));

    // Three snapshots as fast as a burst of clicks queues them, then the close:
    // the file must hold the newest one, not the first the thread picked up.
    writer.save(1, &config("C:\\one"));
    writer.save(2, &config("C:\\two"));
    writer.save(3, &config("C:\\three"));
    assert!(
        writer.flush_and_stop(Duration::from_secs(5)),
        "the writer drained and finished within its deadline"
    );

    let loaded = app_config::load(&file);
    assert_eq!(loaded.notice, None, "the file was written, not damaged");
    assert_eq!(
        loaded.config,
        config("C:\\three"),
        "the close wrote the latest snapshot"
    );
}

#[test]
fn a_flush_with_nothing_to_write_returns_at_once() {
    // A run with no config path has no queue and no thread: the close step is a
    // no-op, never a wait for a writer that does not exist.
    let writer = ConfigWriter::spawn(None);

    let started = Instant::now();
    assert!(writer.flush_and_stop(Duration::from_secs(5)));
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "a writer with no path waited"
    );
}

#[test]
fn a_save_still_reaches_the_file_without_a_flush() {
    // The ordinary path is unchanged: the thread writes a queued snapshot while
    // the window runs, and the close's flush only covers what is left.
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("config.json");
    let writer = ConfigWriter::spawn(Some(file.clone()));

    writer.save(1, &config("C:\\repo"));
    wait_for_file(&file, "C:\\repo");

    // A snapshot older than what the file holds is still dropped instead of
    // undoing it (the writer's version rule), and a late save after the close
    // does not panic: the queue is simply gone.
    writer.save(0, &config("C:\\old"));
    assert!(writer.flush_and_stop(Duration::from_secs(5)));
    assert_eq!(
        app_config::load(&file).config,
        config("C:\\repo"),
        "an older snapshot never overwrites a newer one"
    );
    writer.save(9, &config("C:\\late"));
}

#[test]
fn a_save_that_cannot_be_written_is_reported_and_does_not_block_the_close() {
    let dir = tempfile::tempdir().unwrap();
    // A file where the config's directory should be: the write cannot win.
    let blocker = dir.path().join("not-a-dir");
    std::fs::write(&blocker, "x").unwrap();
    let file = blocker.join("config.json");
    let writer = ConfigWriter::spawn(Some(file));

    let seen: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    writer.on_failure(Arc::new({
        let seen = Arc::clone(&seen);
        move |message| lock(&seen).push(message)
    }));
    writer.save(1, &config("C:\repo"));

    // The failure does not keep the window open: the thread still finishes, and
    // the reason reached the handler instead of vanishing.
    let started = Instant::now();
    assert!(
        writer.flush_and_stop(Duration::from_secs(5)),
        "the writer finished even though the file could not be written"
    );
    assert!(started.elapsed() < Duration::from_secs(5));

    let seen = lock(&seen).clone();
    assert_eq!(seen.len(), 1, "one failure, one reason: {seen:?}");
    assert!(
        seen[0].starts_with("The app config file could not be saved:"),
        "got {:?}",
        seen[0]
    );
}
