//! Why a chat's child died: the app keeps the last stderr lines of each chat's
//! child in a bounded ring, and the death notice (S3) reports them as the
//! reason, in one row per chat. A chat whose stored working directory is gone
//! fails this way: the child exits after printing
//! "Stored session working directory does not exist: <path>".

use gentle_app::{update_chats, AppState, ChatId, Input, Intent};

fn chat(name: &str) -> ChatId {
    ChatId::of_session(name)
}

fn open(state: AppState, name: &str) -> AppState {
    update_chats(state, Input::Intent(Intent::OpenChat(chat(name)))).0
}

fn stderr(state: AppState, name: &str, line: &str) -> AppState {
    update_chats(
        state,
        Input::ChatStderr {
            chat: chat(name),
            line: line.into(),
        },
    )
    .0
}

fn closed(state: AppState, name: &str, expected: bool) -> AppState {
    update_chats(
        state,
        Input::ChatClosed {
            chat: chat(name),
            expected,
        },
    )
    .0
}

#[test]
fn a_death_notice_reports_the_stderr_lines_its_child_left() {
    let state = open(AppState::default(), "a");
    let state = stderr(
        state,
        "a",
        "Stored session working directory does not exist: C:/dev/proj5",
    );
    let state = stderr(state, "a", "Session file: C:/home/sessions/a.jsonl");

    let state = closed(state, "a", false);

    let notice = state.notices.last().expect("a death leaves its notice");
    assert_eq!(
        notice.text,
        "a: the child that runs this chat exited (Stored session working directory does not exist: C:/dev/proj5 · Session file: C:/home/sessions/a.jsonl). Restart to keep talking.",
        "the notice says why the child died"
    );
}

#[test]
fn without_stderr_the_notice_keeps_the_text_it_always_had() {
    let state = open(AppState::default(), "a");

    let state = closed(state, "a", false);

    let notice = state.notices.last().expect("a notice");
    assert_eq!(
        notice.text,
        "a: the child that runs this chat exited. Restart to keep talking."
    );
}

#[test]
fn blank_stderr_lines_are_never_a_reason() {
    let state = open(AppState::default(), "a");
    let state = stderr(state, "a", "");
    let state = stderr(state, "a", "   ");

    let state = closed(state, "a", false);

    let notice = state.notices.last().expect("a notice");
    assert_eq!(
        notice.text,
        "a: the child that runs this chat exited. Restart to keep talking."
    );
}

#[test]
fn the_ring_keeps_the_last_three_lines_and_cuts_long_ones() {
    let state = open(AppState::default(), "a");
    let state = stderr(state, "a", "one");
    let state = stderr(state, "a", "two");
    let state = stderr(state, "a", &"a".repeat(300));
    let state = stderr(state, "a", "three");

    let state = closed(state, "a", false);

    let notice = state.notices.last().expect("a notice");
    assert!(
        notice.text.contains(" · "),
        "the reasons are joined: {notice:?}"
    );
    assert!(notice.text.contains("two"), "the last lines are kept");
    assert!(notice.text.contains("three"));
    assert!(
        notice.text.contains(&"a".repeat(200)),
        "shown down to the cap"
    );
    assert!(
        !notice.text.contains(&"a".repeat(201)),
        "a long line is cut at 200 characters"
    );
    assert!(
        !notice.text.contains("one"),
        "only the last three lines are kept"
    );
}

#[test]
fn an_expected_stop_is_silent_whatever_its_child_complained_about() {
    let state = open(AppState::default(), "a");
    let state = stderr(state, "a", "some slowdown warning");

    let state = closed(state, "a", true);

    assert!(state.notices.is_empty(), "the host stopped it on purpose");
}

#[test]
fn one_chats_stderr_never_leaks_into_another_chats_notice() {
    let state = open(AppState::default(), "a");
    let state = open(state, "b");
    let state = stderr(state, "a", "the cwd of a is gone");

    let state = closed(state, "b", false);

    let notice = state.notices.last().expect("a notice");
    assert_eq!(
        notice.text, "b: the child that runs this chat exited. Restart to keep talking.",
        "b's death has no reason of its own"
    );
    let state = closed(state, "a", false);

    let notices: Vec<String> = state
        .notices
        .iter()
        .map(|notice| notice.text.clone())
        .collect();
    assert!(
        notices
            .iter()
            .any(|text| text.contains("the cwd of a is gone")),
        "a's own reason is reported: {notices:?}"
    );
    assert_eq!(notices.len(), 2, "one row per chat that died: {notices:?}");
}

#[test]
fn a_fresh_child_starts_with_a_clean_stderr_ring() {
    let state = open(AppState::default(), "a");
    let state = stderr(state, "a", "the cwd of a is gone");
    let state = closed(state, "a", false);

    // The restart clears what the old child left behind: a fresh open.
    let state = open(state, "a");

    let state = closed(state, "a", false);

    let notice = state.notices.last().expect("a notice");
    assert_eq!(
        notice.text, "a: the child that runs this chat exited. Restart to keep talking.",
        "the old child's stderr is not the new child's reason: {notice:?}"
    );
}

#[test]
fn a_death_noticed_twice_with_the_same_reason_stays_one_row() {
    let state = open(AppState::default(), "a");
    let state = open(state, "b");
    let state = stderr(state, "a", "stored cwd gone");
    let state = closed(state, "a", false);
    let state = open(state, "a"); // restart: ring cleared with the fresh child
    let state = stderr(state, "a", "stored cwd gone");
    let state = closed(state, "a", false);

    let rows = state
        .notices
        .iter()
        .filter(|notice| notice.text.contains("stored cwd gone"))
        .count();

    assert_eq!(rows, 1, "dedup by level and text still applies");
}
