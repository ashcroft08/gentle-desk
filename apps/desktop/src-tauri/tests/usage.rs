//! Windowless tests of the Usage pane's read through the REAL
//! `Bridge::dispatch`: the shell's own package located in its settings, the
//! one run of its script, and the four states the pane draws.
//!
//! Every test sets its own `APPDATA`, `LOCALAPPDATA`, `PI_CODING_AGENT_DIR`,
//! `GENTLE_SHELL_HOME`, `GENTLE_PI_CONFIG_HOME` and `GENTLE_SHELL_CONFIG_HOME`,
//! so the user's own pi is never read, and node is behind a seam, so no test
//! starts a process. The environment is process-global, so only one test of this
//! file runs at a time.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use gentle_app::{ChatId, Input, Intent, SessionItem};
use gentle_desk_lib::commands::{Bridge, Emit};
use gentle_desk_lib::contract::{UiIntent, UsageProviderStatusView, UsageStatusView, ViewState};
use gentle_desk_lib::settings::NodeProbe;

/// The wait of the read's own thread.
const WAIT: Duration = Duration::from_secs(10);

static ENV_LOCK: Mutex<()> = Mutex::new(());

/// The document the package printed on this machine, verbatim (one no-data
/// provider, two with plans, and one provider without a plan).
const DOCUMENT: &str = r#"{"schema":"pi-usage-sources/usage/v1","generatedAt":1794637790033,"sources":[{"provider":"opencode-go","status":"no-data","fetchedAt":1794637790033,"limits":[]},{"provider":"antigravity","status":"ok","fetchedAt":1794637790033,"limits":[{"name":"Gemini Models","limitReached":false,"windows":[{"label":"week","usedPercent":0,"resetAt":1795242599000,"windowSeconds":604800}]},{"name":"Claude and GPT models","limitReached":false,"windows":[{"label":"week","usedPercent":0,"resetAt":1795242599000,"windowSeconds":604800}]}]},{"provider":"commandcode","status":"ok","plan":"individual goat","fetchedAt":1794637790033,"limits":[{"name":"command code","limitReached":false,"windows":[{"label":"5h","usedPercent":4.6771898400000005,"resetAt":1794644915319,"windowSeconds":18000},{"label":"week","usedPercent":88.272975324,"resetAt":1794727272498,"windowSeconds":604800}]},{"name":"credits","limitReached":false,"windows":[{"label":"billing","usedPercent":44.13765581735515,"resetAt":1796714153000,"windowSeconds":2592000}]}]},{"provider":"claude-bridge","status":"ok","plan":"pro","fetchedAt":1794633348955,"limits":[{"name":"claude","limitReached":false,"windows":[{"label":"5h","usedPercent":21,"resetAt":1794644400000,"windowSeconds":18000},{"label":"week","usedPercent":96,"resetAt":1794794400000,"windowSeconds":604800}]}]}]}"#;

/// The shell's settings as this machine writes them, reduced to the entry that
/// matters.
const SETTINGS: &str =
    r#"{"packages":["npm:gentle-pi","git:github.com/ashcroft08/pi-usage-sources"]}"#;

/// One test's isolated world: its own app data, pi folder and config home, all
/// under a temp directory outside the worktree. Dropping it removes every
/// variable and the whole directory.
struct Harness {
    _serial: MutexGuard<'static, ()>,
    root: PathBuf,
}

impl Harness {
    fn new() -> Self {
        let serial = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let root = scratch_root();
        let agent = root.join("pi").join("agent");
        let config = root.join("config");
        for dir in [
            root.join("appdata"),
            root.join("localappdata"),
            root.join("home"),
            agent.clone(),
            config.clone(),
        ] {
            std::fs::create_dir_all(&dir).expect("the scratch directory is creatable");
        }
        std::env::set_var("APPDATA", root.join("appdata"));
        std::env::set_var("LOCALAPPDATA", root.join("localappdata"));
        std::env::set_var("GENTLE_SHELL_HOME", root.join("home"));
        std::env::set_var("PI_CODING_AGENT_DIR", &agent);
        std::env::set_var("GENTLE_PI_CONFIG_HOME", &config);
        std::env::remove_var("GENTLE_SHELL_CONFIG_HOME");
        Self {
            _serial: serial,
            root,
        }
    }

