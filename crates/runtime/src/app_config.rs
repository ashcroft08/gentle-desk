//! The app config file: the chosen theme, the folders the user added as
//! workspaces, and whether each of them is collapsed in the sidebar.
//!
//! This is the app's own data, not a cache: it lives under the roaming app data
//! directory ([`config_path`]) and is never rebuilt from the disk on its own.
//! The session index cache stays a speedup that can be deleted
//! (`crate::cache_path`).
//!
//! Four separate responsibilities, so none of them mixes I/O with a rule:
//!
//! - [`config_path`]: where the file lives, from the environment value.
//! - [`AppConfig`]: the document and its pure edits (`set_theme`,
//!   `add_workspace`, `set_collapsed`), and the folder identity two spellings
//!   share ([`folder_key`]).
//! - [`load`] and [`save`]: the file itself, with an atomic write. A file this
//!   app does not understand is never overwritten: it is renamed aside and the
//!   reason comes back as a [`ConfigNotice`] for the caller to show.
//! - [`workspace_status`]: whether the folder of a workspace is there right
//!   now. It touches the file system, so it is called outside any lock and is
//!   never part of a load or a save.

mod config;
mod paths;
mod shell_home;
mod status;
mod store;
mod theme;

pub(crate) use config::canonical_path;
pub use config::{folder_key, AppConfig, WorkspaceEntry, CONFIG_VERSION};
pub use paths::config_path;
pub use status::{workspace_status, WorkspaceStatus};
pub use store::{load, save, ConfigNotice, ConfigProblem, Loaded};
pub use theme::Theme;
