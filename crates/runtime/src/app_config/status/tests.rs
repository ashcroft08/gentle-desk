use super::*;
use std::fs;

use gentle_core::i18n::{messages, Language};

use crate::app_config::AppConfig;

#[test]
fn an_existing_folder_is_available() {
    let dir = tempfile::tempdir().unwrap();
    let status = workspace_status(dir.path(), Language::En);
    assert_eq!(status, WorkspaceStatus::Available);
    assert!(status.is_available());
    assert_eq!(status.reason(), None);
}

#[test]
fn a_folder_that_is_gone_is_missing_with_a_reason() {
    let dir = tempfile::tempdir().unwrap();
    let gone = dir.path().join("removed");
    let status = workspace_status(&gone, Language::En);
    assert_eq!(
        status,
        WorkspaceStatus::Missing(messages::workspace_folder_missing(Language::En).to_owned())
    );
    assert!(!status.is_available());
    let reason = status.reason().unwrap();
    assert!(reason.is_ascii(), "{reason}");
    assert!(!reason.is_empty());
}

#[test]
fn a_file_is_not_a_folder_with_a_reason() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("notes.txt");
    fs::write(&file, b"x").unwrap();
    let status = workspace_status(&file, Language::En);
    assert_eq!(
        status,
        WorkspaceStatus::NotAFolder(messages::workspace_not_a_folder(Language::En).to_owned())
    );
    assert!(!status.is_available());
    assert_eq!(
        status.reason(),
        Some(messages::workspace_not_a_folder(Language::En))
    );
}

#[test]
fn a_workspace_that_disappeared_stays_in_the_config_and_reports_missing() {
    let dir = tempfile::tempdir().unwrap();
    let folder = dir.path().join("removed");
    fs::create_dir_all(&folder).unwrap();
    let mut config = AppConfig::default();
    assert!(config.add_workspace(&folder.to_string_lossy()));
    assert_eq!(
        workspace_status(&folder, Language::En),
        WorkspaceStatus::Available
    );

    fs::remove_dir_all(&folder).unwrap();

    assert_eq!(
        workspace_status(
            std::path::Path::new(&config.workspaces[0].path),
            Language::En
        ),
        WorkspaceStatus::Missing(messages::workspace_folder_missing(Language::En).to_owned())
    );
    assert_eq!(
        config.workspaces.len(),
        1,
        "a workspace is never removed on its own"
    );
}
