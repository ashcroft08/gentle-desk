//! Windowless tests of the Settings panes' reads through the REAL
//! `Bridge::dispatch` (0.1.12, S1, S2, S3, S5, S7): the auth store and the update
//! check, with the machine behind seams the test answers.
//!
//! The profiles file and the session-only orchestrator switch were tested here
//! until profiles-editor T2d removed both: no pane reads `profiles.json` any
//! more (the chat's Profile tab reads through that chat's own child,
//! `tests/chat_profile_view.rs`) and the switch is gone with the pane.
//!
//! Every test sets its own `APPDATA`, `LOCALAPPDATA`, `PI_CODING_AGENT_DIR`,
//! `GENTLE_SHELL_HOME`, `GENTLE_PI_CONFIG_HOME` and `GENTLE_SHELL_CONFIG_HOME`,
//! so the user's own pi, config and auth store are never read or written and no
//! test starts a shell or reaches the network. The environment is
//! process-global, so only one test of this file runs at a time.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use gentle_app::{ChatId, Input};
use gentle_core::i18n::Language;
use gentle_core::rpc::{decode_line, Decoded};
use gentle_desk_lib::commands::{Bridge, Emit};
use gentle_desk_lib::contract::{
    AuthStatusView, ProviderStatusView, UiIntent, UpdatesStatusView, ViewState,
};
use gentle_desk_lib::settings::{ProviderAnswer, ProviderProbe, VersionProbe, WebGet};

/// The wait of the update check's own thread.
const WAIT: Duration = Duration::from_secs(10);

/// The npm entry pi's published version lives in.
const REGISTRY: &str = "https://registry.npmjs.org/@earendil-works/pi-coding-agent/latest";

static ENV_LOCK: Mutex<()> = Mutex::new(());

/// One test's isolated world: its own app data, pi folder, gentle-shell home and
/// gentle-pi config home, all under a temp directory outside the worktree.
/// Dropping it removes every variable and the whole directory.
struct Harness {
    _serial: MutexGuard<'static, ()>,
    root: PathBuf,
}

impl Harness {
    fn new() -> Self {
        let serial = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let root = scratch_root();
        let pi = root.join("pi");
        let config = root.join("config");
        for dir in [
            root.join("appdata"),
            root.join("localappdata"),
            root.join("home"),
            pi.clone(),
            config.clone(),
        ] {
            std::fs::create_dir_all(&dir).expect("the scratch directory is creatable");
        }
        std::env::set_var("APPDATA", root.join("appdata"));
        std::env::set_var("LOCALAPPDATA", root.join("localappdata"));
        std::env::set_var("GENTLE_SHELL_HOME", root.join("home"));
        std::env::set_var("PI_CODING_AGENT_DIR", &pi);
        std::env::set_var("GENTLE_PI_CONFIG_HOME", &config);
        std::env::remove_var("GENTLE_SHELL_CONFIG_HOME");
        Self {
            _serial: serial,
            root,
        }
    }

