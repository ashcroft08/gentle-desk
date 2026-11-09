//! The app config as the bridge holds it: one snapshot, read once at startup and
//! written back through one writer (`ConfigWriter`).
//!
//! The file is [`gentle_runtime::app_config`]'s
//! (`<roaming app data>/gentle-desk/config.json`). Reading it is disk I/O, so it
//! happens here, once, BEFORE the holder exists and never under a lock. The
//! holder then keeps the snapshot and replaces it as a whole when the config
//! changes.
//!
//! A run with no config path holds the empty config in memory: nothing is read
//! and nothing is ever saved back.

use std::path::{Path, PathBuf};

use gentle_runtime::app_config::{self, AppConfig, Loaded};

mod writer;

pub use writer::{ConfigWriter, SaveFailure};

/// The config of this run, and what the user should be told about the file it
/// came from.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConfigSnapshot {
    pub config: AppConfig,
    /// The reason the config file could not be used or could not be written, as
    /// [`ConfigNotice::message`] puts it for a file that could not be read: one
    /// line of plain text, `None` when the file was read or was not there and
    /// every save worked.
    ///
    /// [`ConfigNotice::message`]: gentle_runtime::app_config::ConfigNotice::message
    pub notice: Option<String>,
}

impl ConfigSnapshot {
    /// The snapshot of the config file at `path` as it was read, or the empty
    /// config when there is no file to use.
    pub fn at(path: Option<&Path>) -> Self {
        match path {
            Some(path) => Self::of(app_config::load(path)),
            None => Self::default(),
        }
    }

    /// The path of the config file of this run, located from the `APPDATA`
    /// value ([`app_config::config_path`]): the one path both the startup read
    /// and the writer use.
    pub fn path_from_env() -> Option<PathBuf> {
        app_config::config_path(std::env::var_os("APPDATA").as_deref())
    }

    /// The snapshot of the config file of this run.
    pub fn from_env() -> Self {
        Self::at(Self::path_from_env().as_deref())
    }

    fn of(loaded: Loaded) -> Self {
        Self {
            config: loaded.config,
            notice: loaded.notice.map(|notice| notice.message()),
        }
    }
}
