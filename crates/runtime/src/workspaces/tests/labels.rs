use super::*;

#[test]
fn two_workspaces_with_one_name_are_told_apart_by_their_parent() {
    let config = config(&[r"C:\work\app", r"C:\dev\app"]);

    let list = build(&config, &[]);

    let work = workspace(&list, r"C:\work\app");
    let dev = workspace(&list, r"C:\dev\app");
    assert_eq!(work.name, "app");
    assert_eq!(dev.name, "app");
    assert_eq!(work.label, "work/app");
    assert_eq!(dev.label, "dev/app");
}

#[test]
fn a_name_no_other_workspace_shares_stays_plain() {
    let config = config(&[r"C:\work\app", r"C:\dev\solo"]);

    let list = build(&config, &[]);

    assert_eq!(workspace(&list, r"C:\work\app").label, "app");
    assert_eq!(workspace(&list, r"C:\dev\solo").label, "solo");
}

#[test]
fn a_label_is_extended_only_as_far_as_it_takes() {
    let config = AppConfig::default();
    let sessions = [
        session(r"C:\a\x\app", 300),
        session(r"C:\b\x\app", 200),
        session(r"C:\c\y\app", 100),
        session(r"C:\z\solo", 50),
    ];

    let list = build(&config, &sessions);

    // The two `x\app` folders are alike down to their parent, so they take one
    // more segment; `y\app` was already alone at that depth and stops there.
    assert_eq!(workspace(&list, r"C:\a\x\app").label, "a/x/app");
    assert_eq!(workspace(&list, r"C:\b\x\app").label, "b/x/app");
    assert_eq!(workspace(&list, r"C:\c\y\app").label, "y/app");
    assert_eq!(workspace(&list, r"C:\z\solo").label, "solo");
    let labels: Vec<&str> = list
        .workspaces
        .iter()
        .map(|workspace| workspace.label.as_str())
        .collect();
    assert_eq!(labels.len(), 4);
    let mut sorted = labels.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), 4, "labels are unique: {labels:?}");
}

#[test]
fn a_root_is_named_by_its_own_text() {
    let config = config(&[r"C:\", "/"]);

    let list = build(&config, &[]);

    let drive = workspace(&list, r"C:\");
    assert_eq!(drive.name, r"C:\");
    assert_eq!(drive.label, r"C:\");
    let root = workspace(&list, "/");
    assert_eq!(root.name, "/");
    assert_eq!(root.label, "/");
}
