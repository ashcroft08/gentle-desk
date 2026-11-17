//! The pieces of a `/gentle:desktop` answer that more than one method reads.
//!
//! Three things are shared by every store answer and by nothing else:
//!
//! - the layer vocabulary (`ok`, `missing`, `invalid`), which `status`,
//!   `layers[].status` and `writeTargets[].status` all speak;
//! - the two revision tokens, which are deliberately **different types**:
//!   [`StoreRevision`] is the store's read-time digest, [`WriteRevision`] is one
//!   write target's own digest. `set_config` compares `expectedRevision` against
//!   the second and "never against `revision`" (`desktop-rpc.md:265-273`), so the
//!   type system carries the difference instead of a comment;
//! - [`AnswerError`], the typed reason one member of an `ok: true` answer is not
//!   this contract's. A missing required member, a member of another JSON type
//!   and a member naming a value this contract does not define are all typed
//!   errors, never a panic and never a partially trusted read.
//!
//! Everything here is pure: no clock, no IO, no channel.

use std::fmt;

use serde_json::{Map, Value};

/// One layer's status: the `ok` / `missing` / `invalid` vocabulary.
///
/// `status` answers for the whole store, `layers[].status` for one file, and
/// `writeTargets[].status` for the file a write would replace
/// (`desktop-rpc.md:155-176`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerStatus {
    /// Normalized content follows.
    Ok,
    /// No layer file exists.
    Missing,
    /// Layer files exist but none produced content.
    Invalid,
}

impl LayerStatus {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "ok" => Some(Self::Ok),
            "missing" => Some(Self::Missing),
            "invalid" => Some(Self::Invalid),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Missing => "missing",
            Self::Invalid => "invalid",
        }
    }
}

/// The store's read-time digest: `get_config`'s `revision`.
///
/// It is opaque here and it is deliberately *not* a write token: it digests the
/// merged, normalized content of every layer, so for `pins` it cannot identify
/// the file a write replaces (`desktop-rpc.md:265-273`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreRevision(String);

impl StoreRevision {
    /// The answer's own string, or `None` for the empty one no answer reports.
    pub fn parse(value: &str) -> Option<Self> {
        (!value.is_empty()).then(|| Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One write target's own digest: `get_config`'s `writeTargets[].writeRevision`.
///
/// This is the token `set_config` requires as `expectedRevision`, because a write
/// replaces one file, not one store. The spec's own rule - it is compared against
/// this token and "never against `revision`" - is a type boundary here: a
/// [`StoreRevision`] cannot be passed where a [`WriteRevision`] is needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteRevision(String);

impl WriteRevision {
    /// The answer's own string, or `None` for the empty one that would prove
    /// nothing about the state the client saw.
    pub fn parse(value: &str) -> Option<Self> {
        (!value.is_empty()).then(|| Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Why one member of an `ok: true` answer is not this contract's.
///
/// `field` names the member as the wire spells it (`readOnly`,
/// `expectedRevision`, `orchestratorMode`), not a dotted path: an answer either
/// has that member or it does not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnswerError {
    /// A member the answer needs is absent.
    MissingField { field: &'static str },
    /// A member that must be a JSON object is another JSON type.
    NotAnObject { field: &'static str },
    /// A member has a JSON type this contract does not define there.
    WrongType { field: &'static str },
    /// A member names a value this contract does not define.
    UnknownValue { field: &'static str, value: String },
}

impl fmt::Display for AnswerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingField { field } => {
                write!(f, "the answer has no {field} member")
            }
            Self::NotAnObject { field } => {
                write!(f, "the answer's {field} member is not an object")
            }
            Self::WrongType { field } => {
                write!(f, "the answer's {field} member has another JSON type")
            }
            Self::UnknownValue { field, value } => {
                write!(f, "the answer's {field} member names {value:?}")
            }
        }
    }
}

impl std::error::Error for AnswerError {}

/// `value` as a JSON object, or the typed reason it is not one.
pub(super) fn as_object<'a>(
    value: &'a Value,
    field: &'static str,
) -> Result<&'a Map<String, Value>, AnswerError> {
    value.as_object().ok_or(AnswerError::NotAnObject { field })
}

/// The `field` member of `object`, which the answer must carry.
pub(super) fn required_member<'a>(
    object: &'a Map<String, Value>,
    field: &'static str,
) -> Result<&'a Value, AnswerError> {
    object.get(field).ok_or(AnswerError::MissingField { field })
}

pub(super) fn required_str(
    object: &Map<String, Value>,
    field: &'static str,
) -> Result<String, AnswerError> {
    required_member(object, field)?
        .as_str()
        .map(str::to_owned)
        .ok_or(AnswerError::WrongType { field })
}

/// The `field` member when it is a string. An absent member and an explicit
/// `null` both read as "not there", which is how the owner serializes an unset
/// optional member.
pub(super) fn optional_str(
    object: &Map<String, Value>,
    field: &'static str,
) -> Result<Option<String>, AnswerError> {
    match object.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_str()
            .map(|text| Some(text.to_owned()))
            .ok_or(AnswerError::WrongType { field }),
    }
}

pub(super) fn required_bool(
    object: &Map<String, Value>,
    field: &'static str,
) -> Result<bool, AnswerError> {
    required_member(object, field)?
        .as_bool()
        .ok_or(AnswerError::WrongType { field })
}

pub(super) fn optional_bool(
    object: &Map<String, Value>,
    field: &'static str,
) -> Result<Option<bool>, AnswerError> {
    match object.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_bool()
            .map(Some)
            .ok_or(AnswerError::WrongType { field }),
    }
}

pub(super) fn required_u64(
    object: &Map<String, Value>,
    field: &'static str,
) -> Result<u64, AnswerError> {
    required_member(object, field)?
        .as_u64()
        .ok_or(AnswerError::WrongType { field })
}

/// The `field` member as an object, which the answer must carry.
pub(super) fn required_object<'a>(
    object: &'a Map<String, Value>,
    field: &'static str,
) -> Result<&'a Map<String, Value>, AnswerError> {
    as_object(required_member(object, field)?, field)
}

pub(super) fn optional_object<'a>(
    object: &'a Map<String, Value>,
    field: &'static str,
) -> Result<Option<&'a Map<String, Value>>, AnswerError> {
    match object.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => as_object(value, field).map(Some),
    }
}

/// The `field` member as an array of values; an absent or `null` member is an
/// empty list, which is how the owner reports "none of them".
pub(super) fn optional_array<'a>(
    object: &'a Map<String, Value>,
    field: &'static str,
) -> Result<&'a [Value], AnswerError> {
    match object.get(field) {
        None | Some(Value::Null) => Ok(&[]),
        Some(value) => value
            .as_array()
            .map(Vec::as_slice)
            .ok_or(AnswerError::WrongType { field }),
    }
}
