//! Process-level tests of the state bridge against the stand-in child.
//!
//! `GENTLE_SHELL_BIN` points at `tools/fake-gentle-shell/fake.mjs`, run as
//! `node <file>` by the launcher, so these tests drive the REAL bridge pieces
//! with no model, no network and no real session files:
//!
//! - [`RuntimeExecutor`] builds its own host from the process environment, and
//!   hands the tagged stream to an [`EventPump`] (`feed`) that was already
//!   given its emit step (`arm`): this is the one lazy start the bridge
//!   documents, and no other test covers the two halves meeting.
//! - A child's events travel the whole way: host, translator, pump, a fold
//!   through `AppStateHolder`, and one pushed [`ViewState`] per event.
//! - `FAKE_SHELL_LOG` proves what the client actually sent (the prompt reached
//!   the child) and what the child did (it started, and it exited).
//!
//! `Bridge::dispatch` needs a `tauri::AppHandle`, so it cannot run here: the
//! loop it runs is reproduced with the public items it uses
//! (`AppStateHolder::dispatch_commands`, `effects::run_one`, `fold_input`, the
//! pump's `Push`), inline instead of on worker threads so the push order is
//! deterministic.
//!
//! Process note: a child started here ends when its stdin ends, and the OS
//! closes that pipe when the test binary exits, so no child outlives the run.
//! [`shutting_the_host_down_leaves_no_child_behind`] proves the shutdown path
//! end to end against a child whose host the test itself owns.

use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use gentle_app::ChatCommand;
use gentle_backend::chat_host::{
    ChatHost, Clock, Limits, ProcessSpawner, Spawner, SystemClock, TaggedEvent,
};
use gentle_backend::home::{with_home_flag, HomeEnv};
use gentle_backend::launcher::{self, LauncherEnv};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::{
    ChatActivityMark, ChatActivityView, MessageRole, MessageView, NoticeLevel, NoticeView,
    UiIntent, ViewState,
};
use gentle_desk_lib::effects::{self, ChatExecutor, RuntimeExecutor};
use gentle_desk_lib::events::{EventPump, Push};
use gentle_desk_lib::pending::PendingFolders;
use gentle_desk_lib::workspace;
use gentle_runtime::chat_runtime::{ChatRuntime, Outcome};

/// The stand-in child, relative to the repository root.
const FAKE: &str = "tools/fake-gentle-shell/fake.mjs";

/// Every wait of these tests: a child takes seconds to speak, never minutes.
const WAIT: Duration = Duration::from_secs(15);

/// How long the pump must stay silent after the host shuts down.
const QUIET: Duration = Duration::from_millis(500);

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("the workspace root is three levels above apps/desktop/src-tauri")
        .to_path_buf()
}

fn fake_shell() -> PathBuf {
    repo_root().join(FAKE)
}

/// The environment is process-global and the tests must not see each other's
/// child, so only one of them runs at a time and each gets its own tempdir.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// One test's isolated world: its own `GENTLE_SHELL_HOME`, `LOCALAPPDATA` and
/// `FAKE_SHELL_LOG`, never the user's home. Dropping it removes all four
/// variables and the whole directory.
struct Harness {
    _serial: MutexGuard<'static, ()>,
    root: PathBuf,
    log: PathBuf,
}

impl Harness {
    /// `None` when `node` is missing: the stand-in cannot run, and a skip with
    /// a printed reason is better than a silent pass.
    fn new() -> Option<Self> {
        if !node_on_path() {
            eprintln!("skipping: node is not on PATH, so the fake gentle-shell cannot run");
            return None;
        }
        let serial = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let root = scratch_root();
        std::fs::create_dir_all(root.join("home")).expect("the scratch home is creatable");
        std::fs::create_dir_all(root.join("localappdata"))
            .expect("the scratch LOCALAPPDATA is creatable");
        let fake = fake_shell();
        assert!(
            fake.is_file(),
            "the stand-in child is missing: {}",
            fake.display()
        );
        let log = root.join("fake.log");
        std::env::set_var("GENTLE_SHELL_BIN", &fake);
        std::env::set_var("GENTLE_SHELL_HOME", root.join("home"));
        std::env::set_var("LOCALAPPDATA", root.join("localappdata"));
        std::env::set_var("FAKE_SHELL_LOG", &log);
        Some(Self {
            _serial: serial,
            root,
            log,
        })
    }

