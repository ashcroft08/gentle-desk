use super::*;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const DAMAGED: &[u8] = b"{ not json";
const DAMAGED_AGAIN: &[u8] = b"still not json";

fn config_file(dir: &Path) -> PathBuf {
    dir.join("gentle-desk").join("config.json")
}

/// The names in `dir`, sorted, so a test can say exactly what is left behind.
fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn write_damaged(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

#[test]
fn a_missing_file_is_an_empty_config_without_a_notice() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());

    let loaded = load(&path);

    assert_eq!(loaded.config, AppConfig::default());
    assert_eq!(loaded.notice, None);
    assert!(names(dir.path()).is_empty(), "nothing is written by a load");
}

#[test]
fn save_then_load_round_trips_the_workspaces_and_the_collapse_state() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    let mut config = AppConfig::default();
    config.add_workspace(r"C:\dev\p");
    config.add_workspace(r"C:\dev\q");
    config.set_collapsed(r"C:\dev\q", true);

    save(&path, &config).unwrap();

    let loaded = load(&path);
    assert_eq!(loaded.config, config);
    assert_eq!(loaded.notice, None);
}

#[test]
fn the_saved_document_carries_the_top_level_version_and_the_workspaces() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    let mut config = AppConfig::default();
    config.add_workspace(r"C:\dev\p");
    config.set_collapsed(r"C:\dev\p", true);

    save(&path, &config).unwrap();

    let document: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(document["version"], serde_json::json!(CONFIG_VERSION));
    assert_eq!(
        document["workspaces"],
        serde_json::json!([{ "path": r"C:\dev\p", "collapsed": true }])
    );
}

#[test]
fn save_creates_the_directory_and_leaves_no_temp_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());

    save(&path, &AppConfig::default()).unwrap();

    assert_eq!(names(path.parent().unwrap()), ["config.json"]);
    assert_eq!(load(&path).config, AppConfig::default());
}

#[test]
fn a_failed_rename_leaves_the_previous_entry_and_no_temp_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    // A directory cannot be replaced by a file rename, so the last step of the
    // write fails with the old entry untouched.
    fs::create_dir_all(&path).unwrap();
    fs::write(path.join("keep.txt"), b"old").unwrap();

    assert!(save(&path, &AppConfig::default()).is_err());

    assert_eq!(fs::read(path.join("keep.txt")).unwrap(), b"old");
    assert_eq!(names(path.parent().unwrap()), ["config.json"]);
}

#[test]
fn a_save_that_cannot_write_keeps_the_previous_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    let mut first = AppConfig::default();
    first.add_workspace(r"C:\dev\p");
    save(&path, &first).unwrap();
    // A directory where the temporary file goes makes the write fail.
    fs::create_dir_all(path.with_file_name("config.json.tmp")).unwrap();
    let mut second = AppConfig::default();
    second.add_workspace(r"C:\dev\q");

    assert!(save(&path, &second).is_err());

    assert_eq!(load(&path).config, first);
}

#[test]
fn a_damaged_file_is_kept_aside_with_its_bytes_and_an_empty_config_comes_back() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    write_damaged(&path, DAMAGED);

    let loaded = load(&path);

    assert_eq!(loaded.config, AppConfig::default());
    let notice = loaded.notice.unwrap();
    assert_eq!(notice.problem, ConfigProblem::Damaged);
    let aside = notice.aside.clone().unwrap();
    assert_ne!(aside, path);
    assert!(!path.exists(), "the damaged file is not left in place");
    assert_eq!(fs::read(&aside).unwrap(), DAMAGED);
    let name = aside.file_name().unwrap().to_string_lossy().into_owned();
    assert!(name.starts_with("config.json.corrupt-"), "{name}");

    let message = notice.message();
    assert!(message.is_ascii(), "{message}");
    assert!(message.contains(&name), "{message}");
}

#[test]
fn a_second_damaged_file_does_not_overwrite_the_first_aside_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    write_damaged(&path, DAMAGED);
    let first = load(&path).notice.unwrap().aside.unwrap();
    write_damaged(&path, DAMAGED_AGAIN);

    let second = load(&path).notice.unwrap().aside.unwrap();

    assert_ne!(first, second);
    assert_eq!(fs::read(&first).unwrap(), DAMAGED);
    assert_eq!(fs::read(&second).unwrap(), DAMAGED_AGAIN);
}

#[test]
fn the_aside_name_counts_up_when_the_second_is_already_taken() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    // Take the plain aside names of the next few seconds, so whichever second
    // the load runs in, the plain name is taken and the name must count up.
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let earlier: Vec<PathBuf> = (0..5)
        .map(|offset| path.with_file_name(format!("config.json.corrupt-{}", seconds + offset)))
        .collect();
    for aside in &earlier {
        fs::write(aside, b"earlier").unwrap();
    }
    write_damaged(&path, DAMAGED);

    let aside = load(&path).notice.unwrap().aside.unwrap();

    let name = aside.file_name().unwrap().to_string_lossy().into_owned();
    assert!(name.starts_with("config.json.corrupt-"), "{name}");
    assert!(name.ends_with("-2"), "{name}");
    assert_eq!(fs::read(&aside).unwrap(), DAMAGED);
    for untouched in &earlier {
        assert_eq!(fs::read(untouched).unwrap(), b"earlier", "{untouched:?}");
    }
}

#[test]
fn a_file_of_a_version_this_app_does_not_write_is_kept_aside() {
    let dir = tempfile::tempdir().unwrap();
    let path = config_file(dir.path());
    let document = br#"{"version":2,"workspaces":[]}"#;
    write_damaged(&path, document);

    let loaded = load(&path);

    assert_eq!(loaded.config, AppConfig::default());
    let notice = loaded.notice.unwrap();
    assert_eq!(notice.problem, ConfigProblem::UnknownVersion { found: 2 });
    let message = notice.message();
    let aside = notice.aside.unwrap();
    assert!(!path.exists());
    assert_eq!(fs::read(&aside).unwrap(), document);
    assert!(message.contains("2"), "{message}");
    assert!(message.is_ascii(), "{message}");
}
