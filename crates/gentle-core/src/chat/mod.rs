//! Chat state and the pure reducer that folds RPC events into it.
//!
//! Behaviour ported from the reference app's `chatReducer.ts`: working state,
//! the assistant message lifecycle, the activity counter, pending extension
//! dialogs and the last extension error. Nothing here reimplements agent
//! logic; it only mirrors what `gentle-shell --mode rpc` reports.

mod history;
mod reducer;
mod state;

pub use history::messages_from_history;
pub use reducer::{reduce_chat, reduce_chat_at};
pub use state::{ChatMessage, ChatRole, ChatState, Dialog};