    /// The agent folder this run resolves, as the pane sees it: the home the
    /// children get (the shell home, since no home flag is passed).
    fn agent(&self) -> PathBuf {
        self.root.join("home")
    }

    /// Writes the shell's settings, the file whose package list names the
    /// sources.
    fn write_settings(&self, body: &str) {
        std::fs::write(self.agent().join("settings.json"), body)
            .expect("the scratch settings file is writable");
    }

    /// Writes the sources' script where pi checks a `git:` package out.
    fn write_script(&self) -> PathBuf {
        let dir = self
            .agent()
            .join("git")
            .join("github.com")
            .join("ashcroft08")
            .join("pi-usage-sources");
        std::fs::create_dir_all(&dir).expect("the scratch checkout is creatable");
        let script = dir.join("usage-json.ts");
        std::fs::write(&script, "// the sources' headless entry point\n")
            .expect("the scratch script is writable");
        script
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        for name in [
            "APPDATA",
            "LOCALAPPDATA",
            "GENTLE_SHELL_HOME",
            "PI_CODING_AGENT_DIR",
            "GENTLE_PI_CONFIG_HOME",
            "GENTLE_SHELL_CONFIG_HOME",
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
    std::env::temp_dir().join(format!("gentle-desk-usage-{}-{stamp}", std::process::id()))
}

/// The emit seam: every slice the bridge pushed, in order.
fn recorder() -> (Emit, Arc<Mutex<Vec<ViewState>>>) {
    let views = Arc::new(Mutex::new(Vec::new()));
    let kept = Arc::clone(&views);
    let emit: Emit = Arc::new(move |view: &ViewState| {
        lock(&kept).push(view.clone());
        Ok(())
    });
    (emit, views)
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// Waits for the pushed slice a predicate holds for, failing instead of hanging.
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

/// A run that answers `answer` and records the script it was asked to run.
fn node(answer: Result<&'static str, &'static str>, asked: &Arc<Mutex<Vec<String>>>) -> NodeProbe {
    let asked = Arc::clone(asked);
    Arc::new(move |script: &std::path::Path| {
        lock(&asked).push(script.display().to_string());
        answer.map(str::to_owned).map_err(str::to_owned)
    })
}

/// The installed slice of the last push that finished the read, waited for.
fn installed(views: &Arc<Mutex<Vec<ViewState>>>) -> ViewState {
    wait_for("the usage read to answer", || {
        lock(views)
            .iter()
            .rev()
            .find(|view| view.usage.status != UsageStatusView::Reading)
            .cloned()
    })
}

#[test]
fn the_sources_reach_the_pane_through_the_intent() {
    let harness = Harness::new();
    harness.write_settings(SETTINGS);
    let script = harness.write_script();
    let asked = Arc::new(Mutex::new(Vec::new()));
    let (emit, views) = recorder();
    let bridge = Bridge::new()
        .with_emit(emit)
        .with_node_probe(node(Ok(DOCUMENT), &asked));

    let answer = bridge
        .dispatch(UiIntent::RefreshUsage)
        .expect("the usage read answers");

    // The click's own answer says the read is running: the window keeps drawing.
    assert_eq!(answer.usage.status, UsageStatusView::Reading);

    let view = installed(&views);
    assert_eq!(view.usage.status, UsageStatusView::Installed);
    assert_eq!(
        lock(&asked).len(),
        1,
        "one run when the user asks, and only with the locator's own script"
    );
    assert_eq!(
        std::path::Path::new(&lock(&asked)[0]),
        script,
        "the run is the checkout pi made under the agent home"
    );

    let names: Vec<&str> = view
        .usage
        .providers
        .iter()
        .map(|provider| provider.provider.as_str())
        .collect();
    assert_eq!(
        names,
        vec!["opencode-go", "antigravity", "commandcode", "claude-bridge"]
    );
    assert_eq!(
        view.usage.providers[0].status,
        UsageProviderStatusView::NoData,
        "a provider without credentials is no data, never 0%"
    );
    assert_eq!(
        view.usage.providers[2].plan.as_deref(),
        Some("individual goat")
    );
    assert_eq!(
        view.usage.providers[3].limits[0].windows[1].used_percent,
        96.0
    );
    assert!(view
        .usage
        .providers
        .iter()
        .flat_map(|provider| provider.limits.iter())
        .flat_map(|limit| limit.windows.iter())
        .all(|window| window.resets_in.is_some()));
}

#[test]
fn a_shell_without_the_package_says_it_is_missing_and_runs_nothing() {
    let harness = Harness::new();
    // No settings file at all: nobody installed anything for this shell.
    let asked = Arc::new(Mutex::new(Vec::new()));
    let (emit, views) = recorder();
    let bridge = Bridge::new()
        .with_emit(emit)
        .with_node_probe(node(Ok(DOCUMENT), &asked));

    bridge
        .dispatch(UiIntent::RefreshUsage)
        .expect("a missing package is an answer");

    let view = installed(&views);
    assert!(matches!(view.usage.status, UsageStatusView::Missing(_)));
    assert!(view.usage.providers.is_empty());
    assert!(
        lock(&asked).is_empty(),
        "nothing is run for a missing package"
    );

    // The checkout the entry names, but not the script: still missing.
    harness.write_settings(SETTINGS);
    bridge
        .dispatch(UiIntent::RefreshUsage)
        .expect("a checkout without the script is an answer");
    let view = installed(&views);
    assert!(matches!(view.usage.status, UsageStatusView::Missing(_)));
    assert!(lock(&asked).is_empty());
}

#[test]
fn a_run_that_failed_says_why_and_draws_no_provider() {
    let harness = Harness::new();
    harness.write_settings(SETTINGS);
    harness.write_script();
    let asked = Arc::new(Mutex::new(Vec::new()));
    let (emit, views) = recorder();
    let bridge = Bridge::new()
        .with_emit(emit)
        .with_node_probe(node(Err("node is not installed"), &asked));

    bridge
        .dispatch(UiIntent::RefreshUsage)
        .expect("a failed run is an answer");

    let view = installed(&views);
    assert_eq!(
        view.usage.status,
        UsageStatusView::Failed("node is not installed".to_owned())
    );
    assert!(view.usage.providers.is_empty());
}

#[test]
fn a_chat_that_is_open_never_holds_the_read() {
    let harness = Harness::new();
    harness.write_settings(SETTINGS);
    harness.write_script();
    let session = harness.root.join("a.jsonl");
    std::fs::write(&session, "{\"type\":\"session_info\"}\n").expect("a scratch session file");
    let path = session.to_string_lossy().into_owned();

    let asked = Arc::new(Mutex::new(Vec::new()));
    let (emit, views) = recorder();
    let bridge = Bridge::new()
        .with_emit(emit)
        .with_node_probe(node(Ok(DOCUMENT), &asked));

    // A chat is open: the Updates pane's gate has something to count.
    let chat = ChatId::of_session(&path);
    bridge
        .holder
        .fold_input(Input::SessionsLoaded(vec![SessionItem {
            path: path.clone(),
            title: "A chat".to_owned(),
            cwd: "/dev/repo".to_owned(),
            modified_ms: 0,
            message_count: 1,
        }]));
    bridge
        .holder
        .fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
    bridge.holder.fold_input(Input::ChatOpened { chat });
    assert_eq!(bridge.holder.view().open_chats, 1);

    bridge
        .dispatch(UiIntent::RefreshUsage)
        .expect("an open chat is no reason to refuse the read");

    let view = installed(&views);
    assert_eq!(
        view.usage.status,
        UsageStatusView::Installed,
        "this section reads nothing from chats, so no chat can block it"
    );
    assert_eq!(view.open_chats, 1, "the slice still reports the open chat");
    assert_eq!(
        view.usage.providers.len(),
        4,
        "the whole document reaches the pane with a chat open"
    );
}
