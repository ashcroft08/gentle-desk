//! pi's `--mode rpc` JSON-line protocol, plus the `/gentle:desktop` channel.

pub mod base64;
pub mod codec;
pub mod desktop;
pub mod helper_stop;
pub mod types;

pub use codec::{decode_line, encode_command, LineSplitter};
pub use desktop::*;
pub use helper_stop::{cancel_subagent_params, stop_helper, StopOutcome, CANCEL_SUBAGENT};
pub use types::*;
