//! Windowless tests of the Settings pane's reads through the REAL
//! `Bridge::dispatch` (0.1.12, S1, S2, S3, S5, S7): the profiles file, the auth
//! store, the update check and the orchestrator switch, with the machine behind
//! seams the test answers.
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
use gentle_core::rpc::{decode_line, Decoded};
use gentle_desk_lib::commands::{Bridge, Emit};
use gentle_desk_lib::contract::{
    AuthStatusView, OrchestratorModeView, ProfilesStatusView, ProviderStatusView, UiIntent,
    UpdatesStatusView, ViewState,
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

    /// Writes `body` where gentle-pi keeps its profiles.
    fn write_profiles(&self, body: &str) {
        let path = self.root.join("config").join("profiles.json");
        std::fs::write(path, body).expect("the scratch profiles file is writable");
    }

    /// Writes `body` beside the profiles file: the chains gentle-pi keeps in
    /// `profile-fallbacks.json`, the sibling of `profiles.json`.
    fn write_fallbacks(&self, body: &str) {
        let path = self.root.join("config").join("profile-fallbacks.json");
        std::fs::write(path, body).expect("the scratch chain file is writable");
    }

    /// Writes `body` where the shell keeps its auth store: the agent folder the
    /// home flags select.
    fn write_auth(&self, body: &str) {
        let path = self.root.join("home").join("auth.json");
        std::fs::write(path, body).expect("the scratch auth store is writable");
    }

    /// The app config file, if a run wrote one.
    fn config_file(&self) -> PathBuf {
        self.root
            .join("appdata")
            .join("gentle-desk")
            .join("config.json")
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
    Arc::new(move |provider: &str| {
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
    Arc::new(move || {
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
fn the_profiles_file_reaches_the_pane_and_writes_nothing() {
    let harness = Harness::new();
    harness.write_profiles(
        r#"{
            "kind": "gentle-pi.agent_model_profiles",
            "version": 1,
            "active": "Mine",
            "profiles": {
                "Mine": {
                    "orchestrator": {"model": "prov/big", "thinking": "medium"},
                    "worker": {"model": "prov/small", "thinking": "high"}
                }
            },
            "workflow": {"Mine": {"orchestratorMode": "full-delegation"}}
        }"#,
    );
    let (emit, views) = recorder();
    let bridge = Bridge::new().with_emit(emit);

    let answer = bridge
        .dispatch(UiIntent::RefreshProfiles)
        .expect("the profiles read answers");

    assert_eq!(answer.profiles.status, ProfilesStatusView::Ready);
    assert_eq!(answer.profiles.active.as_deref(), Some("Mine"));
    let profile = &answer.profiles.profiles[0];
    assert!(profile.active);
    assert_eq!(profile.name, "Mine");
    assert_eq!(
        profile
            .orchestrator
            .as_ref()
            .and_then(|role| role.model.as_deref()),
        Some("prov/big")
    );
    assert_eq!(profile.roles.len(), 1);
    assert_eq!(
        profile.mode,
        Some(OrchestratorModeView::FullDelegation),
        "the profile's own workflow mode"
    );
    assert_eq!(
        answer.profiles.mode,
        OrchestratorModeView::Organic,
        "the pane's mode is the session switch, not the file's"
    );
    // The answer is the pushed slice: the page draws what the command returned.
    assert_eq!(lock(&views).last().cloned(), Some(answer));
}

#[test]
fn a_profiles_file_of_another_kind_shows_nothing_and_says_why() {
    let harness = Harness::new();
    harness.write_profiles(
        r#"{"kind": "someone.else", "version": 1, "profiles": {"Mine": {"orchestrator": {}}}}"#,
    );
    let (emit, _views) = recorder();
    let bridge = Bridge::new().with_emit(emit);

    let answer = bridge
        .dispatch(UiIntent::RefreshProfiles)
        .expect("a refused document is an answer, not an error");

    match answer.profiles.status {
        ProfilesStatusView::Failed(reason) => {
            assert!(reason.contains("someone.else"), "{reason}");
        }
        other => panic!("a wrong document must say why, got {other:?}"),
    }
    assert!(
        answer.profiles.profiles.is_empty(),
        "a wrong document draws no profile"
    );
}

#[test]
fn the_fallback_chain_reaches_the_pane_from_the_sibling_file() {
    let harness = Harness::new();
    harness.write_profiles(
        r#"{
            "kind": "gentle-pi.agent_model_profiles",
            "version": 1,
            "active": "Mine",
            "profiles": {
                "Mine": {
                    "orchestrator": {"model": "orx/main", "thinking": "medium"},
                    "worker": {"model": "prov/small", "thinking": "high"}
                }
            }
        }"#,
    );
    harness.write_fallbacks(
        r#"{
            "kind": "gentle-pi.profile_model_fallbacks",
            "version": 1,
            "profiles": {
                "Mine": {
                    "orchestrator": [{"model": "orx/backup", "thinking": "high"}]
                }
            }
        }"#,
    );
    let (emit, _views) = recorder();
    let bridge = Bridge::new().with_emit(emit);

    let answer = bridge
        .dispatch(UiIntent::RefreshProfiles)
        .expect("the profiles read answers");

    let profile = &answer.profiles.profiles[0];
    let fallback = profile
        .roles
        .iter()
        .find(|role| role.role == "fallback")
        .expect("the chain the sibling file records is the row");
    assert_eq!(fallback.model.as_deref(), Some("orx/backup"));
    assert_eq!(fallback.thinking.as_deref(), Some("high"));
    assert_eq!(
        profile.roles.len(),
        2,
        "the profile's own role and the chain"
    );
    assert_eq!(
        answer.profiles.status,
        ProfilesStatusView::Ready,
        "reading the sibling file never fails the pane"
    );
}

#[test]
fn a_broken_fallbacks_file_leaves_the_pane_ready_and_draws_no_row() {
    let harness = Harness::new();
    harness.write_profiles(
        r#"{"kind": "gentle-pi.agent_model_profiles", "version": 1, "active": "Mine", "profiles": {"Mine": {"orchestrator": {"model": "prov/big"}}}}"#,
    );
    harness.write_fallbacks(r#"{"kind": "someone.else", "version": 1, "profiles": {}}"#);
    let (emit, _views) = recorder();
    let bridge = Bridge::new().with_emit(emit);

    let answer = bridge
        .dispatch(UiIntent::RefreshProfiles)
        .expect("a broken chain file is not a broken pane");

    assert_eq!(answer.profiles.status, ProfilesStatusView::Ready);
    assert!(
        answer.profiles.profiles[0]
            .roles
            .iter()
            .all(|role| role.role != "fallback"),
        "a chain document of another kind draws no row"
    );
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
fn the_mode_switch_reaches_the_children_and_is_never_stored() {
    let harness = Harness::new();
    let (emit, views) = recorder();
    let bridge = Bridge::new().with_emit(emit);

    let answer = bridge
        .dispatch(UiIntent::SetOrchestratorMode {
            mode: OrchestratorModeView::FullDelegation,
        })
        .expect("the switch answers");

    assert_eq!(answer.profiles.mode, OrchestratorModeView::FullDelegation);
    assert_eq!(
        lock(&views).last().map(|view| view.profiles.mode),
        Some(OrchestratorModeView::FullDelegation)
    );
    assert_eq!(
        bridge.executor.orchestrator_mode(),
        Some(gentle_backend::chat_host::OrchestratorMode::FullDelegation),
        "the executor's spawner carries the mode to the next children"
    );

    // Session-scoped (S1): nothing was written. The config file is either
    // absent or carries no mode at all.
    let written = std::fs::read_to_string(harness.config_file()).unwrap_or_default();
    for key in ["orchestrator", "full-delegation", "organic"] {
        assert!(!written.contains(key), "the config stored {key}: {written}");
    }
}

#[test]
fn the_mode_switch_starts_over_in_a_new_run() {
    let _harness = Harness::new();
    let (emit, _views) = recorder();
    let first = Bridge::new().with_emit(emit);
    first
        .dispatch(UiIntent::SetOrchestratorMode {
            mode: OrchestratorModeView::FullDelegation,
        })
        .expect("the switch answers");

    let (emit, _views) = recorder();
    let second = Bridge::new().with_emit(emit);

    assert_eq!(
        second.holder.view().profiles.mode,
        OrchestratorModeView::Organic,
        "a new run starts on gentle-pi's own default"
    );
    assert_eq!(second.executor.orchestrator_mode(), None);
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
