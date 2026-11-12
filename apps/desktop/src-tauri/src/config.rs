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
//! and nothing is ever saved back, and the run says so once
//! (`NO_PATH_NOTICE`), because a user whose settings folder was not found must
//! never be left thinking their settings are being kept.

use std::path::{Path, PathBuf};

use gentle_runtime::app_config::{self, AppConfig, Loaded};

mod writer;

pub use writer::{ConfigWriter, SaveFailure};

/// The one line a run with no config path shows the user: the settings folder
/// could not be found, so nothing is read and nothing is written this run. It
/// stays for the whole run, because the condition does, and it is a notice
/// (`ViewState::config_notice`), never a failure of what the user asked for.
const NO_PATH_NOTICE: &str =
    "The settings folder could not be found, so gentle-desk started with an empty \
     config and your settings will not persist this run.";

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
    /// config when this run has no config path at all.
    ///
    /// The two cases are told apart: a path whose file is not there yet is the
    /// normal first run and says nothing, while no path means the settings
    /// folder could not be found, which the run reports once
    /// ([`NO_PATH_NOTICE`]) and never by failing.
    pub fn at(path: Option<&Path>) -> Self {
        match path {
            Some(path) => Self::of(app_config::load(path)),
            None => Self {
                config: AppConfig::default(),
                notice: Some(NO_PATH_NOTICE.to_owned()),
            },
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
