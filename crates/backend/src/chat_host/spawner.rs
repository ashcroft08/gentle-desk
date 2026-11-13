//! Where a chat's child comes from.
//!
//! The host never builds a command line itself: it asks a [`Spawner`] for a
//! child for a [`ChatKey`]. The real implementation is [`ProcessSpawner`],
//! which is the only place that knows a saved chat gets `--session <path>` and
//! a new one gets no session flag at all.

use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::sync::Mutex;

use crate::launcher::{self, Launcher, LauncherEnv, LauncherError};
use crate::process::{RpcProcess, SpawnOptions};

use super::child::Child;
use super::ChatKey;

/// The variable pi's intercom extension reads as the identity of the session it
/// holds. pi allows one process per id, so every chat's child needs its own or
/// the second chat to open makes the first one report "Intercom is offline".
const INTERCOM_ID_VAR: &str = "PI_INTERCOM_STABLE_ID";

/// What every id this spawner hands out starts with, so an id found in pi's
/// warning or in a lock file says where it came from.
const INTERCOM_ID_PREFIX: &str = "gentle-desk-";

/// The variable gentle-pi reads its orchestrator mode from
/// (lib/orchestrator-mode-policy.ts): `organic` or `full-delegation`, with
/// `organic` as its own default when the variable is unset. It is passed to the
/// child and never set in this process, so one chat's switch cannot reach
/// another process.
const ORCHESTRATOR_MODE_VAR: &str = "GENTLE_PI_ORCHESTRATOR_MODE";

/// How the orchestrator runs a turn: the two values gentle-pi accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrchestratorMode {
    /// The orchestrator does the work itself and delegates when it chooses to.
    Organic,
    /// The orchestrator delegates and keeps the conversation to itself.
    FullDelegation,
}

impl OrchestratorMode {
    /// The value the environment variable carries.
    pub fn as_str(self) -> &'static str {
        match self {
            OrchestratorMode::Organic => "organic",
            OrchestratorMode::FullDelegation => "full-delegation",
        }
    }

    /// The mode `name` spells, `None` for anything else: gentle-pi's own two
    /// values, never a third one invented here.
    pub fn parse(name: &str) -> Option<Self> {
        match name.trim() {
            "organic" => Some(OrchestratorMode::Organic),
            "full-delegation" => Some(OrchestratorMode::FullDelegation),
            _ => None,
        }
    }
}

/// The intercom id of `key`: the same chat always gets the same id, two chats
/// get two.
///
/// A saved chat is identified by its session path, the one thing that names the
/// same chat across restarts, so its id is a fixed digest of that path: ASCII
/// and bounded even when the path is neither, and stable for as long as the
/// path is. The digest is FNV-1a written out here on purpose: `DefaultHasher`
/// may change between Rust versions, and an id that drifts would leave a stale
/// lock behind on pi's side.
///
/// A new chat has no path yet and is only unique inside this process, so its id
/// is just its counter. It is the right shape: pi only needs the id to differ
/// from the ids of the children alive beside it, and a fresh chat's child never
/// outlives the new-chat key that names it, because the chat becomes the saved
/// one once its child reports the session file.
pub fn intercom_id(key: &ChatKey) -> OsString {
    let mut id = OsString::from(INTERCOM_ID_PREFIX);
    match key {
        ChatKey::Saved(path) => {
            // The path bytes rather than a display string: two paths that differ
            // only in a byte nobody can print are two chats.
            let digest = fnv1a(b"saved\0", path.as_os_str().as_encoded_bytes());
            id.push(format!("{digest:016x}"));
        }
        ChatKey::New(counter) => id.push(format!("new-{counter}")),
    }
    id
}

/// FNV-1a over `tag` and then `bytes`: a small, fixed digest whose output never
/// changes between builds of this program.
fn fnv1a(tag: &[u8], bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in tag.iter().chain(bytes) {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

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
    /// The orchestrator mode every later child is started with (0.1.12, S1, T4).
    /// Behind a lock because the host holds this spawner through
    /// `Arc<dyn Spawner>` and the switch arrives after it was built; `None` is
    /// gentle-pi's own default (`organic`), so nothing is passed until the user
    /// chooses, and nothing is ever persisted.
    mode: Mutex<Option<OrchestratorMode>>,
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
            mode: Mutex::new(None),
        }
    }

    /// The arguments every child of this spawner gets, before its own: the ones
    /// the shell was started with, so `--isolated`, `--model` and friends reach
    /// every chat.
    pub fn with_args(mut self, args: Vec<OsString>) -> Self {
        self.args = args;
        self
    }

    /// Sets the orchestrator mode every later child is started with. `None` puts
    /// new children back on gentle-pi's default by passing no variable at all.
    ///
    /// Session-scoped by design (S1): nothing is written to the app config, and
    /// no variable is set in this process, so the mode belongs to the children
    /// started from now on and dies with them. A child already running keeps the
    /// mode it was started with, which is what `applies to chats opened from now
    /// on` means.
    pub fn set_orchestrator_mode(&self, mode: Option<OrchestratorMode>) {
        *lock(&self.mode) = mode;
    }

    /// The mode later children are started with.
    pub fn orchestrator_mode(&self) -> Option<OrchestratorMode> {
        *lock(&self.mode)
    }

    /// The options for one chat: the forwarded arguments, then `--session
    /// <path>` for a saved chat and nothing more for a new one, and the folder
    /// the open named as the working folder. Never a session switch: the session
    /// is chosen at spawn.
    ///
    /// The child's environment is this spawner's plus the intercom id of the
    /// chat (see [`intercom_id`]) and, when the user chose one, the orchestrator
    /// mode. One process per intercom id on pi's side, so sharing one id between
    /// children would make a chat lose its intercom the moment a second one
    /// opens. An entry the caller supplied for either variable is replaced,
    /// never duplicated, because a child can only hold one value per name.
    pub fn options_for(&self, key: &ChatKey, folder: Option<&Path>) -> SpawnOptions {
        let mut args = self.args.clone();
        if let ChatKey::Saved(path) = key {
            args.extend([OsString::from("--session"), path.clone().into_os_string()]);
        }
        let mut env = self.env.clone();
        put(
            &mut env,
            (OsString::from(INTERCOM_ID_VAR), intercom_id(key)),
        );
        if let Some(mode) = self.orchestrator_mode() {
            put(
                &mut env,
                (
                    OsString::from(ORCHESTRATOR_MODE_VAR),
                    OsString::from(mode.as_str()),
                ),
            );
        }
        SpawnOptions {
            args,
            env,
            cwd: folder.map(Path::to_path_buf),
        }
    }
}

/// Puts one variable into a child's environment, replacing the entry the caller
/// supplied for that name: a process can only hold one value per variable.
fn put(env: &mut Vec<(OsString, OsString)>, entry: (OsString, OsString)) {
    match env.iter_mut().find(|(name, _)| *name == entry.0) {
        Some(existing) => *existing = entry,
        None => env.push(entry),
    }
}

/// A lock that survives a poisoned mutex: a thread that died holding the mode
/// must not make every later spawn panic.
fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
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