    fn log(&self) -> &Path {
        &self.log
    }

    fn root(&self) -> &Path {
        &self.root
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        for name in [
            "GENTLE_SHELL_BIN",
            "GENTLE_SHELL_HOME",
            "LOCALAPPDATA",
            "FAKE_SHELL_LOG",
        ] {
            std::env::remove_var(name);
        }
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// A fresh directory of the system tempdir, never inside the worktree.
fn scratch_root() -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "gentle-desk-fake-shell-{}-{stamp}",
        std::process::id()
    ))
}

/// Whether the launcher will find `node` on `PATH` (it looks for `node.exe` on
/// Windows, as this does).
fn node_on_path() -> bool {
    let name = if cfg!(windows) { "node.exe" } else { "node" };
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join(name).is_file())
}

/// `commands::Bridge` without the `AppHandle`: the same holder, the same
/// executor and the same pump, with a recorder where the app emits
/// `state-changed`.
struct Bridge {
    holder: Arc<AppStateHolder>,
    executor: Arc<dyn ChatExecutor>,
    pushed: Arc<Mutex<Vec<ViewState>>>,
    folders: Arc<PendingFolders>,
}

impl Bridge {
    /// The bridge of `commands::Bridge::new`, with the pump's emit step armed
    /// exactly as startup arms it.
    fn runtime_executor() -> Self {
        let holder = Arc::new(AppStateHolder::default());
        let pump = Arc::new(EventPump::new(Arc::clone(&holder)));
        let pushed = Arc::new(Mutex::new(Vec::new()));
        arm(&pump, Arc::clone(&pushed));
        let folders = Arc::new(PendingFolders::default());
        Self {
            holder: Arc::clone(&holder),
            executor: Arc::new(RuntimeExecutor::new(
                Arc::clone(&pump),
                Arc::clone(&folders),
                Arc::clone(&holder),
            )),
            pushed,
            folders,
        }
    }

    /// The same bridge over a host the test itself owns, so it can shut that
    /// host down. The executor's host is not reachable through any public item
    /// (`RuntimeExecutor` has no accessor), so the parts it assembles are
    /// assembled here instead: the same launcher resolution, the same
    /// `ChatHost`, the same `ChatRuntime`.
    fn owned_runtime() -> (Self, Arc<ChatRuntime>) {
        let holder = Arc::new(AppStateHolder::default());
        let pump = Arc::new(EventPump::new(Arc::clone(&holder)));
        let pushed = Arc::new(Mutex::new(Vec::new()));
        arm(&pump, Arc::clone(&pushed));
        let (runtime, events) = build_runtime();
        pump.feed(Arc::clone(&runtime), events);
        let bridge = Self {
            holder,
            executor: Arc::new(Direct(Arc::clone(&runtime))),
            pushed,
            folders: Arc::new(PendingFolders::default()),
        };
        (bridge, runtime)
    }

    /// `Bridge::dispatch` minus the emit: fold the intent, push the slice, run
    /// each per-chat command and fold what it came to, pushing each step.
    /// Inline rather than on a worker thread, so the push order is fixed.
    fn dispatch(&self, intent: UiIntent) -> ViewState {
        // The folder a workspace named, taken from the intent before the fold, as
        // `Bridge::dispatch` does; a blank path names none.
        let folder = match &intent {
            UiIntent::NewChatInWorkspace { path } => Some(path.clone()),
            _ => None,
        };
        let (view, commands) = self.holder.dispatch_commands(intent);
        record(&self.pushed, view.clone());
        if let Some(folder) = folder {
            workspace::remember_folder(&self.folders, &commands, &folder);
        }
        for command in commands {
            let ran = effects::run_one(command, self.executor.as_ref());
            for input in ran.inputs {
                record(&self.pushed, self.holder.fold_input(input));
            }
            // `ran.confirm_after` arms the bridge's 20 s net. The child here
            // speaks inside the bounded wait, and a sleeping thread per open
            // would only outlive the test.
        }
        view
    }

