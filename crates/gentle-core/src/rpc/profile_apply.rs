//! Applying one saved profile through `/gentle:desktop`'s `apply_profile`
//! (profiles-editor T2a: S8, S9, and L8's findings).
//!
//! Saving a profile does not make it usable: `apply_profile` is what the
//! terminal's `/gentle:profiles` screen does after a save, reused whole
//! (`docs/desktop-rpc.md:839-905`). Two modes, and the difference matters:
//!
//! - [`ApplyMode::Default`] is **Make default for new sessions**: the global
//!   layer, which can be outranked by a repo pin, and the only mode that can
//!   answer `confirmation_required` - the screen's own diff dialog - which must
//!   be shown to the human and, if approved, resent with its token;
//! - [`ApplyMode::Session`] is **Use in this session**: it binds the profile to
//!   this RPC session's own child and never asks. L8 read that from gentle-pi's
//!   own source (`session` has no global write to confirm), so a `session` answer
//!   that asks anyway is reported as
//!   [`ProfileAnswerError::ConfirmationForSession`] (an answer this module does
//!   not act on) instead of being modeled as a round trip that cannot exist.
//!
//! The token is not authentication (`desktop-rpc.md:873-877`): it proves the
//! apply runs against the text that was shown. Getting a human to approve is the
//! caller's job, so nothing here resends a token on its own.
//!
//! As in [`super::helper_stop`], every failure is a typed outcome: the method is
//! only written when the child's `capabilities` advertised it, and an inactive
//! host, a timeout, a closed child, an unparsable payload or a refusal is
//! reported, never raised.

use std::fmt;

use serde_json::{json, Map, Value};

use super::answer::{as_object, optional_array, required_object, required_str, AnswerError};
use super::desktop::{
    DesktopChannel, DesktopClient, DesktopError, DesktopOutcome, NoAnswer, RequestError,
};

/// The method that applies one saved profile.
pub const APPLY_PROFILE: &str = "apply_profile";

/// The channel's own limit for `profile` and `confirmation`, in characters.
pub const MAX_NAME_CHARS: usize = 128;

/// Which of the screen's two outcomes this apply asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyMode {
    /// Bind the profile to this RPC session only. Writes no global routing, no
    /// pins and no materialized store, and never asks for confirmation.
    Session,
    /// Make the profile the global default for new sessions, through the
    /// screen's own diff dialog, pinned-repository rule and rollback.
    Default,
}

impl ApplyMode {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "session" => Some(Self::Session),
            "default" => Some(Self::Default),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Session => "session",
            Self::Default => "default",
        }
    }
}

/// Why a `profile` (or its confirmation) name the channel would refuse is never
/// sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileParamsError {
    /// The channel requires 1-128 characters.
    EmptyProfile,
    /// The channel requires 1-128 characters.
    ProfileTooLong { chars: usize },
}

impl fmt::Display for ProfileParamsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyProfile => f.write_str("an applied profile needs a name"),
            Self::ProfileTooLong { chars } => write!(
                f,
                "the profile name is {chars} characters; the channel allows {MAX_NAME_CHARS}"
            ),
        }
    }
}

impl std::error::Error for ProfileParamsError {}

/// The token of a `confirmation_required` answer.
///
/// It is bound to what was shown: the mode, the profile name, the exact title and
/// message and the exact profile content. A resend carries it unchanged
/// (`desktop-rpc.md:871-877`); a token that is empty or longer than the channel's
/// 128 characters is not one at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmationToken(String);

impl ConfirmationToken {
    pub fn parse(value: &str) -> Option<Self> {
        let chars = value.chars().count();
        (chars > 0 && chars <= MAX_NAME_CHARS).then(|| Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One of the screen's notices, in the order the apply produced it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteLevel {
    Info,
    Warning,
    Error,
}

impl NoteLevel {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "info" => Some(Self::Info),
            "warning" => Some(Self::Warning),
            "error" => Some(Self::Error),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

/// One notice the screen produced while applying.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileNote {
    pub level: NoteLevel,
    pub text: String,
}

/// An apply that ran: `applied` when the screen's last notice was `info`,
/// `not_applied` otherwise (a warning, or no notice, says why in `notes`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileApply {
    pub profile: String,
    pub mode: ApplyMode,
    pub notes: Vec<ProfileNote>,
}

/// The screen's own dialog, which the human must see before anything is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileConfirmation {
    pub profile: String,
    pub mode: ApplyMode,
    pub title: String,
    pub message: String,
    pub token: ConfirmationToken,
}

