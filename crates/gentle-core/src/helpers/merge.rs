//! Keeping tasks that gentle-agents stops reporting.
//!
//! gentle-agents holds a task in its in-memory store only while it runs; once
//! it finishes the record moves to disk and the next frame omits it, although
//! `summary.finished` still counts it. Without this merge the finished count
//! would go up while the task vanished from the list.

use super::types::{HelperStatus, HelperTask};

/// Merges a new frame's tasks with the previous ones.
///
/// - A task in `next` replaces its previous record outright: the frame is
///   authoritative for whatever it still reports.
/// - A previous task missing from `next` is retained. If it was still
///   running, waiting or queued it is promoted to done, with `ended_at_ms`
///   filled from `now_ms` unless it already had one; a terminal status is
///   never re-derived.
///
/// The result is ordered running, waiting, queued, then finished by end time
/// (newest first).
pub fn merge_tasks(
    previous: &[HelperTask],
    next: Vec<HelperTask>,
    now_ms: Option<u64>,
) -> Vec<HelperTask> {
    let mut merged = next;
    let retained: Vec<HelperTask> = previous
        .iter()
        .filter(|old| !merged.iter().any(|task| task.id == old.id))
        .map(|old| retain(old, now_ms))
        .collect();
    merged.extend(retained);
    order(merged)
}

fn retain(task: &HelperTask, now_ms: Option<u64>) -> HelperTask {
    match task.status {
        HelperStatus::Running | HelperStatus::Waiting | HelperStatus::Queued => HelperTask {
            status: HelperStatus::Done,
            ended_at_ms: task.ended_at_ms.or(now_ms),
            ..task.clone()
        },
        HelperStatus::Done | HelperStatus::Failed | HelperStatus::Cancelled => task.clone(),
    }
}

fn order(tasks: Vec<HelperTask>) -> Vec<HelperTask> {
    let rank = |status: HelperStatus| match status {
        HelperStatus::Running => 0,
        HelperStatus::Waiting => 1,
        HelperStatus::Queued => 2,
        HelperStatus::Done | HelperStatus::Failed | HelperStatus::Cancelled => 3,
    };
    let mut ordered = tasks;
    // Stable: ties keep the frame's order. A missing end time sorts last.
    ordered.sort_by(|a, b| {
        rank(a.status).cmp(&rank(b.status)).then_with(|| {
            if rank(a.status) == 3 {
                b.ended_at_ms.cmp(&a.ended_at_ms)
            } else {
                std::cmp::Ordering::Equal
            }
        })
    });
    ordered
}
