//! The dialog slice of the contract, read from the app state (T7a).
//!
//! One blocking extension dialog at a time: the FIRST pending dialog of the
//! foreground chat, exactly as the iced shell draws it
//! (`crates/ui/src/view/dialog.rs`). The app queues them oldest first and keeps
//! a background chat's dialog waiting for the user to return to that chat (S4),
//! so nothing here reaches into another chat's queue.
//!
//! The typed text of an input or editor dialog is NOT here and never travels
//! through this slice: it is the web view's own state (S2), and the only thing
//! that comes back is the answer.

use gentle_app::AppState;
use gentle_core::chat::Dialog;

use crate::contract::DialogView;

/// The foreground chat's first pending dialog, or `None` when no chat is
/// foreground or nothing is waiting.
pub(crate) fn dialog(state: &AppState) -> Option<DialogView> {
    let chat = state.chats.foreground()?;
    state.chats.dialogs(chat).first().map(view_dialog)
}

/// One `gentle_core` dialog as the contract names it. The two enums carry the
/// same four kinds and the same fields.
fn view_dialog(dialog: &Dialog) -> DialogView {
    match dialog {
        Dialog::Confirm { id, title, message } => DialogView::Confirm {
            id: id.clone(),
            title: title.clone(),
            message: message.clone(),
        },
        Dialog::Select { id, title, options } => DialogView::Select {
            id: id.clone(),
            title: title.clone(),
            options: options.clone(),
        },
        Dialog::Input {
            id,
            title,
            placeholder,
        } => DialogView::Input {
            id: id.clone(),
            title: title.clone(),
            placeholder: placeholder.clone(),
        },
        Dialog::Editor { id, title, prefill } => DialogView::Editor {
            id: id.clone(),
            title: title.clone(),
            prefill: prefill.clone(),
        },
    }
}