/// The `ok` answer of one apply, as this contract models it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileAnswer {
    /// The apply ran and the screen's last notice was `info`.
    Applied(ProfileApply),
    /// The apply ran and left a warning or no notice at all.
    NotApplied(ProfileApply),
    /// Nothing was written: the human is asked first, then the request is
    /// resent with [`ProfileConfirmation::token`].
    ConfirmationRequired(ProfileConfirmation),
}

/// Why an `ok` answer is not one this module acts on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileAnswerError {
    /// A member of the answer is not this contract's.
    Field(AnswerError),
    /// A `session` apply answered `confirmation_required`. `session` never asks
    /// (`desktop-rpc.md:880`, L8), so there is no dialog to show and no token to
    /// resend.
    ConfirmationForSession,
}

impl From<AnswerError> for ProfileAnswerError {
    fn from(error: AnswerError) -> Self {
        Self::Field(error)
    }
}

impl fmt::Display for ProfileAnswerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Field(error) => error.fmt(f),
            Self::ConfirmationForSession => f.write_str(
                "a session apply answered confirmation_required, which the session mode never asks",
            ),
        }
    }
}

impl std::error::Error for ProfileAnswerError {}

/// One `apply_profile` attempt: the answer, or the typed reason there is none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyProfileOutcome {
    /// The apply ran with an `info` notice last, and one `profile_applied` event
    /// follows it.
    Applied(ProfileApply),
    /// The apply ran but did not end well: `notes` say why, and a failed write
    /// has already been rolled back by the screen's own path.
    NotApplied(ProfileApply),
    /// Nothing was written; show `title` and `message`, and resend with `token`
    /// only if the human approves.
    ConfirmationRequired(ProfileConfirmation),
    /// No profile by that name is saved. Nothing was written (a missing
    /// `profiles.json` may have been seeded).
    UnknownProfile(DesktopError),
    /// Another apply is running; this one changed nothing.
    Busy(DesktopError),
    /// The profiles owner did not answer for this session.
    ProfilesUnavailable(DesktopError),
    /// Any other refusal this slice does not model (`invalid_request`, ...).
    Rejected(DesktopError),
    /// The child's `capabilities` does not list `apply_profile`.
    Unsupported,
    /// No usable answer: an inactive host, a timeout, a closed child, an
    /// unparsable payload, or a client that never resolved its contract.
    NoAnswer(NoAnswer),
    /// The request could not be built: the profile name the channel would refuse.
    InvalidParams(ProfileParamsError),
    /// The request broke a contract limit and never left.
    InvalidRequest(RequestError),
    /// The child answered `ok`, but not this contract's answer.
    UnexpectedAnswer(ProfileAnswerError),
}

/// The `apply_profile` params: `profile`, `mode`, and `confirmation` only on a
/// repeat.
///
/// Any other key, a malformed value or a mode other than `default` / `session` is
/// `invalid_request` (`desktop-rpc.md:846-853`), so nothing else is written. A
/// first apply never carries a confirmation: the answer is what asks.
pub fn apply_profile_params(
    profile: &str,
    mode: ApplyMode,
    confirmation: Option<&ConfirmationToken>,
) -> Result<Value, ProfileParamsError> {
    let chars = profile.chars().count();
    if chars == 0 {
        return Err(ProfileParamsError::EmptyProfile);
    }
    if chars > MAX_NAME_CHARS {
        return Err(ProfileParamsError::ProfileTooLong { chars });
    }
    let mut params = Map::new();
    params.insert("profile".to_owned(), json!(profile));
    params.insert("mode".to_owned(), json!(mode.as_str()));
    if let Some(token) = confirmation {
        params.insert("confirmation".to_owned(), json!(token.as_str()));
    }
    Ok(Value::Object(params))
}

