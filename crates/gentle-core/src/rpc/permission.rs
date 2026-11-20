//! The session permission mode over the `/gentle:desktop` channel (composer-dock
//! T3a).
//!
//! `get_permission_mode` takes `{}` and answers `{ mode, default_mode }`;
//! `set_permission_mode` takes `{ mode }` (and an optional `persist`, which this
//! module never sends: the UI changes this session, never the saved default) and
//! answers the state re-read after the attempt (`desktop-rpc.md:479-560`).
//!
//! The mode travels as the channel's own string. [`SessionMode`] is what this
//! app may ASK for; [`PermissionState::mode`] is what the channel REPORTS, kept
//! raw so a value this build does not know is shown, never hidden.

use serde_json::{json, Map, Value};

use super::answer::{required_str, AnswerError};
use super::desktop::{
    DesktopChannel, DesktopClient, DesktopError, DesktopOutcome, NoAnswer, RequestError,
};

/// The method that reads the session mode.
pub const GET_PERMISSION_MODE: &str = "get_permission_mode";
/// The method that changes the session mode.
pub const SET_PERMISSION_MODE: &str = "set_permission_mode";

/// A mode this app can ask the channel for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionMode {
    Normal,
    Auto,
    Yolo,
    SuperYolo,
}

impl SessionMode {
    /// The mode named by the channel's wire string, exactly as it is written.
    pub fn parse(wire: &str) -> Option<Self> {
        match wire {
            "normal" => Some(Self::Normal),
            "auto" => Some(Self::Auto),
            "yolo" => Some(Self::Yolo),
            "super-yolo" => Some(Self::SuperYolo),
            _ => None,
        }
    }

    /// The wire string the channel expects.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Auto => "auto",
            Self::Yolo => "yolo",
            Self::SuperYolo => "super-yolo",
        }
    }

    /// Whether choosing this mode takes an explicit confirmation first: the two
    /// that stop asking for permission.
    pub fn needs_confirmation(self) -> bool {
        matches!(self, Self::Yolo | Self::SuperYolo)
    }
}

/// The state the owner reports: the session's mode and the saved default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionState {
    /// The session mode in effect, as the channel wrote it.
    pub mode: String,
    /// The default a new primary session would start from, as the channel wrote it.
    pub default_mode: String,
}

/// The `get_permission_mode` params: an empty object, nothing else is accepted.
pub fn get_permission_params() -> Value {
    json!({})
}

/// The `set_permission_mode` params: exactly the mode. `persist` is never sent.
pub fn set_permission_params(mode: SessionMode) -> Value {
    json!({ "mode": mode.as_str() })
}

/// One permission exchange, read or write: the state, or why there is none.
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionOutcome {
    /// A correlated `ok` answer: the state as the owner re-read it.
    Answered(PermissionState),
    /// The owner refused (`permission_denied`, `permission_unavailable`,
    /// `invalid_request`, ...): the mode is unchanged.
    Rejected(DesktopError),
    /// The child's `capabilities` does not list the method.
    Unsupported,
    /// No usable answer: an inactive host, a timeout, a closed child.
    NoAnswer(NoAnswer),
    /// The request broke a contract limit and never left.
    InvalidRequest(RequestError),
    /// The child answered `ok`, but not this contract's answer.
    UnexpectedAnswer(AnswerError),
}

/// Reads the session mode through a client that keeps this chat's request ids.
///
/// Asks `capabilities` first when the client has none, so a failed connection is
/// retried on the next read instead of poisoning the chat; support is never
/// inferred.
pub fn read_permission<C: DesktopChannel>(client: &mut DesktopClient<C>) -> PermissionOutcome {
    ask(client, GET_PERMISSION_MODE, get_permission_params())
}

/// Asks the owner to switch this session to `mode` and answers the state it
/// re-read afterwards, never an echo of the request.
pub fn write_permission<C: DesktopChannel>(
    client: &mut DesktopClient<C>,
    mode: SessionMode,
) -> PermissionOutcome {
    ask(client, SET_PERMISSION_MODE, set_permission_params(mode))
}

fn ask<C: DesktopChannel>(
    client: &mut DesktopClient<C>,
    method: &str,
    params: Value,
) -> PermissionOutcome {
    if client.capabilities().is_none() {
        client.connect();
    }
    match client.ask(method, params) {
        DesktopOutcome::Data { data, .. } => match parse_state(&data) {
            Ok(state) => PermissionOutcome::Answered(state),
            Err(error) => PermissionOutcome::UnexpectedAnswer(error),
        },
        DesktopOutcome::Rejected { error, .. } => PermissionOutcome::Rejected(error),
        DesktopOutcome::Unsupported { .. } => PermissionOutcome::Unsupported,
        DesktopOutcome::NoAnswer(reason) => PermissionOutcome::NoAnswer(reason),
        DesktopOutcome::Invalid(error) => PermissionOutcome::InvalidRequest(error),
    }
}

fn parse_state(data: &Value) -> Result<PermissionState, AnswerError> {
    let object: &Map<String, Value> = data
        .as_object()
        .ok_or(AnswerError::NotAnObject { field: "data" })?;
    Ok(PermissionState {
        mode: required_str(object, "mode")?,
        default_mode: required_str(object, "default_mode")?,
    })
}
