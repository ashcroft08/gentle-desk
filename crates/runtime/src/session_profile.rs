//! Which profile one chat is using, from that chat's own session file (L9).
//!
//! The desktop channel exposes no read for a session's profile binding (L8:
//! `get_config` has six stores, none session-scoped, and `apply_profile`'s
//! `session` answer is ephemeral), so the maintainer's ruling was to read the
//! chat's session JSONL: the LAST `gentle-pi.session-profile/v1` entry says
//! which profile the session bound, and a chat with no such entry (or a terminal
//! `clear`) shows the default.
//!
//! **This is an approximation, by that ruling.** gentle-pi resolves the binding
//! by replaying the session's active BRANCH and corroborating the selected entry
//! against the file (`replaySessionProfileBranch` and
//! `docs/session-profile-format.md:41-60`); none of that is reimplemented here.
//! What this reader does is what the ruling asked for: scan the file text, take
//! the newest entry of the family, read its own kind and name, and let the UI
//! label the answer as an approximation. A session whose newest family entry is
//! on a side branch is therefore read here as if it were on the active one.
//!
//! The entry kinds and payload members follow `docs/session-profile-format.md:9-38`
//! literally: `bind` (origins `user`, `local`, `repo`, `global`) and `clear`
//! (legacy, or `startup: "none"`). The newest family entry is TERMINAL - an
//! unusable one is [`SessionProfileEntry::Invalid`] and never resurrects an older
//! binding - and a malformed line is skipped, never fatal, exactly as Pi's own
//! session parser skips it.
//!
//! Everything here is pure: the caller supplies the text (no IO, no clock), so a
//! 400 MB session file costs what its text costs and nothing is re-read.

use serde_json::{Map, Value};

/// The `customType` of the session-profile family this reader understands.
pub const CUSTOM_TYPE: &str = "gentle-pi.session-profile/v1";

/// The payload kind of a binding.
pub const BIND_KIND: &str = "bind";

/// The payload kind that removes a binding.
pub const CLEAR_KIND: &str = "clear";

/// Which layer a binding came from: the closed set the record's decoder accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionProfileOrigin {
    /// An explicit choice - the screen's "Use in this session".
    User,
    /// The session started with the local pin.
    Local,
    /// The session started with the repo pin.
    Repo,
    /// The session started with the globally active profile.
    Global,
}

impl SessionProfileOrigin {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "user" => Some(Self::User),
            "local" => Some(Self::Local),
            "repo" => Some(Self::Repo),
            "global" => Some(Self::Global),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Local => "local",
            Self::Repo => "repo",
            Self::Global => "global",
        }
    }
}

/// One binding: the profile the session uses, and the layer it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionProfileBind {
    pub name: String,
    pub origin: SessionProfileOrigin,
}

/// What the newest profile-family entry of one session says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionProfileEntry {
    /// No profile-family entry at all: the session never bound one.
    Absent,
    /// A binding, which the UI shows (as an approximation) as this chat's profile.
    Bound(SessionProfileBind),
    /// A terminal clear: no profile, and the default applies.
    Cleared,
    /// An entry of this family this reader cannot use: an unknown kind, or a
    /// `bind` missing a member the record requires. It is terminal, exactly as
    /// the record's own replay says - an older binding is never revived.
    Invalid,
}

/// The profile this session's newest `gentle-pi.session-profile/v1` entry binds,
/// or `None` when there is none left to show (absent, cleared, or unusable).
///
/// This is the one call the pane needs: "which profile does this chat use" is a
/// name or nothing, and "nothing" is the default's business.
pub fn last_session_profile(jsonl_text: &str) -> Option<SessionProfileBind> {
    match last_session_profile_entry(jsonl_text) {
        SessionProfileEntry::Bound(bind) => Some(bind),
        SessionProfileEntry::Absent
        | SessionProfileEntry::Cleared
        | SessionProfileEntry::Invalid => None,
    }
}

/// What the newest profile-family entry of `jsonl_text` says, including the three
/// answers that are not a name.
///
/// Lines are read as text: a line that is not JSON, not an object, or not an entry
/// of this `customType` is skipped, so one broken line never loses the session's
/// profile. A `\r\n` file reads exactly like a `\n` one.
pub fn last_session_profile_entry(jsonl_text: &str) -> SessionProfileEntry {
    let mut terminal = SessionProfileEntry::Absent;
    for line in jsonl_text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let Some(entry) = value.as_object() else {
            continue;
        };
        if entry.get("customType").and_then(Value::as_str) != Some(CUSTOM_TYPE) {
            continue;
        }
        terminal = read_payload(entry.get("data"));
    }
    terminal
}

/// One entry's payload, as the record's own reader classifies it: no `kind` is
/// invalid, `clear` is a clear, `bind` is read member by member, and any other
/// kind is one this reader cannot use.
fn read_payload(data: Option<&Value>) -> SessionProfileEntry {
    let Some(Value::Object(payload)) = data else {
        return SessionProfileEntry::Invalid;
    };
    match payload.get("kind").and_then(Value::as_str) {
        Some(CLEAR_KIND) => SessionProfileEntry::Cleared,
        Some(BIND_KIND) => match read_bind(payload) {
            Some(bind) => SessionProfileEntry::Bound(bind),
            None => SessionProfileEntry::Invalid,
        },
        _ => SessionProfileEntry::Invalid,
    }
}

/// One binding, or `None` when the payload is missing a member the record
/// requires.
///
/// `modelProfiles` must be there and be an object - "known invalid fields reject
/// the whole binding" - but its routes are NOT re-normalized here: that is
/// gentle-pi's routing rule, this reader only wants the name. The name is taken
/// as written; the record's own slug policy is the writer's, and a name it would
/// refuse can only mislabel a pane, never route anything.
fn read_bind(payload: &Map<String, Value>) -> Option<SessionProfileBind> {
    let name = payload.get("name")?.as_str()?;
    if name.is_empty() {
        return None;
    }
    let origin = SessionProfileOrigin::parse(payload.get("origin")?.as_str()?)?;
    payload.get("modelProfiles")?.as_object()?;
    Some(SessionProfileBind {
        name: name.to_owned(),
        origin,
    })
}
