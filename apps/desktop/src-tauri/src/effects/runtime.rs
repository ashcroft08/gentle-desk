//! The real executor: the app's per-chat commands against `gentle-runtime`.
//!
//! The chat host is built on the FIRST command that needs it, never at startup:
//! a window where nobody opens a chat resolves no launcher and starts no
//! process. `gentle_runtime::startup::start()` is deliberately not used - it
//! also brings up the session index, the watcher and the first refresh pass,
//! which are other slices - so the three parts a chat host needs are assembled
//! here, from the same public pieces the runtime's startup uses.
//!
//! Child events are NOT handled here: the tagged stream is handed to
//! [`EventPump`] the moment the host is built, and the pump's own thread turns
//! them into reducer inputs and pushes the slice. Nothing else reads it.

use std::ffi::OsString;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use gentle_app::ChatCommand;
use gentle_backend::chat_host::{
    ChatHost, Clock, Limits, ProcessSpawner, Spawner, SystemClock, TaggedEvent,
};
use gentle_backend::home::{with_home_flag, HomeEnv};
use gentle_backend::launcher::{self, LauncherEnv};
use gentle_runtime::chat_runtime::{ChatRuntime, Outcome};

use super::ChatExecutor;
use crate::bridge::AppStateHolder;
use crate::close::CLOSING;
use crate::events::EventPump;
use crate::pending::PendingFolders;

/// The message every command of a run without a host answers with, the one the
/// iced shell used when gentle-shell could not be resolved at all.
const NO_HOST: &str = "gentle-shell is not available";

/// Runs chat commands against the real children, building the host the first
/// time one arrives.
pub struct RuntimeExecutor {
    /// The host, built once. `Err` keeps the run's fatal message, so every
    /// later command answers the same way instead of retrying a broken launcher
    /// on each click.
    runtime: OnceLock<Result<Arc<ChatRuntime>, String>>,
    /// Where the child events go: the first build hands it the tagged stream,
    /// which is what starts its thread (the `arm` of startup holds the other
    /// half). This is the lazy start the pump documents.
    pump: Arc<EventPump>,
    /// The folders the workspace intents named, waiting for the open of a chat
    /// (`pending`).
    folders: Arc<PendingFolders>,
    /// Where a launcher that cannot be resolved is reported: the run's one fatal
    /// bar (`bridge::fatal`), which the next push carries.
    holder: Arc<AppStateHolder>,
    /// Set by the close before anything else: from then on no command resolves a
    /// launcher, builds a host or starts a child.
    closed: AtomicBool,
}

impl RuntimeExecutor {
    pub fn new(
        pump: Arc<EventPump>,
        folders: Arc<PendingFolders>,
        holder: Arc<AppStateHolder>,
    ) -> Self {
        Self {
            runtime: OnceLock::new(),
            pump,
            folders,
            holder,
            closed: AtomicBool::new(false),
        }
    }

    /// Stops accepting new work: the close flow's first step, and the reason a
    /// command that arrives while shutting down starts nothing.
    pub fn stop_work(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }

    /// Whether new work is refused.
    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    /// Stops every live child, through the host's own close path: stdin first,
    /// then its 2 s grace, then the whole process tree. Blocking by design (one
    /// grace period per child that lingers), so the close flow runs it off the
    /// window's thread with its own deadline. A host that was never built has
    /// nothing to stop, and one that failed to build has no children.
    pub fn shutdown(&self) {
        if let Some(Ok(runtime)) = self.runtime.get() {
            runtime.shutdown();
        }
    }

    /// The host, built now if this is the first ask, and its event stream handed
    /// to the pump exactly once.
    fn runtime(&self) -> Result<Arc<ChatRuntime>, String> {
        self.runtime
            .get_or_init(|| match build() {
                Ok((runtime, events)) => {
                    let runtime = Arc::new(runtime);
                    self.pump.feed(Arc::clone(&runtime), events);
                    Ok(runtime)
                }
                // A launcher that cannot be resolved makes every chat
                // impossible, so it is the run's fatal bar and not only this
                // chat's notice. Noted once, here, because this resolution is
                // kept: no later command retries a broken launcher.
                Err(message) => {
                    self.holder.note_fatal(message.clone());
                    Err(message)
                }
            })
            .clone()
    }
}

impl ChatExecutor for RuntimeExecutor {
    fn execute(&self, command: ChatCommand) -> Outcome {
        // The gate comes first: a command that arrives while the window is
        // closing answers the closing line without resolving a launcher, building
        // a host or starting a child.
        if self.is_closed() {
            return refuse(command);
        }
        let folder = match &command {
            ChatCommand::OpenProcess { chat } => self.folders.take(chat),
            ChatCommand::Send { .. } => None,
        };
        match self.runtime() {
            Ok(runtime) => runtime.execute_in(command, folder),
            Err(message) => match command {
                ChatCommand::OpenProcess { chat } => Outcome::OpenFailed { chat, message },
                ChatCommand::Send { chat, .. } => Outcome::SendFailed { chat, message },
            },
        }
    }
}

/// The outcome of a command that arrived after the close: the failed open or
/// failed send the reducer already knows, carrying [`CLOSING`].
fn refuse(command: ChatCommand) -> Outcome {
    let message = CLOSING.to_owned();
    match command {
        ChatCommand::OpenProcess { chat } => Outcome::OpenFailed { chat, message },
        ChatCommand::Send { chat, .. } => Outcome::SendFailed { chat, message },
    }
}

/// The host itself: real children, the searcher's limits, and the launcher plus
/// the forwarded arguments of this window.
fn build() -> Result<(ChatRuntime, std::sync::mpsc::Receiver<TaggedEvent>), String> {
    let spawner = spawner()?;
    let host = ChatHost::with_parts(
        Limits::default(),
        Arc::new(spawner) as Arc<dyn Spawner>,
        Arc::new(SystemClock) as Arc<dyn Clock>,
    );
    Ok(ChatRuntime::new(host))
}

/// The spawner of one child per chat, with the launcher this run resolves and
/// the arguments this process was started with (a home flag is added when they
/// name none).
fn spawner() -> Result<ProcessSpawner, String> {
    let args: Vec<OsString> = with_home_flag(
        std::env::args_os().skip(1).collect(),
        &HomeEnv::from_process(),
    );
    let launcher = launcher::resolve(&LauncherEnv::from_process()).map_err(|error| {
        // Logged once, here: a notice saying "not available" is for the user,
        // the reason is for whoever reads the bridge's output.
        eprintln!("{NO_HOST}: {error}");
        NO_HOST.to_owned()
    })?;
    Ok(ProcessSpawner::new(launcher).with_args(args))
}
