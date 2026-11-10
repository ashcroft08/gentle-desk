use super::*;
use gentle_core::helpers::HelperThread;

fn task(label: &str, prompt: &str, agent: &str) -> HelperTask {
    HelperTask {
        id: "task-1".into(),
        agent: agent.into(),
        label: label.into(),
        prompt: prompt.into(),
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
fn the_panel_only_shows_when_there_are_tasks() {
    assert!(!is_visible(&HelpersActivity::default()));
    let with_task = HelpersActivity {
        tasks: vec![task("a", "", "")],
        ..HelpersActivity::default()
    };
    assert!(is_visible(&with_task));
}
