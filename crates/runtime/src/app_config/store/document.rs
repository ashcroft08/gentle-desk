//! The document itself: the bytes this app writes and the reading of them.
//!
//! Pure JSON work, no path and no file: the version is always
//! [`CONFIG_VERSION`] on the way out, and anything that is not a document of
//! this version comes back as a [`ConfigProblem`].

use serde::Serialize;

use gentle_backend::home::HomeChoice;

use super::super::config::{AppConfig, WorkspaceEntry, CONFIG_VERSION};
use super::super::theme::Theme;
use super::super::window::WindowState;
use super::ConfigProblem;

/// The document as this app writes it: the version is always
/// [`CONFIG_VERSION`], whatever a caller holds.
#[derive(Serialize)]
struct DocumentRef<'a> {
    version: u32,
    theme: Theme,
    /// Written only once the user chose, so a file of a user who never saw the
    /// welcome screen stays byte-for-byte what older builds wrote.
    #[serde(skip_serializing_if = "Option::is_none")]
    shell_home: Option<HomeChoice>,
    /// How the window was closed, written only once a close recorded it, for the
    /// same reason: a run that never closed a window writes the document older
    /// builds wrote. It reads back leniently through the field's own
    /// deserializer, like the theme and the home.
    #[serde(skip_serializing_if = "Option::is_none")]
    window: Option<WindowState>,
    /// The version the last run of the app was, written only once a run stored
    /// one, for the same reason: a run that never stored one writes the document
    /// older builds wrote.
    #[serde(skip_serializing_if = "Option::is_none")]
    last_run_version: Option<&'a str>,
    workspaces: &'a [WorkspaceEntry],
}

/// The bytes of `config` as this app writes them.
pub(super) fn bytes(config: &AppConfig) -> std::io::Result<Vec<u8>> {
    let document = DocumentRef {
        version: CONFIG_VERSION,
        theme: config.theme,
        shell_home: config.shell_home,
        window: config.window,
        last_run_version: config.last_run_version.as_deref(),
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
