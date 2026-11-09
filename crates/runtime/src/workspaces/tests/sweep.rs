//! A smoke test of the one pass: the whole real-world input size is grouped
//! without a per-session walk of every workspace. Nothing here is timed.

use super::*;

const FOLDERS: usize = 200;
const SESSIONS: usize = 50_000;

#[test]
fn fifty_thousand_sessions_are_grouped_in_one_list() {
    let folders: Vec<String> = (0..FOLDERS).map(|at| format!(r"C:\dev\p{at}")).collect();
    let sessions: Vec<SessionItem> = (0..SESSIONS)
        .map(|at| session(&folders[at % FOLDERS], at as u64))
        .collect();

    let list = build(&AppConfig::default(), &sessions);

    assert_eq!(list.workspaces.len(), FOLDERS);
    let placed: usize = list
        .workspaces
        .iter()
        .map(|workspace| workspace.sessions.len())
        .sum();
    assert_eq!(placed, SESSIONS, "every session is in its workspace");
    assert_eq!(list.unplaced, 0);
    assert_eq!(
        list.workspaces[0].last_activity_ms,
        Some(SESSIONS as u64 - 1),
        "the newest session is on the first row"
    );
    assert!(list
        .workspaces
        .windows(2)
        .all(|pair| pair[0].last_activity_ms >= pair[1].last_activity_ms));
}
