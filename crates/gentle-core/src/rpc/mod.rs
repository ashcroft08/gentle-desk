//! pi's `--mode rpc` JSON-line protocol.

pub mod codec;
pub mod types;

pub use codec::{decode_line, encode_command, LineSplitter};
pub use types::*;
