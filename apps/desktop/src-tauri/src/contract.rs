//! Rust-to-web contract types.
//!
//! The TypeScript declarations that the Svelte view compiles against are
//! GENERATED from the definitions below into
//! `apps/desktop/src/contract/bindings.ts`. A change on the Rust side must
//! land together with a regenerated file; `tests/contract_bindings.rs` fails
//! the build when the committed file no longer matches.

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ChatBadge {
    pub chat_id: String,
    pub activity: ChatActivityMark,
    pub unread: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum ChatActivityMark {
    Working,
    Queued,
    Opening,
    NewReply,
    NeedsAttention,
    Ended,
}

/// Sample contract command. Trivial by design: it exists so the generator
/// proves that command names, camel-cased argument names and return types are
/// typed over the same Rust definition, not re-declared by hand.
#[tauri::command]
#[specta::specta]
pub fn get_badge(chat_id: String) -> ChatBadge {
    ChatBadge {
        chat_id,
        activity: ChatActivityMark::Working,
        unread: false,
    }
}
