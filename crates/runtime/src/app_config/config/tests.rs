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
fn removing_a_workspace_forgets_it_and_keeps_the_others_in_the_order_they_were_added() {
    let mut config = AppConfig::default();
    assert!(config.add_workspace(r"C:\dev\p"));
    assert!(config.add_workspace(r"C:\dev\q"));
    assert!(config.add_workspace(r"C:\dev\r"));

    assert!(config.remove_workspace(r"C:\dev\q"));

    assert_eq!(
        config.workspaces,
        [
            WorkspaceEntry {
                path: r"C:\dev\p".to_owned(),
                collapsed: false,
            },
            WorkspaceEntry {
                path: r"C:\dev\r".to_owned(),
                collapsed: false,
            },
        ],
        "the folders the config keeps are untouched and still in order"
    );
    assert_eq!(config.workspace(r"C:\dev\q"), None);
}

#[test]
fn removing_a_workspace_is_reached_through_another_spelling_of_the_folder() {
    let mut config = AppConfig::default();
    assert!(config.add_workspace(r"C:\dev\p"));

    assert!(config.remove_workspace(r"\\?\C:\Dev\P\"));

    assert!(
        config.workspaces.is_empty(),
        "the one spelling is one folder"
    );
}

#[test]
fn removing_a_workspace_the_config_does_not_hold_changes_nothing() {
    let mut config = AppConfig::default();
    assert!(config.add_workspace(r"C:\dev\p"));

    assert!(!config.remove_workspace(r"C:\dev\other"));
    assert!(!config.remove_workspace(""));
    assert!(!config.remove_workspace("   "));

    assert_eq!(config.workspaces.len(), 1);
    assert_eq!(config.workspaces[0].path, r"C:\dev\p");
}

#[test]
fn the_fold_of_the_workspaces_it_keeps_survives_another_one_being_removed() {
    let mut config = AppConfig::default();
    assert!(config.add_workspace(r"C:\dev\p"));
    assert!(config.add_workspace(r"C:\dev\q"));
    assert!(config.set_collapsed(r"C:\dev\p", true));

    assert!(config.remove_workspace(r"C:\dev\q"));

    assert_eq!(
        config.workspaces,
        [WorkspaceEntry {
            path: r"C:\dev\p".to_owned(),
            collapsed: true,
        }]
    );
}

#[test]
fn a_workspace_removed_can_be_added_again_expanded() {
    let mut config = AppConfig::default();
    assert!(config.add_workspace(r"C:\dev\p"));
    assert!(config.set_collapsed(r"C:\dev\p", true));
    assert!(config.remove_workspace(r"C:\dev\p"));

    assert!(config.add_workspace(r"C:\dev\p"));

    assert_eq!(
        config.workspaces,
        [WorkspaceEntry {
            path: r"C:\dev\p".to_owned(),
            collapsed: false,
        }],
        "adding a folder the config does not hold still stores it expanded"
    );
}

#[test]
fn the_canonical_spelling_is_what_a_workspace_is_stored_with() {
    assert_eq!(canonical_path(r"\\?\C:\dev\p\"), r"C:\dev\p");
    assert_eq!(canonical_path(r"\\?\UNC\srv\share"), r"\\srv\share");
    assert_eq!(canonical_path(r"C:\"), r"C:\");
    assert_eq!(canonical_path("/"), "/");
}

#[test]
fn a_default_config_has_the_system_theme() {
    assert_eq!(AppConfig::default().theme, Theme::System);
}

#[test]
fn setting_the_theme_reports_whether_it_changed() {
    let mut config = AppConfig::default();
    assert!(config.set_theme(Theme::Gentle));
    assert_eq!(config.theme, Theme::Gentle);
    assert!(
        !config.set_theme(Theme::Gentle),
        "the same theme changes nothing, so nothing is saved"
    );
    assert!(config.set_theme(Theme::System));
    assert_eq!(config.theme, Theme::System);
}

#[test]
fn the_six_themes_are_named_the_way_the_stylesheet_names_them() {
    // The picker order, and the six `data-theme` values of
    // `apps/desktop/src/theme.css`, are the same six names.
    let names: Vec<&str> = Theme::ALL.iter().map(|theme| theme.as_str()).collect();
    assert_eq!(
        names,
        [
            "system",
            "dark",
            "Gentle",
            "Gentleman-Cute",
            "Gentleman-Sexy",
            "light"
        ]
    );
    for theme in Theme::ALL {
        assert_eq!(Theme::from_name(theme.as_str()), Some(theme));
    }
    assert_eq!(Theme::from_name("Neon"), None);
    assert_eq!(Theme::from_name("gentle"), None, "the names are exact");
}

#[test]
fn a_default_config_notifies_when_the_window_is_not_in_front() {
    // The maintainer's proposal (notifications S4, L3): on, and only while the
    // window is not in front, until they choose otherwise.
    assert_eq!(
        AppConfig::default().notifications,
        Notifications::WhenUnfocused
    );
}

#[test]
fn setting_the_notifications_mode_reports_whether_it_changed() {
    let mut config = AppConfig::default();
    assert!(config.set_notifications(Notifications::Off));
    assert_eq!(config.notifications, Notifications::Off);
    assert!(
        !config.set_notifications(Notifications::Off),
        "the same mode changes nothing, so nothing is saved"
    );
    assert!(config.set_notifications(Notifications::WhenUnfocused));
    assert_eq!(config.notifications, Notifications::WhenUnfocused);
}

#[test]
fn the_two_notification_modes_are_named_the_way_the_file_stores_them() {
    let names: Vec<&str> = Notifications::ALL
        .iter()
        .map(|mode| mode.as_str())
        .collect();
    assert_eq!(names, ["when-unfocused", "off"]);
    for mode in Notifications::ALL {
        assert_eq!(Notifications::from_name(mode.as_str()), Some(mode));
    }
    assert_eq!(
        Notifications::from_name("always"),
        None,
        "the names are exact"
    );
    assert_eq!(Notifications::from_name("WhenUnfocused"), None);
}

#[test]
fn a_default_config_skips_no_version() {
    assert_eq!(AppConfig::default().skipped_update, None);
}

#[test]
fn skipping_a_version_reports_whether_it_changed_and_keeps_one_skip() {
    let mut config = AppConfig::default();
    assert!(config.set_skipped_update(" 2.0.5 "));
    assert_eq!(
        config.skipped_update.as_deref(),
        Some("2.0.5"),
        "the stored version is trimmed, so it compares as the one the check reported"
    );
    assert!(
        !config.set_skipped_update("2.0.5"),
        "the same skip again changes nothing, so nothing is saved"
    );
    assert!(
        config.set_skipped_update("2.0.6"),
        "one skip at a time: a newer skip replaces the one before it"
    );
    assert_eq!(config.skipped_update.as_deref(), Some("2.0.6"));
}

#[test]
fn a_version_that_is_not_a_strict_xyz_stores_no_skip() {
    // The comparison the pane and the check make is `x.y.z`'s own
    // (gentle_runtime::updates::parse): a value it cannot compare is no skip at
    // all, so it never hides an update.
    let mut config = AppConfig::default();
    for bad in [
        "",
        "   ",
        "2",
        "2.0",
        "2.0.5.1",
        "v2.0.5",
        "2.0.5-beta.1",
        "latest",
    ] {
        assert!(!config.set_skipped_update(bad), "{bad:?} was stored");
        assert_eq!(config.skipped_update, None, "{bad:?} was stored");
    }
}

#[test]
fn stopping_the_skip_answers_whether_anything_was_skipped() {
    let mut config = AppConfig::default();
    assert!(!config.clear_skipped_update(), "nothing was skipped");
    assert!(config.set_skipped_update("2.0.5"));
    assert!(config.clear_skipped_update());
    assert_eq!(config.skipped_update, None);
    assert!(
        !config.clear_skipped_update(),
        "a second stop changes nothing, so nothing is saved"
    );
}