    fn pushed(&self) -> Vec<ViewState> {
        lock(&self.pushed).clone()
    }
}

/// The pump's emit step, recording instead of emitting.
fn arm(pump: &EventPump, pushed: Arc<Mutex<Vec<ViewState>>>) {
    let push: Push = Box::new(move |view| record(&pushed, view));
    pump.arm(push);
}

fn record(pushed: &Mutex<Vec<ViewState>>, view: ViewState) {
    lock(pushed).push(view);
}

/// A lock that survives a poisoned mutex: a test that failed while holding one
/// must not turn into a second failure for the next test.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// The same host `RuntimeExecutor` builds, from the same public parts.
fn build_runtime() -> (Arc<ChatRuntime>, Receiver<TaggedEvent>) {
    let args = with_home_flag(
        std::env::args_os().skip(1).collect(),
        &HomeEnv::from_process(),
    );
    let launcher = launcher::resolve(&LauncherEnv::from_process())
        .expect("GENTLE_SHELL_BIN points at the stand-in child");
    let spawner = ProcessSpawner::new(launcher).with_args(args);
    let host = ChatHost::with_parts(
        Limits::default(),
        Arc::new(spawner) as Arc<dyn Spawner>,
        Arc::new(SystemClock) as Arc<dyn Clock>,
    );
    let (runtime, events) = ChatRuntime::new(host);
    (Arc::new(runtime), events)
}

/// The executor of a host the test owns: the same `ChatExecutor` seam, without
/// the lazy `OnceLock`.
struct Direct(Arc<ChatRuntime>);

impl ChatExecutor for Direct {
    fn execute(&self, command: ChatCommand) -> Outcome {
        self.0.execute(command)
    }
}

