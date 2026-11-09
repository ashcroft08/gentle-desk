use super::*;

#[test]
fn a_default_config_is_version_one_with_no_workspaces() {
    let config = AppConfig::default();
    assert_eq!(config.version, CONFIG_VERSION);
    assert_eq!(CONFIG_VERSION, 1);
    assert!(config.workspaces.is_empty());
}

#[test]
fn adding_a_workspace_keeps_its_path_and_starts_expanded() {
    let mut config = AppConfig::default();
    assert!(config.add_workspace(r"C:\dev\gentle-desk"));
    assert_eq!(
        config.workspaces,
        [WorkspaceEntry {
            path: r"C:\dev\gentle-desk".to_owned(),
            collapsed: false,
        }]
    );
    assert_eq!(
        config.workspace(r"C:\dev\gentle-desk"),
        Some(&WorkspaceEntry {
            path: r"C:\dev\gentle-desk".to_owned(),
            collapsed: false,
        })
    );
}

#[test]
fn adding_the_same_folder_twice_is_one_workspace() {
    let mut config = AppConfig::default();
    assert!(config.add_workspace(r"C:\dev\p"));
    assert!(!config.add_workspace(r"C:\dev\p"));
    assert_eq!(config.workspaces.len(), 1);
    assert_eq!(config.workspaces[0].path, r"C:\dev\p");
}

#[test]
fn an_empty_path_adds_no_workspace() {
    let mut config = AppConfig::default();
    assert!(!config.add_workspace(""));
    assert!(!config.add_workspace("   "));
    assert!(config.workspaces.is_empty());
}

#[test]
fn the_verbatim_prefix_and_a_trailing_separator_name_the_same_folder() {
    assert_eq!(folder_key(r"\\?\C:\dev\p\"), folder_key(r"C:\dev\p"));
    assert_eq!(folder_key(r"\\?\UNC\srv\share"), folder_key(r"\\srv\share"));
    assert_eq!(folder_key("/home/ana/p/"), folder_key("/home/ana/p"));

    let mut config = AppConfig::default();
    assert!(config.add_workspace(r"C:\dev\p"));
    assert!(!config.add_workspace(r"\\?\C:\dev\p\"));
    assert_eq!(
        config.workspaces,
        [WorkspaceEntry {
            path: r"C:\dev\p".to_owned(),
            collapsed: false,
        }],
        "the first spelling is the one kept"
    );
}

#[test]
fn a_folder_whose_name_starts_with_unc_is_left_alone() {
    // Only the verbatim form spells a share as `UNC\...`: a folder that merely
    // starts with those letters keeps its name, separators aside.
    let mut config = AppConfig::default();
    assert!(config.add_workspace(r"UNCLE\proj"));
    assert!(config.add_workspace(r"unc\x\"));
    assert_eq!(
        config.workspaces,
        [
            WorkspaceEntry {
                path: r"UNCLE\proj".to_owned(),
                collapsed: false,
            },
            WorkspaceEntry {
                path: r"unc\x".to_owned(),
                collapsed: false,
            },
        ]
    );
}

#[test]
fn a_verbatim_folder_whose_name_starts_with_unc_loses_only_the_prefix() {
    let mut config = AppConfig::default();
    assert!(config.add_workspace(r"\\?\UNCLE\x"));
    assert_eq!(config.workspaces[0].path, r"UNCLE\x");
}

#[test]
fn a_workspace_added_with_the_verbatim_prefix_is_stored_without_it() {
    let mut config = AppConfig::default();
    assert!(config.add_workspace(r"\\?\C:\dev\p"));
    assert_eq!(config.workspaces[0].path, r"C:\dev\p");
}

#[test]
fn a_trailing_separator_is_not_stored_but_a_root_keeps_its_own() {
    let mut config = AppConfig::default();
    assert!(config.add_workspace(r"C:\dev\p\"));
    assert_eq!(config.workspaces[0].path, r"C:\dev\p");

    let mut config = AppConfig::default();
    assert!(config.add_workspace(r"C:\"));
    assert_eq!(config.workspaces[0].path, r"C:\");

    let mut config = AppConfig::default();
    assert!(config.add_workspace("/"));
    assert_eq!(config.workspaces[0].path, "/");
}

#[cfg(windows)]
#[test]
fn adding_the_same_folder_in_another_case_is_one_workspace() {
    let mut config = AppConfig::default();
    assert!(config.add_workspace(r"C:\Dev\Gentle"));
    assert!(!config.add_workspace(r"c:\dev\gentle"));
    assert_eq!(config.workspaces.len(), 1);
    assert_eq!(config.workspaces[0].path, r"C:\Dev\Gentle");
}

#[test]
fn the_collapse_state_of_a_known_workspace_can_change() {
    let mut config = AppConfig::default();
    assert!(config.add_workspace(r"C:\dev\p"));
    assert!(config.set_collapsed(r"C:\dev\p", true));
    assert!(config.workspaces[0].collapsed);
    assert!(config.set_collapsed(r"C:\dev\p", false));
    assert!(!config.workspaces[0].collapsed);
}

#[test]
fn the_collapse_state_is_reached_through_another_spelling_of_the_folder() {
    let mut config = AppConfig::default();
    assert!(config.add_workspace(r"C:\dev\p"));
    assert!(config.set_collapsed(r"\\?\C:\dev\p\", true));
    assert!(config.workspaces[0].collapsed);
    assert_eq!(config.workspaces.len(), 1);
}

#[test]
fn collapsing_an_unknown_workspace_changes_nothing() {
    let mut config = AppConfig::default();
    assert!(config.add_workspace(r"C:\dev\p"));
    assert!(!config.set_collapsed(r"C:\dev\other", true));
    assert!(!config.workspaces[0].collapsed);
}

#[test]
fn the_canonical_spelling_is_what_a_workspace_is_stored_with() {
    assert_eq!(canonical_path(r"\\?\C:\dev\p\"), r"C:\dev\p");
    assert_eq!(canonical_path(r"\\?\UNC\srv\share"), r"\\srv\share");
    assert_eq!(canonical_path(r"C:\"), r"C:\");
    assert_eq!(canonical_path("/"), "/");
}
