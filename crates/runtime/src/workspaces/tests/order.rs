use super::*;

#[test]
fn workspaces_are_ordered_by_their_newest_session() {
    let config = AppConfig::default();
    let sessions = [
        session(r"C:\b\one", 100),
        session(r"C:\a\two", 900),
        session(r"C:\c\three", 500),
    ];

    let list = build(&config, &sessions);

    assert_eq!(paths(&list), [r"C:\a\two", r"C:\c\three", r"C:\b\one"]);
}

#[test]
fn equal_activity_is_ordered_by_key() {
    let config = AppConfig::default();
    let sessions = [
        session(r"C:\b\app", 100),
        session(r"C:\a\app", 100),
        session(r"C:\c\app", 100),
    ];

    let list = build(&config, &sessions);

    assert_eq!(paths(&list), [r"C:\a\app", r"C:\b\app", r"C:\c\app"]);
}

#[test]
fn a_workspace_with_activity_is_placed_by_it_not_by_config_order() {
    let config = config(&[r"C:\old", r"C:\new"]);
    let sessions = [session(r"C:\old", 100), session(r"C:\new", 900)];

    let list = build(&config, &sessions);

    assert_eq!(paths(&list), [r"C:\new", r"C:\old"]);
}

#[test]
fn workspaces_with_no_chat_come_last_in_config_order() {
    let config = config(&[r"C:\one", r"C:\two", r"C:\three"]);
    let sessions = [
        session(r"C:\two", 100),
        session(r"C:\derived", 500),
        session(r"C:\two", 50),
    ];

    let list = build(&config, &sessions);

    assert_eq!(
        paths(&list),
        [r"C:\derived", r"C:\two", r"C:\one", r"C:\three"],
        "a folder with no chat has no activity to be placed by"
    );
}
