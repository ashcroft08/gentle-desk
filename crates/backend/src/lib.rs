//! Process layer of gentle-desk.
//!
//! Resolves how to invoke `gentle-shell` and (later) owns the `--mode rpc`
//! child. Depends on `gentle-core` and never on any UI code.

pub mod launcher;
pub mod process;
