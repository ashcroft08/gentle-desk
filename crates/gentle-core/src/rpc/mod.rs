//! pi's `--mode rpc` JSON-line protocol, plus the `/gentle:desktop` channel.

pub mod answer;
pub mod base64;
pub mod codec;
pub mod config_store;
pub mod desktop;
pub mod helper_stop;
pub mod profile_apply;
pub mod types;

pub use answer::*;
pub use codec::{decode_line, encode_command, LineSplitter};
pub use config_store::*;
pub use desktop::*;
pub use helper_stop::{cancel_subagent_params, stop_helper, StopOutcome, CANCEL_SUBAGENT};
pub use profile_apply::*;
pub use types::*;
