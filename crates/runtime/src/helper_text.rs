//! The helpers (delegated subagents) panel as plain text: the status label, the
//! task title and the counters line both windows draw, plus, in [`format`],
//! the short text of one tool call.
//!
//! Nothing here draws: the iced panel (`crates/ui/src/view/helpers.rs` and
//! `crates/ui/src/view/helper_thread.rs`) and, later, the Tauri app both sit on
//! top of this module. The colour a status is drawn in stays with the window,
//! and so does the language: the caller holds it and passes it in, because the
//! words are the catalog's (`gentle_core::i18n::messages`).
//!
//! [`task_title`] is the ONE text here that is not the app's own words: it is
//! the task's label, prompt or agent as the shell reported them, so it is shown
//! exactly as it arrives (D7) and takes no language.

pub mod format;

use gentle_core::helpers::{HelperStatus, HelperTask, HelpersSummary};
use gentle_core::i18n::{messages, Language};

/// "1 running · 0 queued · 0 waiting · 2 finished", in `language`.
pub fn summary_line(language: Language, summary: &HelpersSummary) -> String {
    messages::helpers_summary(
        language,
        summary.running,
        summary.queued,
        summary.waiting,
        summary.finished,
    )
}

/// The one word a task's status wears, in `language`.
pub const fn status_label(language: Language, status: HelperStatus) -> &'static str {
    messages::helper_status_label(language, status)
}

/// The label if there is one, else the first line of the prompt, else the agent, else the id.
pub fn task_title(task: &HelperTask) -> String {
    let first_line = |text: &str| {
        text.lines()
            .map(str::trim)
            .find(|l| !l.is_empty())
            .map(str::to_owned)
    };
    first_line(&task.label)
        .or_else(|| first_line(&task.prompt))
        .or_else(|| first_line(&task.agent))
        .unwrap_or_else(|| task.id.clone())
}

#[cfg(test)]
mod tests;
