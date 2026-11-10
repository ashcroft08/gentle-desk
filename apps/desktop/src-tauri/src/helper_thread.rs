//! The open helper's thread slice of the contract (T7b-2), read from the app
//! state.
//!
//! `crates/ui/src/view/helper_thread.rs` draws the selected task's thread: what
//! it said, what it thought, its notes, and the tool calls it ran with their
//! status, arguments and output preview. This module maps that thread onto plain
//! display data, so no `serde_json::Value` ever reaches the web side.

use gentle_app::AppState;
use gentle_core::helpers::{HelperTask, HelperThreadItem};
use gentle_runtime::helper_text::format::{args_line, output_preview, tool_status, ToolStatus};
use gentle_runtime::helper_text::{status_label, task_title};
use gentle_runtime::text::truncate;

use crate::contract::{HelperThreadItemView, HelperThreadView, ToolStatusView};
use crate::helpers::{narrow, view_status};

/// How many items of one helper's thread the push carries.
///
/// iced draws every item a thread holds and scrolls them, but a thread can be
/// long, and a push on every event of the chat would then re-send a whole
/// transcript. The last items are what a reader following the helper looks at,
/// so the push keeps the tail - the same size as the chat thread's
/// `MESSAGES_PUSHED` tail - and counts the rest as dropped.
pub const THREAD_ITEMS_PUSHED: usize = 50;

/// Longest prompt echoed at the top, in characters: iced's own limit
/// (`crates/ui/src/view/helper_thread.rs`), applied here so the view draws the
/// text it is given.
const PROMPT_CHARS: usize = 300;

/// The selected helper's thread, or `None` when nothing is selected, no chat is
/// foreground, or the selected id names no task of that chat.
pub(crate) fn thread(state: &AppState) -> Option<HelperThreadView> {
    let chat = state.chats.foreground()?;
    let activity = &state.chats.state(chat)?.helpers;
    open(&activity.tasks, state.selected_helper.as_deref()?)
}

/// The thread of the task `selected` names among `tasks`. A selection only ever
/// names a card the panel draws (the reducer checks that), so an id that names
/// no task has no thread: the view then keeps drawing the list.
fn open(tasks: &[HelperTask], selected: &str) -> Option<HelperThreadView> {
    let task = tasks.iter().find(|task| task.id == selected)?;
    Some(view(task))
}

/// One task's thread as the thread view draws it: the header iced draws above
/// the items, then the tail of the items themselves.
fn view(task: &HelperTask) -> HelperThreadView {
    let total = task.thread.items.len();
    let start = total.saturating_sub(THREAD_ITEMS_PUSHED);
    HelperThreadView {
        id: task.id.clone(),
        title: task_title(task),
        agent: task.agent.clone(),
        status: view_status(task.status),
        status_label: status_label(task.status).to_owned(),
        prompt: truncate(&task.prompt, PROMPT_CHARS),
        dropped: dropped(task.thread.dropped, start),
        items: task.thread.items[start..].iter().map(item).collect(),
    }
}

/// How many earlier items the thread does not show: the ones the helper state
/// itself dropped, plus the `unsent` ones this push left out (the items before
/// the tail it carries).
fn dropped(dropped_by_state: u64, unsent: usize) -> u32 {
    narrow(dropped_by_state).saturating_add(narrow(unsent as u64))
}

/// One item as its own drawn shape. The words of a tool's status stay the
/// view's; the three states are the rule's (`tool_status`).
fn item(item: &HelperThreadItem) -> HelperThreadItemView {
    match item {
        HelperThreadItem::Text(text) => HelperThreadItemView::Said { text: text.clone() },
        HelperThreadItem::Thinking(text) => HelperThreadItemView::Thought { text: text.clone() },
        HelperThreadItem::Note(text) => HelperThreadItemView::Note { text: text.clone() },
        HelperThreadItem::Tool {
            name,
            args,
            output,
            running,
            is_error,
            ..
        } => HelperThreadItemView::Ran {
            name: name.clone(),
            status: view_tool_status(tool_status(*running, *is_error)),
            args_line: args_line(args),
            output_preview: output_preview(output),
        },
    }
}

/// The contract's tool status matches the rule's one to one.
fn view_tool_status(status: ToolStatus) -> ToolStatusView {
    match status {
        ToolStatus::Running => ToolStatusView::Running,
        ToolStatus::Failed => ToolStatusView::Failed,
        ToolStatus::Ok => ToolStatusView::Ok,
    }
}

#[cfg(test)]
mod tests;
