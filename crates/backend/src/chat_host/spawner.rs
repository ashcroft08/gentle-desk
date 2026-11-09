//! Where a chat's child comes from.
//!
//! The host never builds a command line itself: it asks a [`Spawner`] for a
//! child for a [`ChatKey`]. The real implementation is [`ProcessSpawner`],
//! which is the only place that knows a saved chat gets `--session <path>` and
//! a new one gets no session flag at all.

use std::ffi::OsString;
use std::io;
use std::path::Path;

use crate::launcher::{self, Launcher, LauncherEnv, LauncherError};
use crate::process::{RpcProcess, SpawnOptions};

use super::child::Child;
use super::ChatKey;

/// Starts one child for one chat.
pub trait Spawner: Send + Sync {
    /// Starts the child of `key` in `folder`, or in the folder of this process
    /// when the caller names none.
    fn spawn(&self, key: &ChatKey, folder: Option<&Path>) -> io::Result<Box<dyn Child>>;
}

/// Real children: one launcher for every chat, the forwarded arguments of the
/// shell, `--mode rpc` plus the session flag of the chat being started.
pub struct ProcessSpawner {
    launcher: Launcher,
    /// Arguments every child gets before its own (e.g. `--isolated`).
    args: Vec<OsString>,
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

    /// A spawner with an explicit child environment. Every child starts in the
    /// folder its open names, or in ours when it names none.
    pub fn with_env(launcher: Launcher, env: Vec<(OsString, OsString)>) -> Self {
        Self {
            launcher,
            args: Vec::new(),
            env,
        }
    }

    /// The arguments every child of this spawner gets, before its own: the ones
    /// the shell was started with, so `--isolated`, `--model` and friends reach
    /// every chat.
    pub fn with_args(mut self, args: Vec<OsString>) -> Self {
        self.args = args;
        self
    }

    /// The options for one chat: the forwarded arguments, then `--session
    /// <path>` for a saved chat and nothing more for a new one, and the folder
    /// the open named as the working folder. Never a session switch: the session
    /// is chosen at spawn.
    pub fn options_for(&self, key: &ChatKey, folder: Option<&Path>) -> SpawnOptions {
        let mut args = self.args.clone();
        if let ChatKey::Saved(path) = key {
            args.extend([OsString::from("--session"), path.clone().into_os_string()]);
        }
        SpawnOptions {
            args,
            env: self.env.clone(),
            cwd: folder.map(Path::to_path_buf),
        }
    }
}

impl Spawner for ProcessSpawner {
    fn spawn(&self, key: &ChatKey, folder: Option<&Path>) -> io::Result<Box<dyn Child>> {
        let options = self.options_for(key, folder);
        RpcProcess::spawn(&self.launcher, &options)
            .map(|process| Box::new(process) as Box<dyn Child>)
    }
}

#[cfg(test)]
mod tests;
