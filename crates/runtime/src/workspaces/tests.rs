use gentle_app::SessionItem;

use super::*;
use crate::app_config::{folder_key, AppConfig};

mod labels;
mod order;
mod sweep;

/// A session header reporting `cwd`, last written at `modified_ms`.
fn session(cwd: &str, modified_ms: u64) -> SessionItem {
    SessionItem {
        path: format!("session-{modified_ms}.jsonl"),
        title: String::new(),
        cwd: cwd.to_owned(),
        modified_ms,
        message_count: 0,
    }
}

/// A config holding `paths` as workspaces, in that order, all expanded.
fn config(paths: &[&str]) -> AppConfig {
    let mut config = AppConfig::default();
    for path in paths {
        assert!(config.add_workspace(path), "add_workspace({path})");
    }
    config
}

/// The paths of the rows, in the order they are shown.
fn paths(list: &WorkspaceList) -> Vec<&str> {
    list.workspaces
        .iter()
        .map(|workspace| workspace.path.as_str())
        .collect()
}

/// The row of `path`, whatever spelling the row shows it with.
fn workspace<'a>(list: &'a WorkspaceList, path: &str) -> &'a Workspace {
    let key = folder_key(path);
    list.workspaces
        .iter()
        .find(|workspace| workspace.key == key)
        .unwrap_or_else(|| panic!("no workspace for {path}"))
}

#[test]
fn a_folder_the_config_holds_and_a_session_reports_is_one_workspace() {
    let config = config(&[r"C:\dev\gentle"]);
    let sessions = [
        session(r"\\?\C:\dev\gentle\", 100),
        session(r"C:\dev\gentle", 200),
    ];

    let list = build(&config, &sessions);

    assert_eq!(
        paths(&list),
        [r"C:\dev\gentle"],
        "the config spelling stays"
    );
    let workspace = &list.workspaces[0];
    assert_eq!(workspace.key, folder_key(r"C:\dev\gentle"));
    assert!(workspace.from_config);
    assert!(!workspace.collapsed);
    assert_eq!(workspace.name, "gentle");
    assert_eq!(workspace.label, "gentle");
    assert_eq!(workspace.last_activity_ms, Some(200));
    assert_eq!(workspace.sessions, [0, 1], "input order, newest first");
    assert_eq!(list.unplaced, 0);
}

#[cfg(windows)]
#[test]
fn the_same_folder_in_another_letter_case_is_one_workspace() {
    let config = config(&[r"C:\Dev\Gentle"]);
    let sessions = [session(r"c:\DEV\gentle\", 100)];

    let list = build(&config, &sessions);

    assert_eq!(paths(&list), [r"C:\Dev\Gentle"]);
    assert_eq!(list.workspaces[0].sessions, [0]);
}

#[test]
fn a_folder_with_no_chat_is_listed_and_keeps_its_collapsed_state() {
    let mut config = config(&[r"C:\empty"]);
    assert!(config.set_collapsed(r"C:\empty", true));

    let list = build(&config, &[]);

    assert_eq!(paths(&list), [r"C:\empty"]);
    let workspace = &list.workspaces[0];
    assert!(workspace.from_config);
    assert!(workspace.collapsed);
    assert_eq!(workspace.last_activity_ms, None);
    assert!(workspace.sessions.is_empty());
    assert_eq!(workspace.name, "empty");
    assert_eq!(workspace.label, "empty");
    assert_eq!(list.unplaced, 0);
}

#[test]
fn a_folder_only_the_sessions_report_is_a_workspace_that_is_never_collapsed() {
    let config = AppConfig::default();
    let sessions = [session(r"\\?\C:\work\api\", 300)];

    let list = build(&config, &sessions);

    assert_eq!(paths(&list), [r"C:\work\api"], "the canonical spelling");
    let workspace = &list.workspaces[0];
    assert!(!workspace.from_config);
    assert!(!workspace.collapsed);
    assert_eq!(workspace.last_activity_ms, Some(300));
    assert_eq!(workspace.sessions, [0]);
    assert_eq!(workspace.name, "api");
    assert_eq!(workspace.label, "api");
}

#[test]
fn a_session_with_a_blank_cwd_belongs_to_no_workspace() {
    let config = AppConfig::default();
    let sessions = [
        session("", 100),
        session("   ", 200),
        session(r"C:\dev\p", 300),
    ];

    let list = build(&config, &sessions);

    assert_eq!(list.unplaced, 2);
    assert_eq!(paths(&list), [r"C:\dev\p"], "no workspace is invented");
    assert_eq!(list.workspaces[0].sessions, [2]);
}

#[test]
fn the_activity_of_a_workspace_is_its_newest_session_and_its_sessions_keep_input_order() {
    let config = config(&[r"C:\p"]);
    let sessions = [
        session(r"C:\p", 300),
        session(r"C:\other", 500),
        session(r"C:\p", 100),
        session(r"C:\p\", 200),
    ];

    let list = build(&config, &sessions);

    assert_eq!(
        paths(&list),
        [r"C:\other", r"C:\p"],
        "newest activity first"
    );
    let p = workspace(&list, r"C:\p");
    assert_eq!(p.sessions, [0, 2, 3]);
    assert_eq!(p.last_activity_ms, Some(300));
}

#[test]
fn no_config_and_no_sessions_is_an_empty_list() {
    let list = build(&AppConfig::default(), &[]);

    assert_eq!(list, WorkspaceList::default());
    assert!(list.workspaces.is_empty());
    assert_eq!(list.unplaced, 0);
}