    /// Writes `body` where the shell keeps its auth store: the agent folder the
    /// home flags select.
    fn write_auth(&self, body: &str) {
        let path = self.root.join("home").join("auth.json");
        std::fs::write(path, body).expect("the scratch auth store is writable");
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
    std::env::temp_dir().join(format!(
        "gentle-desk-settings-{}-{stamp}",
        std::process::id()
    ))
}

/// The emit seam of a test: every slice the bridge pushed, in order.
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

/// A check probe that answers what the test says for one provider and records
/// the ids it was asked about.
fn probe(
    answers: Vec<(&'static str, i32, &'static str)>,
    asked: &Arc<Mutex<Vec<String>>>,
) -> ProviderProbe {
    let asked = Arc::clone(asked);
    Arc::new(move |provider: &str, _language: Language| {
        lock(&asked).push(provider.to_owned());
        let answer = answers
            .iter()
            .find(|(name, _, _)| *name == provider)
            .map(|(_, code, line)| (*code, (*line).to_owned()))
            .unwrap_or((-1, String::new()));
        Ok(ProviderAnswer {
            exit_code: answer.0,
            stdout: answer.1,
        })
    })
}

/// The real `gentle-shell --version` output of the user's linked machine.
fn versions(pi: &'static str) -> VersionProbe {
    Arc::new(move |_language: Language| {
        Ok(format!(
            "gentle-shell 4.7.0\npi {pi}\nhome linked C:\\Users\\u\\.pi\\agent\n"
        ))
    })
}

/// One GET that answers the test's bodies and records every URL it was asked.
fn web(
    manifest: Option<&'static str>,
    registry: Option<&'static str>,
    asked: &Arc<Mutex<Vec<String>>>,
) -> WebGet {
    let asked = Arc::clone(asked);
    Arc::new(move |url: &str| {
        lock(&asked).push(url.to_owned());
        let body = if url == REGISTRY { registry } else { manifest };
        body.map(str::to_owned)
            .ok_or_else(|| format!("{url} did not answer"))
    })
}

#[test]
fn the_auth_store_is_read_for_its_names_and_never_its_values() {
    let harness = Harness::new();
    harness.write_auth(
        r#"{
            "anthropic": {"type": "oauth", "access": "sk-ant-SECRET"},
            "commandcode": {"type": "api_key", "key": "cc-SECRET"}
        }"#,
    );
    let asked = Arc::new(Mutex::new(Vec::new()));
    let (emit, views) = recorder();
    let bridge = Bridge::new().with_emit(emit).with_provider_probe(probe(
        vec![
            (
                "anthropic",
                1,
                r#"{"status":"not_ready","provider":"anthropic","reason":"OAuth token expired"}"#,
            ),
            (
                "commandcode",
                0,
                r#"{"status":"ready","provider":"commandcode","authType":"api_key"}"#,
            ),
        ],
        &asked,
    ));

    let answer = bridge
        .dispatch(UiIntent::RefreshAuth)
        .expect("the auth read answers");

    assert_eq!(answer.auth.status, AuthStatusView::Ready);
    assert_eq!(
        *lock(&asked),
        vec!["anthropic".to_owned(), "commandcode".to_owned()],
        "one check per provider, in the store's order"
    );
    assert_eq!(answer.auth.providers.len(), 2);
    assert_eq!(answer.auth.providers[0].name, "anthropic");
    assert_eq!(
        answer.auth.providers[0].status,
        ProviderStatusView::NotReady
    );
    assert_eq!(
        answer.auth.providers[0].reason.as_deref(),
        Some("OAuth token expired")
    );
    assert_eq!(answer.auth.providers[1].status, ProviderStatusView::Ready);

    // What the page receives carries no material from the store.
    let pushed = serde_json::to_string(&lock(&views).last().cloned().expect("a push")).unwrap();
    for secret in ["SECRET", "sk-ant", "cc-", "access", "key"] {
        assert!(!pushed.contains(secret), "the slice carried {secret}");
    }
}

