//! `refresh_paths`: the same per-file rules as `refresh`, for the paths a file
//! watcher reports instead of a directory listing.

use super::test_home::{append, bump_mtime, header, messages, user, Home};
use super::*;

#[test]
fn a_new_file_is_added() {
    let home = Home::new();
    let mut index = home.index();
    let path = home.write("s", &[header("s"), user("hola")]);
    let stats = index.refresh_paths(std::slice::from_ref(&path));
    assert_eq!(
        (stats.files, stats.rescanned, stats.appended, stats.removed),
        (1, 1, 0, 0)
    );
    assert!(stats.bytes_read > 0);
    assert_eq!(messages(&index, &path), Some(1));
}

#[test]
fn a_grown_file_only_has_its_appended_bytes_read() {
    let home = Home::new();
    let path = home.write("s", &[header("s"), user("hola")]);
    let mut index = home.index();
    index.refresh().unwrap();

    let written = append(&path, &user("segundo"));
    let stats = index.refresh_paths(std::slice::from_ref(&path));
    assert_eq!((stats.rescanned, stats.appended), (0, 1));
    assert_eq!(stats.bytes_read, written as u64);
    assert_eq!(messages(&index, &path), Some(2));
}

#[test]
fn an_unchanged_file_is_not_read() {
    let home = Home::new();
    let path = home.write("s", &[header("s"), user("hola")]);
    let mut index = home.index();
    index.refresh().unwrap();

    let stats = index.refresh_paths(&[path]);
    assert_eq!(
        stats,
        RefreshStats {
            files: 1,
            ..RefreshStats::default()
        }
    );
}

#[test]
fn a_same_size_rewrite_with_a_new_mtime_is_rescanned() {
    let home = Home::new();
    let path = home.write("s", &[header("s"), user("aaaa")]);
    let mut index = home.index();
    index.refresh().unwrap();

    home.write("s", &[header("s"), user("bbbb")]);
    bump_mtime(&path, 5);
    let stats = index.refresh_paths(std::slice::from_ref(&path));
    assert_eq!((stats.rescanned, stats.appended), (1, 0));
    assert_eq!(
        index.entries()[0].summary.first_message.as_deref(),
        Some("bbbb")
    );
}

#[test]
fn a_shrunk_file_is_rescanned_from_the_start() {
    let home = Home::new();
    let path = home.write("s", &[header("s"), user("uno"), user("dos")]);
    let mut index = home.index();
    index.refresh().unwrap();

    home.write("s", &[header("s")]);
    let stats = index.refresh_paths(std::slice::from_ref(&path));
    assert_eq!((stats.rescanned, stats.appended), (1, 0));
    assert_eq!(messages(&index, &path), Some(0));
}

#[test]
fn a_deleted_file_is_dropped() {
    let home = Home::new();
    let path = home.write("s", &[header("s")]);
    let mut index = home.index();
    index.refresh().unwrap();

    fs::remove_file(&path).unwrap();
    let stats = index.refresh_paths(&[path]);
    assert_eq!(
        stats,
        RefreshStats {
            files: 1,
            removed: 1,
            ..RefreshStats::default()
        }
    );
    assert!(index.entries().is_empty());
}

#[test]
fn a_missing_file_that_was_never_cached_is_a_no_op() {
    let home = Home::new();
    let path = home.write("s", &[header("s")]);
    let mut index = home.index();
    fs::remove_file(&path).unwrap();

    let stats = index.refresh_paths(&[path]);
    assert_eq!(
        stats,
        RefreshStats {
            files: 1,
            ..RefreshStats::default()
        }
    );
    assert!(index.entries().is_empty());
}

#[test]
fn a_path_reported_twice_is_processed_once() {
    let home = Home::new();
    let path = home.write("s", &[header("s")]);
    let mut index = home.index();
    index.refresh().unwrap();

    let written = append(&path, &user("hola"));
    let stats = index.refresh_paths(&[path.clone(), path.clone(), path.clone()]);
    assert_eq!((stats.files, stats.appended), (1, 1));
    assert_eq!(stats.bytes_read, written as u64);
    assert_eq!(messages(&index, &path), Some(1));
}

#[test]
fn a_restored_index_reads_only_the_appended_bytes_by_path() {
    let home = Home::new();
    let path = home.write("s", &[header("s"), user("hola")]);
    let mut index = home.index();
    index.refresh().unwrap();

    let written = append(&path, &user("segundo"));
    let mut fresh = SessionIndex::new(home.sessions());
    fresh.restore(index.snapshot());
    let stats = fresh.refresh_paths(std::slice::from_ref(&path));
    assert_eq!((stats.rescanned, stats.appended), (0, 1));
    assert_eq!(stats.bytes_read, written as u64);
    assert_eq!(messages(&fresh, &path), Some(2));
}

#[cfg(windows)]
#[test]
fn windows_updates_an_entry_from_a_verbatim_path() {
    let home = Home::new();
    let path = home.write("s", &[header("s")]);
    let mut index = home.index();
    index.refresh().unwrap();

    append(&path, &user("hola"));
    let verbatim = PathBuf::from(format!(r"\\?\{}", path.display()));
    let stats = index.refresh_paths(&[verbatim]);
    assert_eq!((stats.files, stats.appended), (1, 1));
    assert_eq!(messages(&index, &path), Some(1));
}
