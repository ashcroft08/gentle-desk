//! The chat-content slice of the contract, read from the app state.
//!
//! Only what a child's events change and what the state really keeps: the
//! foreground chat's thread. A chat's stderr lines stay the state's private
//! ring and reach the view only as the reason of a death notice; dialogs,
//! helpers and the session list are other slices.

use std::collections::HashMap;

use gentle_app::AppState;
use gentle_core::chat::{ChatItem, ChatMessage, ChatRole, ChatState, ToolPayload};

use crate::contract::{
    ChatItemView, ImageView, MessageRole, MessageView, ThinkingItemView, ToolCallItemView,
};
use crate::images::{ImageRef, Images};

/// How many of the foreground chat's rows the push carries, in BOTH lists.
///
/// One message's text is the model's whole markdown reply, and one tool row can
/// carry three payloads, so a long session would push megabytes on every event;
/// the thread renders the tail, and the older rows of a reopened session arrive
/// with its history (a later slice), not with every push.
pub const MESSAGES_PUSHED: usize = 50;

/// The images a chat sent, by message id: what a text row drew when it was sent.
type Sent = HashMap<String, Vec<ImageRef>>;

/// The foreground chat's messages, oldest first, cut to the last
/// [`MESSAGES_PUSHED`]. No foreground chat is no thread.
///
/// `images` answers which of them were sent with images: the app state carries
/// none (S2 of the migration: `crates/app` is not changed by this slice), so the
/// shell remembers them per chat and message id, and a message without any draws
/// the empty list it always drew.
pub(crate) fn messages(state: &AppState, images: &Images) -> Vec<MessageView> {
    let Some((chat, thread)) = thread(state) else {
        return Vec::new();
    };
    let sent = images.sent(chat.as_str());
    // The transcript keeps thinking and tool rows too; a message is what this
    // list pushes, so it is the text projection that travels (L5). The window
    // is the last MESSAGES_PUSHED text rows, taken from the END of the thread by
    // the state's own accessor, so the cost is the rows that travel - never a
    // count and a skip that walk the whole transcript on every refresh (R3-002).
    thread
        .last_text_items(MESSAGES_PUSHED)
        .map(|message| message_view(message, &sent))
        .collect()
}

/// The foreground chat's transcript, oldest row first, cut to the last
/// [`MESSAGES_PUSHED`] rows of ANY kind: what a card draws, in the order the
/// child produced it (chat-transcript T2a, L4).
///
/// The window comes from [`ChatState::last_items`], which walks the transcript
/// from its end and stops at the limit-th row, so the cost is the rows that
/// travel and the head of a long session is never visited - the same bound, and
/// the same argument, as [`messages`]. A text row built here is the row
/// [`messages`] pushes for the same message, because both go through
/// [`message_view`]: there is one rule for a text row, not two.
pub(crate) fn items(state: &AppState, images: &Images) -> Vec<ChatItemView> {
    let Some((chat, thread)) = thread(state) else {
        return Vec::new();
    };
    let sent = images.sent(chat.as_str());
    thread
        .last_items(MESSAGES_PUSHED)
        .map(|item| item_view(item, &sent))
        .collect()
}

/// The thread a session FILE paints while its chat's child is still starting
/// (chat-open-latency T1): the rows in BOTH lists a push carries, built by the
/// one rule [`message_view`] applies to a text row, so a file row and the child's
/// own row for the same message read the same - only their ids differ, and the
/// ids are the core's [`PROVISIONAL_PREFIX`](gentle_core::chat::PROVISIONAL_PREFIX)
/// ones, never a `msg-<n>` the reducer owns.
///
/// No image is ever attached: the shell remembers the images of the messages this
/// run SENT, and a row read from a file was not one of them.
pub(crate) fn provisional(rows: &[ChatMessage]) -> (Vec<MessageView>, Vec<ChatItemView>) {
    let sent = Sent::new();
    let messages: Vec<MessageView> = rows
        .iter()
        .map(|message| message_view(message, &sent))
        .collect();
    let items = messages
        .iter()
        .cloned()
        .map(ChatItemView::Message)
        .collect();
    (messages, items)
}

/// The foreground chat and its thread, which every slice of this module needs
/// before it can read a row.
fn thread(state: &AppState) -> Option<(&gentle_app::ChatId, &ChatState)> {
    let chat = state.chats.foreground()?;
    let thread = state.chats.state(chat)?;
    Some((chat, thread))
}

/// One text row, built the ONE way both lists build it.
///
/// Cloned, never moved: the state keeps its own copy of every id, text and error
/// string.
fn message_view(message: &ChatMessage, sent: &Sent) -> MessageView {
    MessageView {
        id: message.id.clone(),
        role: view_role(message.role),
        text: message.text.clone(),
        streaming: message.streaming,
        error: message.error.clone(),
        images: sent
            .get(&message.id)
            .map(|images| images.iter().cloned().map(ImageView::from).collect())
            .unwrap_or_default(),
    }
}

/// One row of the transcript, whichever shape the chat gave it.
fn item_view(item: &ChatItem, sent: &Sent) -> ChatItemView {
    match item {
        ChatItem::Text(message) => ChatItemView::Message(message_view(message, sent)),
        ChatItem::Thinking {
            id,
            text,
            streaming,
            duration_ms,
            ..
        } => ChatItemView::Thinking(ThinkingItemView {
            id: id.clone(),
            text: text.clone(),
            streaming: *streaming,
            duration_ms: duration_ms.map(millis),
        }),
        ChatItem::Tool {
            id,
            tool_name,
            args,
            partial_result,
            result,
            is_error,
            running,
            duration_ms,
            ..
        } => ChatItemView::Tool(ToolCallItemView {
            id: id.clone(),
            tool_name: tool_name.clone(),
            // The child's own JSON, as it stands: the card decides how to draw
            // it, so nothing here flattens one into a line this side chose.
            args: args.as_ref().map(payload),
            partial_result: partial_result.as_ref().map(payload),
            result: result.as_ref().map(payload),
            is_error: *is_error,
            running: *running,
            duration_ms: duration_ms.map(millis),
            // The row's own two marks, from the rule the core owns (R3-2): a
            // card words "the output was cut" and "a field was dropped"
            // differently, and only the core knows which happened.
            truncated: item.payload_truncated(),
            dropped: item.payload_dropped(),
        }),
    }
}

/// One payload's JSON, as the contract carries it.
fn payload(payload: &ToolPayload) -> serde_json::Value {
    payload.value.clone()
}

/// A millisecond count as the double the contract carries: the generated
/// bindings refuse a 64-bit integer on the wire.
fn millis(duration_ms: u64) -> f64 {
    duration_ms as f64
}

/// The contract's role enum matches the chat state's roles one to one.
fn view_role(role: ChatRole) -> MessageRole {
    match role {
        ChatRole::User => MessageRole::User,
        ChatRole::Assistant => MessageRole::Assistant,
    }
}
