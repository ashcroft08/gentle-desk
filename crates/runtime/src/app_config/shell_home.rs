//! Reading of the stored home choice (`shell_home` in the config file).
//!
//! LENIENT on purpose, like the theme: the choice is one additive field of a
//! document that also holds the user's workspaces, so a value this version does
//! not know loads as "not chosen yet" instead of turning the whole file into a
//! damaged one. The user is simply asked again.

use gentle_backend::home::HomeChoice;
use serde::{Deserialize, Deserializer};

/// The stored choice, or `None` for an absent, unknown or mistyped value.
pub(crate) fn or_none<'de, D>(deserializer: D) -> Result<Option<HomeChoice>, D::Error>
where
    D: Deserializer<'de>,
{
    let stored = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(stored
        .as_ref()
        .and_then(serde_json::Value::as_str)
        .and_then(HomeChoice::from_name))
}
