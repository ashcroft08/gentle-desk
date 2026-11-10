//! Unit tests of the two rules the helper thread slice keeps on its own: the
//! selection lookup and the dropped count.

use gentle_core::helpers::{HelperStatus, HelperTask, HelperThread};

use super::{dropped, open};

fn task(id: &str) -> HelperTask {
    HelperTask {
        id: id.to_owned(),
        agent: "explore".to_owned(),
        label: String::new(),
        prompt: String::new(),
        status: HelperStatus::Running,
        created_at_ms: 0,
        started_at_ms: None,
        ended_at_ms: None,
        last_step: None,
        last_activity_at_ms: None,
        turns: 0,
        tool_calls: 0,
        error: None,
        thread: HelperThread::default(),
    }
}

#[test]
fn the_selected_task_is_the_one_whose_thread_opens() {
    let tasks = [task("r"), task("v")];

    let open = open(&tasks, "v").expect("a listed task has a thread");

    assert_eq!(open.id, "v");
}

#[test]
fn an_id_the_panel_does_not_list_has_no_thread() {
    assert!(open(&[], "ghost").is_none());
    assert!(
        open(&[task("r")], "ghost").is_none(),
        "an id that names no task falls back to the list"
    );
}

#[test]
fn a_dropped_count_that_does_not_fit_saturates_instead_of_wrapping() {
    assert_eq!(dropped(u64::MAX, 1), u32::MAX);
    assert_eq!(dropped(0, 0), 0);
}