#[test]
fn the_update_check_covers_the_desk_pi_and_never_the_shell() {
    let _harness = Harness::new();
    let asked = Arc::new(Mutex::new(Vec::new()));
    let (emit, views) = recorder();
    let bridge = Bridge::new()
        .with_emit(emit)
        .with_version_probe(versions("1.5.2"))
        .with_web_get(web(
            Some(r#"{"version":"0.1.12","notes":"..."}"#),
            Some(r#"{"name":"@earendil-works/pi-coding-agent","version":"2.0.5"}"#),
            &asked,
        ));
    bridge.holder.set_app_version("0.1.12");

    // A run that named no manifest address can still compare pi; the desk's own
    // row then has nothing to compare and says so by staying unknown.
    bridge.start_updates(None);

    let ready = wait_for("the check to push its versions", || {
        lock(&views)
            .iter()
            .rev()
            .find(|view| view.updates.status == UpdatesStatusView::Ready)
            .cloned()
    });
    assert_eq!(ready.updates.desk.current.as_deref(), Some("0.1.12"));
    assert_eq!(ready.updates.desk.latest, None);
    assert_eq!(ready.updates.desk.update_available, None);
    // The user's verified pending update: pi 1.5.2 against 2.0.5.
    assert_eq!(ready.updates.pi.current.as_deref(), Some("1.5.2"));
    assert_eq!(ready.updates.pi.latest.as_deref(), Some("2.0.5"));
    assert_eq!(ready.updates.pi.update_available, Some(true));
    // gentle-shell: its version is known, its channel is not, so no claim.
    assert_eq!(ready.updates.shell.current.as_deref(), Some("4.7.0"));
    assert_eq!(ready.updates.shell.latest, None);
    assert_eq!(ready.updates.shell.update_available, None);
    assert_eq!(
        *lock(&asked),
        vec![REGISTRY.to_owned()],
        "one GET: the manifest address was not named, so nothing else was asked"
    );
}

#[test]
fn the_check_runs_once_when_the_app_opens() {
    let _harness = Harness::new();
    let asked = Arc::new(Mutex::new(Vec::new()));
    let (emit, views) = recorder();
    let bridge = Bridge::new()
        .with_emit(emit)
        .with_version_probe(versions("1.5.2"))
        .with_web_get(web(
            Some(r#"{"version":"0.1.13"}"#),
            Some(r#"{"version":"2.0.5"}"#),
            &asked,
        ));
    bridge.holder.set_app_version("0.1.12");
    assert_eq!(
        bridge.holder.view().updates.status,
        UpdatesStatusView::Unknown,
        "a run that did not open yet has checked nothing"
    );

    // The step the setup runs when the window opens: one check, on its own
    // thread, from the endpoint the updater names.
    bridge.start_updates(Some("https://releases.test/latest.json"));

    let ready = wait_for("the opening check to push its versions", || {
        lock(&views)
            .iter()
            .rev()
            .find(|view| view.updates.status == UpdatesStatusView::Ready)
            .cloned()
    });
    assert_eq!(
        ready.updates.desk.update_available,
        Some(true),
        "the manifest announces 0.1.13 and the run is 0.1.12"
    );
    assert_eq!(ready.updates.pi.update_available, Some(true));
    assert_eq!(
        *lock(&asked),
        vec![
            "https://releases.test/latest.json".to_owned(),
            REGISTRY.to_owned()
        ]
    );

    // "Check again" asks the same two addresses again, and its answer to the
    // click says the check is running.
    let answer = bridge
        .dispatch(UiIntent::CheckUpdates)
        .expect("the check starts again");
    assert_eq!(answer.updates.status, UpdatesStatusView::Checking);
    wait_for("the second check to answer", || {
        lock(&views)
            .iter()
            .filter(|view| view.updates.status == UpdatesStatusView::Ready)
            .count()
            .ge(&2)
            .then_some(())
    });
    assert_eq!(lock(&asked).len(), 4, "two checks, two addresses each");
}

#[test]
fn the_context_reading_of_the_foreground_chat_reaches_the_slice() {
    let _harness = Harness::new();
    let (emit, _views) = recorder();
    let bridge = Bridge::new().with_emit(emit);
    let chat = ChatId::of_session("C:/pi/sessions/a.jsonl");
    bridge
        .holder
        .fold_input(Input::Intent(gentle_app::Intent::OpenChat(chat.clone())));
    assert_eq!(
        bridge.holder.view().context_usage,
        Default::default(),
        "a chat that has not answered yet has no reading, and no zero"
    );

    let Decoded::Event(event) = decode_line(
        "{\"type\":\"response\",\"command\":\"get_session_stats\",\"success\":true,\
         \"data\":{\"contextUsage\":{\"tokens\":38000,\"contextWindow\":200000,\"percent\":19.0}}}",
    ) else {
        panic!("the fixture is an event");
    };
    let view = bridge.holder.fold_input(Input::ChatEvent { chat, event });

    assert_eq!(view.context_usage.tokens, Some(38_000));
    assert_eq!(view.context_usage.percent, Some(19.0));
    assert_eq!(view.context_usage.window, Some(200_000));
}
