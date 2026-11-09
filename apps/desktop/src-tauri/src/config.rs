//! The app config as the bridge holds it: one snapshot, read once at startup.
//!
//! The file is [`gentle_runtime::app_config`]'s
//! (`<roaming app data>/gentle-desk/config.json`). Reading it is disk I/O, so it
//! happens here, once, BEFORE the holder exists and never under a lock. The
//! holder then keeps the snapshot and replaces it as a whole when the config
//! changes.
//!
//! A run with no config path holds the empty config in memory: nothing is read
//! and nothing is ever saved back.

use std::path::Path;

use gentle_runtime::app_config::{self, AppConfig, Loaded};

/// The config of this run, and what the user should be told about the file it
/// came from.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConfigSnapshot {
    pub config: AppConfig,
    /// The reason a file could not be used, as [`ConfigNotice::message`] puts
    /// it: one line of plain text, `None` when the file was read or was not
    /// there.
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

    /// The snapshot of the config file of this run, located from the `APPDATA`
    /// value ([`app_config::config_path`]).
    pub fn from_env() -> Self {
        let path = app_config::config_path(std::env::var_os("APPDATA").as_deref());
        Self::at(path.as_deref())
    }

    fn of(loaded: Loaded) -> Self {
        Self {
            config: loaded.config,
            notice: loaded.notice.map(|notice| notice.message()),
        }
    }
}
