//! Where a chat's child comes from.
//!
//! The host never builds a command line itself: it asks a [`Spawner`] for a
//! child for a [`ChatKey`]. The real implementation is [`ProcessSpawner`],
//! which is the only place that knows a saved chat gets `--session <path>` and
//! a new one gets no session flag at all.

use std::ffi::OsString;
use std::io;

use crate::launcher::{self, Launcher, LauncherEnv, LauncherError};
use crate::process::{RpcProcess, SpawnOptions};

use super::child::Child;
use super::ChatKey;

/// Starts one child for one chat.
pub trait Spawner: Send + Sync {
    fn spawn(&self, key: &ChatKey) -> io::Result<Box<dyn Child>>;
}

/// Real children: one launcher for every chat, `--mode rpc` plus the session
/// flag of the chat being started.
pub struct ProcessSpawner {
    launcher: Launcher,
    env: Vec<(OsString, OsString)>,
}

impl ProcessSpawner {
    /// Resolves `gentle-shell` from the process environment (see
    /// [`launcher::resolve`]), so a missing install is a typed error, not a
    /// panic at the first open.
    pub fn resolve() -> Result<Self, LauncherError> {
        Ok(Self::new(launcher::resolve(&LauncherEnv::from_process())?))
    }

    /// A spawner for an already resolved launcher, with the child environment
    /// [`ProcessSpawner::with_env`] documents.
    pub fn new(launcher: Launcher) -> Self {
        Self::with_env(
            launcher,
            vec![(
                // pi's rpc mode only serves extension UI requests with an interactive host.
                "GENTLE_SHELL_INTERACTIVE_HOST".into(),
                "1".into(),
            )],
        )
    }

    /// A spawner with an explicit child environment and working directory of
    /// `None` (the process inherits ours).
    pub fn with_env(launcher: Launcher, env: Vec<(OsString, OsString)>) -> Self {
        Self { launcher, env }
    }

    /// The options for one chat: `--session <path>` for a saved chat, nothing
    /// for a new one. Never a session switch: the session is chosen at spawn.
    pub fn options_for(&self, key: &ChatKey) -> SpawnOptions {
        let args = match key {
            ChatKey::Saved(path) => {
                vec![OsString::from("--session"), path.clone().into_os_string()]
            }
            ChatKey::New(_) => Vec::new(),
        };
        SpawnOptions {
            args,
            env: self.env.clone(),
            cwd: None,
        }
    }
}

impl Spawner for ProcessSpawner {
    fn spawn(&self, key: &ChatKey) -> io::Result<Box<dyn Child>> {
        let options = self.options_for(key);
        RpcProcess::spawn(&self.launcher, &options)
            .map(|process| Box::new(process) as Box<dyn Child>)
    }
}

#[cfg(test)]
mod tests;
