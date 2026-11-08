//! Process layer of gentle-desk.
//!
//! Resolves how to invoke `gentle-shell` and (later) owns the `--mode rpc`
//! child. Depends on `gentle-core` and never on any UI code.

pub mod chat_host;
pub mod home;
pub mod launcher;
pub mod process;
pub mod session_cache;
pub mod session_index;
pub mod session_schedule;
pub mod session_snapshot;
pub mod session_watch;

pub use session_snapshot::{FileSnapshot, IndexSnapshot};
