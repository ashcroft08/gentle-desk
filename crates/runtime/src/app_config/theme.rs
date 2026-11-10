//! The six themes the app can be set to, named exactly as the stylesheet and
//! the gentle-shell picker name them (`design/system.md`, and the six
//! `data-theme` blocks of `apps/desktop/src/theme.css`).
//!
//! One spelling for three places, so nothing has to be translated: the value
//! stored in the config file, the value the contract carries to the web side,
//! and the label the picker draws.
//!
//! Reading is LENIENT on purpose: the theme is one additive field of a document
//! that also holds the user's workspaces, so a value this version does not know
//! loads as the default instead of turning the whole file into a damaged one
//! (`or_default`). Nothing is ever migrated or moved aside for it.

use serde::{Deserialize, Deserializer, Serialize};

/// The chosen appearance of the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Theme {
    /// The reader's own setting (`prefers-color-scheme`), and the default of a
    /// config file that does not name one.
    #[default]
    #[serde(rename = "system")]
    System,
    #[serde(rename = "dark")]
    Dark,
    /// One of the gentle-shell themes, with its own palette.
    #[serde(rename = "Gentle")]
    Gentle,
    #[serde(rename = "Gentleman-Cute")]
    GentlemanCute,
    #[serde(rename = "Gentleman-Sexy")]
    GentlemanSexy,
    #[serde(rename = "light")]
    Light,
}

impl Theme {
    /// The six themes in the order the picker draws them.
    pub const ALL: [Theme; 6] = [
        Theme::System,
        Theme::Dark,
        Theme::Gentle,
        Theme::GentlemanCute,
        Theme::GentlemanSexy,
        Theme::Light,
    ];

    /// The name the file stores, the contract carries and the picker draws.
    pub fn as_str(self) -> &'static str {
        match self {
            Theme::System => "system",
            Theme::Dark => "dark",
            Theme::Gentle => "Gentle",
            Theme::GentlemanCute => "Gentleman-Cute",
            Theme::GentlemanSexy => "Gentleman-Sexy",
            Theme::Light => "light",
        }
    }

    /// The theme `name` spells, `None` for anything else. Exact: the names are
    /// the ones the stylesheet selects on, so `gentle` names no theme.
    pub fn from_name(name: &str) -> Option<Theme> {
        Theme::ALL.into_iter().find(|theme| theme.as_str() == name)
    }
}

/// Reads the stored theme: one of the six names, or the default for a value
/// this version does not know - an unknown string, or a value of another type -
/// so the rest of the document (the workspaces) survives what this field holds.
pub(crate) fn or_default<'de, D>(deserializer: D) -> Result<Theme, D::Error>
where
    D: Deserializer<'de>,
{
    let stored = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(stored
        .as_ref()
        .and_then(serde_json::Value::as_str)
        .and_then(Theme::from_name)
        .unwrap_or_default())
}
