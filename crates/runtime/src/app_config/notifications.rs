//! The one persisted notifications setting (notifications S4): whether the app
//! says so, with a tone and a system toast, when it is not in front.
//!
//! Two values, named the way the config file stores them and the contract
//! carries them (`"when-unfocused"`, `"off"`), so no side translates a mode.
//!
//! Reading is LENIENT on purpose, like the theme: the setting is one additive
//! field of a document that also holds the user's workspaces, so a value this
//! version does not know loads as the default instead of turning the whole file
//! into a damaged one (`or_default`). Nothing is ever migrated or moved aside
//! for it, and `CONFIG_VERSION` does not move.

use serde::{Deserialize, Deserializer, Serialize};

/// When the app notifies the user about a chat that needs them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Notifications {
    /// A tone and a system notification, only while the window is not in front.
    /// The default of a run that chose nothing (the maintainer's proposal, L3).
    #[default]
    #[serde(rename = "when-unfocused")]
    WhenUnfocused,
    /// Nothing: no tone, no system notification, whatever the window is doing.
    #[serde(rename = "off")]
    Off,
}

impl Notifications {
    /// The two modes in the order the Settings pane draws them.
    pub const ALL: [Notifications; 2] = [Notifications::WhenUnfocused, Notifications::Off];

    /// The name the file stores and the contract carries.
    pub fn as_str(self) -> &'static str {
        match self {
            Notifications::WhenUnfocused => "when-unfocused",
            Notifications::Off => "off",
        }
    }

    /// The mode `name` spells, `None` for anything else. Exact: the names are
    /// the stored ones, so `WhenUnfocused` names no mode.
    pub fn from_name(name: &str) -> Option<Notifications> {
        Notifications::ALL
            .into_iter()
            .find(|mode| mode.as_str() == name)
    }
}

/// Reads the stored mode: one of the two names, or the default for a value this
/// version does not know - an unknown string, or a value of another type - so
/// the rest of the document (the workspaces) survives what this field holds.
pub(crate) fn or_default<'de, D>(deserializer: D) -> Result<Notifications, D::Error>
where
    D: Deserializer<'de>,
{
    let stored = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(stored
        .as_ref()
        .and_then(serde_json::Value::as_str)
        .and_then(Notifications::from_name)
        .unwrap_or_default())
}