/// Asks one chat's child to apply `profile`, through a client that keeps that
/// chat's own request ids and cached capabilities.
///
/// A request that cannot be built is [`ApplyProfileOutcome::InvalidParams`] and
/// is never written - not even the `capabilities` probe, because no method of
/// this contract is needed to answer it. Everything else is one round trip per
/// attempt, exactly as the screen's own path does it.
pub fn apply_profile<C: DesktopChannel>(
    client: &mut DesktopClient<C>,
    profile: &str,
    mode: ApplyMode,
    confirmation: Option<&ConfirmationToken>,
) -> ApplyProfileOutcome {
    let params = match apply_profile_params(profile, mode, confirmation) {
        Ok(params) => params,
        Err(error) => return ApplyProfileOutcome::InvalidParams(error),
    };
    if client.capabilities().is_none() {
        client.connect();
    }
    match client.ask(APPLY_PROFILE, params) {
        DesktopOutcome::Data { data, .. } => match parse_apply_profile_answer(&data) {
            Ok(ProfileAnswer::Applied(apply)) => ApplyProfileOutcome::Applied(apply),
            Ok(ProfileAnswer::NotApplied(apply)) => ApplyProfileOutcome::NotApplied(apply),
            Ok(ProfileAnswer::ConfirmationRequired(confirmation)) => {
                ApplyProfileOutcome::ConfirmationRequired(confirmation)
            }
            Err(error) => ApplyProfileOutcome::UnexpectedAnswer(error),
        },
        DesktopOutcome::Rejected { error, .. } => match error.code.as_str() {
            "unknown_profile" => ApplyProfileOutcome::UnknownProfile(error),
            "profile_apply_busy" => ApplyProfileOutcome::Busy(error),
            "profiles_unavailable" => ApplyProfileOutcome::ProfilesUnavailable(error),
            _ => ApplyProfileOutcome::Rejected(error),
        },
        DesktopOutcome::Unsupported { .. } => ApplyProfileOutcome::Unsupported,
        DesktopOutcome::NoAnswer(reason) => ApplyProfileOutcome::NoAnswer(reason),
        DesktopOutcome::Invalid(error) => ApplyProfileOutcome::InvalidRequest(error),
    }
}

/// Reads one `apply_profile` answer (`data` of the response envelope).
///
/// Unknown extra members are ignored; the members this dialect needs - the
/// echoed `profile` and `mode`, the `status`, and, for a dialog, the title,
/// message and token - are typed, and a `confirmation_required` under `session`
/// is refused as the contradiction it is.
pub fn parse_apply_profile_answer(data: &Value) -> Result<ProfileAnswer, ProfileAnswerError> {
    let object = as_object(data, "data")?;
    let profile = required_str(object, "profile")?;
    let mode_name = required_str(object, "mode")?;
    let mode = ApplyMode::parse(&mode_name).ok_or(AnswerError::UnknownValue {
        field: "mode",
        value: mode_name,
    })?;
    let status = required_str(object, "status")?;
    match status.as_str() {
        "applied" => Ok(ProfileAnswer::Applied(ProfileApply {
            profile,
            mode,
            notes: parse_notes(object)?,
        })),
        "not_applied" => Ok(ProfileAnswer::NotApplied(ProfileApply {
            profile,
            mode,
            notes: parse_notes(object)?,
        })),
        "confirmation_required" => {
            if mode == ApplyMode::Session {
                return Err(ProfileAnswerError::ConfirmationForSession);
            }
            let confirmation = required_object(object, "confirmation")?;
            let token = required_str(confirmation, "token")?;
            Ok(ProfileAnswer::ConfirmationRequired(ProfileConfirmation {
                profile,
                mode,
                title: required_str(confirmation, "title")?,
                message: required_str(confirmation, "message")?,
                token: ConfirmationToken::parse(&token).ok_or(AnswerError::UnknownValue {
                    field: "token",
                    value: token,
                })?,
            }))
        }
        other => Err(AnswerError::UnknownValue {
            field: "status",
            value: other.to_owned(),
        }
        .into()),
    }
}

/// The notices an apply produced, in order. An absent list is "no notice", which
/// is one of the two ways an apply does not end `applied` - never a parse error.
fn parse_notes(object: &Map<String, Value>) -> Result<Vec<ProfileNote>, AnswerError> {
    let mut notes = Vec::new();
    for note in optional_array(object, "notes")? {
        let note = as_object(note, "notes")?;
        let level = required_str(note, "level")?;
        notes.push(ProfileNote {
            level: NoteLevel::parse(&level).ok_or(AnswerError::UnknownValue {
                field: "level",
                value: level,
            })?,
            text: required_str(note, "text")?,
        });
    }
    Ok(notes)
}
