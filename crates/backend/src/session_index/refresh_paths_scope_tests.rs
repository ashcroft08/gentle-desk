//! What `refresh_paths` never touches: paths that are not session file
//! locations, and sessions nobody reported.

use super::test_home::{append, header, messages, user, Home};
use super::*;

#[test]
fn a_path_that_is_not_a_session_file_location_is_ignored() {
    let home = Home::new();
    let valid = home.write("s", &[header("s"), user("hola")]);
    // A directory that looks like a session file: still not one.
    fs::create_dir_all(home.project().join("dir.jsonl")).unwrap();
    fs::create_dir_all(home.project().join("sub")).unwrap();
    let outside = home.root().join("elsewhere").join("--C--a--");
    fs::create_dir_all(&outside).unwrap();
    fs::write(outside.join("s.jsonl"), format!("{}\n", header("s"))).unwrap();
    let ignored = [
        home.sessions().join("loose.jsonl"),           // depth 1
        home.project().join("sub").join("deep.jsonl"), // depth 3
        home.project().join("notes.txt"),              // .txt
        home.project().join("dir.jsonl"),              // a directory
        outside.join("s.jsonl"),                       // outside the home
        PathBuf::from("--C--a--").join("s.jsonl"),     // relative
    ];

    let mut index = home.index();
    assert_eq!(index.refresh_paths(&ignored), RefreshStats::default());
    assert!(index.entries().is_empty());

    // The same call with one real session path in it: only that one counts.
    let mut report = ignored.to_vec();
    report.push(valid.clone());
    let stats = index.refresh_paths(&report);
    assert_eq!((stats.files, stats.rescanned), (1, 1));
    assert_eq!(messages(&index, &valid), Some(1));
}

/// The property that proves there is no enumeration: a path nobody reported is
/// not even looked at, and a file created next door is not discovered.
#[test]
fn only_the_reported_paths_are_updated() {
    let home = Home::new();
    let a = home.write("a", &[header("a")]);
    let b = home.write("b", &[header("b")]);
    let mut index = home.index();
    index.refresh().unwrap();

    append(&a, &user("uno"));
    append(&b, &user("dos"));
    let c = home.write("c", &[header("c")]);

    let stats = index.refresh_paths(std::slice::from_ref(&a));
    assert_eq!(stats.files, 1);
    assert_eq!(messages(&index, &a), Some(1));
    assert_eq!(messages(&index, &b), Some(0), "B was not reported");
    assert_eq!(messages(&index, &c), None, "C was not reported");

    // The safety-net pass is what sees the rest.
    index.refresh().unwrap();
    assert_eq!(messages(&index, &a), Some(1));
    assert_eq!(messages(&index, &b), Some(1));
    assert_eq!(messages(&index, &c), Some(0));
}
