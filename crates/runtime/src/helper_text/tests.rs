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
fn the_title_falls_back_from_label_to_prompt_to_agent_to_id() {
    assert_eq!(task_title(&task("Research", "p", "researcher")), "Research");
    assert_eq!(
        task_title(&task("  ", "\n  Look into auth\nmore", "researcher")),
        "Look into auth"
    );
    assert_eq!(task_title(&task("", "", "researcher")), "researcher");
    assert_eq!(task_title(&task("", "", "")), "task-1");
}

#[test]
fn the_summary_line_lists_the_four_counters() {
    let summary = HelpersSummary {
        running: 1,
        queued: 2,
        waiting: 3,
        finished: 4,
    };
    assert_eq!(
        summary_line(&summary),
        "1 running · 2 queued · 3 waiting · 4 finished"
    );
}

#[test]
fn every_status_has_a_label() {
    assert_eq!(status_label(HelperStatus::Cancelled), "cancelled");
    assert_eq!(status_label(HelperStatus::Waiting), "waiting");
}