/// Waits for what a bounded wait is about, failing the test instead of hanging.
fn wait_for<T>(what: &str, mut probe: impl FnMut() -> Option<T>) -> T {
    let deadline = Instant::now() + WAIT;
    loop {
        if let Some(value) = probe() {
            return value;
        }
        assert!(
            Instant::now() < deadline,
            "waited {WAIT:?} for {what} and it never came"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The last pushed slice a predicate holds for, newest first.
fn last_where(
    pushed: &Arc<Mutex<Vec<ViewState>>>,
    mut keep: impl FnMut(&ViewState) -> bool,
) -> Option<ViewState> {
    lock(pushed).iter().rev().find(|view| keep(view)).cloned()
}

/// The stand-in's log, one JSON object per line, as far as it is readable: a
/// line still being written is skipped, never a failure.
fn log_lines(log: &Path) -> Vec<serde_json::Value> {
    let Ok(text) = std::fs::read_to_string(log) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

/// The pids of the children that logged their start, in order.
fn start_pids(log: &Path) -> Vec<u64> {
    log_lines(log)
        .iter()
        .filter(|line| line["event"] == "start")
        .filter_map(|line| line["pid"].as_u64())
        .collect()
}

/// The pid of the child that received the prompt `message`, if one did.
fn prompt_pid(log: &Path, message: &str) -> Option<u64> {
    log_lines(log)
        .iter()
        .find(|line| {
            line["event"] == "command" && line["type"] == "prompt" && line["message"] == message
        })
        .and_then(|line| line["pid"].as_u64())
}

/// Whether that child logged its own exit.
fn exited(log: &Path, pid: u64) -> bool {
    log_lines(log)
        .iter()
        .any(|line| line["event"] == "exit" && line["pid"].as_u64() == Some(pid))
}

#[test]
fn an_opened_chat_reaches_the_child_and_its_echo_is_pushed() {
    let Some(harness) = Harness::new() else {
        return;
    };
    let bridge = Bridge::runtime_executor();

    let opened = bridge.dispatch(UiIntent::OpenNewChat);
    let chat = opened
        .foreground_chat
        .clone()
        .expect("a new chat is the foreground one");

    // The opening mark ends with the fresh child's own first event: the reply
    // to the `get_state` the executor asked for right after the start. That
    // reply can only arrive through the pump the executor fed, so this wait is
    // the arm/feed handshake meeting.
    wait_for("the chat's child to answer its first request", || {
        (bridge.holder.view().activity.is_empty()).then_some(())
    });

    bridge.dispatch(UiIntent::SendPrompt {
        text: "hello".to_owned(),
    });

    let echo = MessageView {
        id: "msg-1".to_owned(),
        role: MessageRole::Assistant,
        text: "echo: hello".to_owned(),
        streaming: false,
        error: None,
    };
    let view = wait_for("the echo to be pushed and the turn to settle", || {
        last_where(&bridge.pushed, |view| {
            view.messages.contains(&echo) && view.messages.len() == 2 && view.activity.is_empty()
        })
    });

    let user = MessageView {
        id: "msg-0".to_owned(),
        role: MessageRole::User,
        text: "hello".to_owned(),
        streaming: false,
        error: None,
    };
    assert_eq!(view.foreground_chat.as_deref(), Some(chat.as_str()));
    assert_eq!(
        view.messages,
        vec![user, echo],
        "the pushed slice carries the turn, oldest first"
    );
    assert_eq!(view.notices, Vec::new());
    assert_eq!(view.activity, Vec::new(), "the turn settled");

    // The pushed state followed the child event by event: the working mark the
    // child's `agent_start` sets reached a slice of its own.
    let working = ChatActivityView {
        chat_id: chat.as_str().to_owned(),
        activity: ChatActivityMark::Working,
    };
    assert!(
        bridge
            .pushed()
            .iter()
            .any(|view| view.activity == vec![working.clone()]),
        "no pushed slice carried the working mark"
    );

    // The prompt reached the child, and the one child that received it is the
    // one the executor started.
    let pid = wait_for("the child to log the prompt", || {
        prompt_pid(harness.log(), "hello")
    });
    assert_eq!(
        start_pids(harness.log()),
        vec![pid],
        "exactly one stand-in child ran for this test"
    );
}

#[test]
fn an_error_turn_ends_in_the_state_the_reducer_defines() {
    let Some(harness) = Harness::new() else {
        return;
    };
    let bridge = Bridge::runtime_executor();

    let opened = bridge.dispatch(UiIntent::OpenNewChat);
    let chat = opened
        .foreground_chat
        .clone()
        .expect("a new chat is the foreground one");
    wait_for("the chat's child to answer its first request", || {
        (bridge.holder.view().activity.is_empty()).then_some(())
    });

    bridge.dispatch(UiIntent::SendPrompt {
        text: "error".to_owned(),
    });

    // What the reducer defines for `stopReason: "error"`: the provider's own
    // text is the message's text AND its error flag carries what the provider
    // said, so the thread can draw it as a failure (T5a). The turn ends, and the
    // extension's own `notify` that closes the stand-in's turn is a notice: the
    // next test proves its level, text and action.
    let expected = vec![
        MessageView {
            id: "msg-0".to_owned(),
            role: MessageRole::User,
            text: "error".to_owned(),
            streaming: false,
            error: None,
        },
        MessageView {
            id: "msg-1".to_owned(),
            role: MessageRole::Assistant,
            text: "Provider says no.".to_owned(),
            streaming: false,
            error: Some("quota exceeded (fake)".to_owned()),
        },
    ];
    let view = wait_for("the failed turn to settle", || {
        last_where(&bridge.pushed, |view| {
            view.messages == expected && view.activity.is_empty()
        })
    });

    assert_eq!(view.foreground_chat.as_deref(), Some(chat.as_str()));
    assert_eq!(view.messages, expected);
    assert_eq!(
        view.notices.len(),
        1,
        "the turn's notify is a notice, not a silent line: {:?}",
        view.notices
    );
    assert_eq!(view.activity, Vec::new(), "the turn settled");

    // So what failed is the stand-in's model, not the bridge: its prompt did
    // reach the child.
    let pid = wait_for("the child to log the prompt", || {
        prompt_pid(harness.log(), "error")
    });
    assert_eq!(start_pids(harness.log()), vec![pid]);
}

#[test]
fn an_error_turns_notify_reaches_the_notices_with_its_level() {
    let Some(harness) = Harness::new() else {
        return;
    };
    let bridge = Bridge::runtime_executor();

    bridge.dispatch(UiIntent::OpenNewChat);
    wait_for("the chat's child to answer its first request", || {
        (bridge.holder.view().activity.is_empty()).then_some(())
    });

    bridge.dispatch(UiIntent::SendPrompt {
        text: "error".to_owned(),
    });

    // The stand-in ends its failing turn with a `warning` notify: the pushed
    // slice the web side renders carries it as a notice of that level, with the
    // extension's own text and no action, so no button is drawn that does
    // nothing.
    let expected = NoticeView {
        id: 0,
        text: "rate limit reached (fake)".to_owned(),
        level: NoticeLevel::Warning,
        action: None,
    };
    wait_for("the extension's notify to become a notice", || {
        last_where(&bridge.pushed, |view| {
            view.notices == vec![expected.clone()]
        })
    });

    // The notice outlives the turn that raised it: the settled slice, the one
    // the window draws when nothing is working, still carries it.
    let view = wait_for("the turn to settle with its notice on screen", || {
        last_where(&bridge.pushed, |view| {
            view.notices == vec![expected.clone()] && view.activity.is_empty()
        })
    });
    assert_eq!(view.notices, vec![expected]);

    // So the notice comes from the child's own line, not from a fault of the
    // bridge: that child received the prompt.
    let pid = wait_for("the child to log the prompt", || {
        prompt_pid(harness.log(), "error")
    });
    assert_eq!(start_pids(harness.log()), vec![pid]);
}

#[test]
fn shutting_the_host_down_leaves_no_child_behind() {
    let Some(harness) = Harness::new() else {
        return;
    };
    let (bridge, runtime) = Bridge::owned_runtime();

    bridge.dispatch(UiIntent::OpenNewChat);
    let pid = wait_for("the child to log its start", || {
        start_pids(harness.log()).first().copied()
    });
    wait_for("the chat's child to answer its first request", || {
        (bridge.holder.view().activity.is_empty()).then_some(())
    });

    let pushed_before = bridge.pushed().len();
    runtime.shutdown();

    // The child is gone: the stand-in writes its own `exit` line as it ends, so
    // the proof comes from the process that stopped, not from this test.
    wait_for("the child to exit", || {
        exited(harness.log(), pid).then_some(())
    });

    // The pump folds that close - an expected stop is not a death, so it leaves
    // no notice - and then has nothing left to read.
    wait_for("the expected close to be folded", || {
        (bridge.pushed().len() > pushed_before).then_some(())
    });
    let settled = bridge.pushed().len();
    std::thread::sleep(QUIET);
    assert_eq!(
        bridge.pushed().len(),
        settled,
        "the pump kept pushing after its host shut down"
    );

    let view = bridge.holder.view();
    assert!(
        view.notices.is_empty(),
        "an expected stop: {:?}",
        view.notices
    );
    assert!(
        view.activity.is_empty(),
        "the chat is idle once its child stops: {:?}",
        view.activity
    );
}

#[test]
fn a_new_chat_in_a_workspace_runs_its_child_in_that_folder() {
    let Some(harness) = Harness::new() else {
        return;
    };
    let repo = harness.root.join("workspace");
    std::fs::create_dir_all(&repo).expect("the workspace folder is creatable");
    let bridge = Bridge::runtime_executor();

    let opened = bridge.dispatch(UiIntent::NewChatInWorkspace {
        path: repo.to_string_lossy().into_owned(),
    });
    let chat = opened
        .foreground_chat
        .clone()
        .expect("a new chat is the foreground one");
    assert!(chat.starts_with("new-"), "got {chat}");

    // The stand-in reports the folder it was started in, and a child only logs
    // its start once its process is really there.
    let started = wait_for("the child to log its start", || {
        log_lines(harness.log())
            .into_iter()
            .find(|line| line["event"] == "start")
    });
    let cwd = started["cwd"]
        .as_str()
        .expect("the start line carries the folder");
    assert_eq!(
        std::fs::canonicalize(cwd).expect("the folder the child reports"),
        std::fs::canonicalize(&repo).expect("the workspace folder"),
        "the child runs in the folder of the workspace"
    );
    assert_eq!(
        start_pids(harness.log()).len(),
        1,
        "one workspace click starts one child"
    );
    assert!(
        bridge.folders.is_empty(),
        "the open consumed the folder it was given"
    );

    wait_for("the chat's child to answer its first request", || {
        (bridge.holder.view().activity.is_empty()).then_some(())
    });
}

#[test]
fn a_workspace_folder_that_is_gone_starts_no_child() {
    let Some(harness) = Harness::new() else {
        return;
    };
    let gone = harness.root.join("gone");
    let bridge = Bridge::runtime_executor();

    bridge.dispatch(UiIntent::NewChatInWorkspace {
        path: gone.to_string_lossy().into_owned(),
    });

    // The host's refusal comes back through the failed-open path the reducer
    // already defines: one error notice naming the chat and its reason, and the
    // opening phase over.
    let view = wait_for("the failed open to be reported", || {
        last_where(&bridge.pushed, |view| !view.notices.is_empty())
    });
    assert_eq!(view.notices.len(), 1, "one notice for one attempt");
    assert_eq!(view.notices[0].level, NoticeLevel::Error);
    assert!(
        view.notices[0].text.contains("no folder at"),
        "the reason names the folder, got {:?}",
        view.notices[0].text
    );
    assert!(
        view.activity.is_empty(),
        "a failed open ends the opening phase"
    );

    // The refusal happens before any child is started, so nothing ran.
    std::thread::sleep(QUIET);
    assert!(
        start_pids(harness.log()).is_empty(),
        "a child ran for a folder that is not there"
    );
    assert!(
        bridge.folders.is_empty(),
        "the entry was taken, so it cannot leak"
    );
}

#[test]
fn an_unresolvable_launcher_is_the_runs_global_banner() {
    // The launcher is resolved on the first command that needs a child. When it
    // cannot be resolved at all, no chat can ever start, so the run says so
    // once, in the bar above everything, instead of only in that chat's notice.
    // The path does not exist, so nothing is ever spawned.
    let Some(harness) = Harness::new() else {
        return;
    };
    let missing = harness.root().join("no-such-gentle-shell");
    std::env::set_var("GENTLE_SHELL_BIN", &missing);
    let bridge = Bridge::runtime_executor();

    bridge.dispatch(UiIntent::OpenNewChat);

    let view = wait_for("the failed open to be reported", || {
        last_where(&bridge.pushed, |view| view.fatal.is_some())
    });
    assert_eq!(
        view.fatal.as_deref(),
        Some("gentle-shell is not available"),
        "the bar says the launcher is not there, never a path or a stack"
    );
    assert!(
        view.notices
            .iter()
            .any(|notice| notice.text.contains("gentle-shell is not available")),
        "the chat that could not start still gets its own notice, as iced did"
    );
    assert!(
        start_pids(harness.log()).is_empty(),
        "a launcher that is not there starts no child"
    );
}
