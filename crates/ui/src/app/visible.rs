//! Which chat the window shows, and what it may do with it.
//!
//! One place answers "the chat on screen" so the frame, the composer and the
//! cache all agree, and one place decides whether that chat can be talked to.

use gentle_app::AppState;
use gentle_core::chat::ChatState;

/// The chat the window shows: the foreground one of the per-chat shell, or the
/// empty single thread before the first chat is opened.
pub(super) fn chat(app: &AppState) -> &ChatState {
    app.chats
        .foreground()
        .and_then(|chat| app.chats.state(chat))
        .unwrap_or(&app.chat)
}

/// Whether the chat the window shows has lost its child (S3). Only that chat is
/// disabled: every other chat keeps its own child and stays usable.
pub(super) fn ended(app: &AppState) -> bool {
    app.chats
        .foreground()
        .is_some_and(|chat| app.chats.is_ended(chat))
}

/// Whether the composer may send that text right now.
///
/// Disabled for a failure that ends everything (gentle-shell could not be
/// resolved) and for a chat whose child died: there is nowhere for the prompt to
/// go, and the notice beside it carries the Restart. Nothing else disables it, so
/// one death never takes the other chats with it.
pub(super) fn can_send(app: &AppState, input: &str, fatal: bool) -> bool {
    !input.trim().is_empty() && !fatal && !ended(app)
}

/// The session file the sidebar marks as open.
pub(super) fn current_session(app: &AppState) -> Option<&str> {
    app.chats
        .foreground()
        .and_then(|chat| chat.session_path())
        .or(app.current_session.as_deref())
}
