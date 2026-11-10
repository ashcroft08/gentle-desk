//! The document itself: the bytes this app writes and the reading of them.
//!
//! Pure JSON work, no path and no file: the version is always
//! [`CONFIG_VERSION`] on the way out, and anything that is not a document of
//! this version comes back as a [`ConfigProblem`].

use serde::Serialize;

use super::super::config::{AppConfig, WorkspaceEntry, CONFIG_VERSION};
use super::super::theme::Theme;
use super::ConfigProblem;

/// The document as this app writes it: the version is always
/// [`CONFIG_VERSION`], whatever a caller holds.
#[derive(Serialize)]
struct DocumentRef<'a> {
    version: u32,
    theme: Theme,
    workspaces: &'a [WorkspaceEntry],
}

/// The bytes of `config` as this app writes them.
pub(super) fn bytes(config: &AppConfig) -> std::io::Result<Vec<u8>> {
    let document = DocumentRef {
        version: CONFIG_VERSION,
        theme: config.theme,
        workspaces: &config.workspaces,
    };
    serde_json::to_vec(&document)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
}

/// The config in `data`, or why it is not a document this app writes.
pub(super) fn parse(data: &[u8]) -> Result<AppConfig, ConfigProblem> {
    let value: serde_json::Value =
        serde_json::from_slice(data).map_err(|_| ConfigProblem::Damaged)?;
    let version = value
        .get("version")
        .and_then(serde_json::Value::as_u64)
        .ok_or(ConfigProblem::Damaged)?;
    if version != u64::from(CONFIG_VERSION) {
        return Err(ConfigProblem::UnknownVersion { found: version });
    }
    serde_json::from_value(value).map_err(|_| ConfigProblem::Damaged)
}
