//! The Settings dialog's one read: the shape of the snapshot it draws from, and
//! the rule that a read that fails says so in ITS OWN section instead of failing
//! the whole object.
//!
//! `gather` is driven over the 0.1.12 seams alone: no test here reads a real auth
//! store, starts a shell, runs `node` or reaches the network, so the gathering
//! function is proved on fakes and the machine stays untouched.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use gentle_core::i18n::Language;
use gentle_desk_lib::contract::{
    AuthStatusView, HomeChoiceView, InstallView, LanguageView, NotificationsMode,
    ProviderStatusView, ShellView, ThemeView, UpdatesStatusView, UsageProviderStatusView,
    UsageStatusView,
};
use gentle_desk_lib::settings::{
    FileRead, NodeProbe, ProviderAnswer, ProviderProbe, VersionProbe, WebGet, SHELL_REGISTRY,
};
use gentle_desk_lib::settings_snapshot::{gather, Reads, SettingsConfig, SettingsSnapshot};

/// The npm entry pi's published version lives in, as the Updates read names it.
const REGISTRY: &str = "https://registry.npmjs.org/@earendil-works/pi-coding-agent/latest";

/// The release manifest the desk's own row is compared against.
const MANIFEST: &str = "https://example.invalid/latest.json";

/// The npm entry gentle-shell's published version lives in (0.8.1 S6).
const SHELL_ENTRY: &str = SHELL_REGISTRY;

/// The auth store, as the shell writes it: two providers and four values that
/// must never travel.
const STORE: &str = r#"{
    "anthropic": {"type": "oauth", "access": "sk-SECRET", "refresh": "rt-SECRET"},
    "commandcode": {"type": "api_key", "key": "cc-SECRET"}
}"#;

/// The shell's settings, reduced to the entry that names the sources' package.
const SETTINGS: &str = r#"{"packages":["npm:gentle-pi","git:github.com/o/pi-usage-sources"]}"#;

/// What the sources' one run prints: one provider with a plan and a window.
const DOCUMENT: &str = concat!(
    r#"{"schema":"pi-usage-sources/usage/v1","sources":[{"provider":"commandcode","#,
    r#""status":"ok","plan":"individual goat","limits":[{"name":"command code","#,
    r#""limitReached":false,"windows":[{"label":"5h","usedPercent":4.5,"resetAt":null}]}]}]}"#,
);

/// The agent folder every fake path hangs off: it is joined, never read.
fn agent() -> PathBuf {
    PathBuf::from("agent")
}

/// The folder of the auth store the fake reader answers for.
fn auth_store() -> PathBuf {
    agent().join("auth.json")
}

/// The Settings the dialog already holds when it opens, as the push carries it.
fn config() -> SettingsConfig {
    SettingsConfig {
        theme: ThemeView::Gentle,
        language: Some(LanguageView::Es),
        notifications: NotificationsMode::Off,
        skipped_update: None,
        shell: ShellView {
            pi_path: Some("C:/pi/agent".to_owned()),
            choice: Some(HomeChoiceView::Link),
            ask: false,
            launcher_found: true,
            install: InstallView::Idle,
            restart_needed: false,
        },
        app_version: Some("0.1.9".to_owned()),
        open_chats: 2,
        config_notice: Some("the config file was set aside".to_owned()),
    }
}

/// A file reader over a table keyed by file NAME: the text for a file the table
/// holds, `None` for a store nobody wrote, and a read that failed for everything
/// else. No test touches a real home.
fn store(entries: Vec<(&'static str, Option<&'static str>)>) -> FileRead {
    Arc::new(move |path: &Path, _language: Language| {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        match entries.iter().find(|(held, _)| *held == name) {
            Some((_, text)) => Ok(text.map(str::to_owned)),
            None => Err(format!("{name} could not be read")),
        }
    })
}

/// The store a healthy machine has: the auth store, the shell's settings and the
/// sources' script.
fn machine() -> FileRead {
    store(vec![
        ("auth.json", Some(STORE)),
        ("settings.json", Some(SETTINGS)),
        (
            "usage-json.ts",
            Some("// the sources' headless entry point\n"),
        ),
    ])
}

/// One `auth check` per provider, answering ready to every name it is asked
/// about, and recording the names in order.
fn probe(asked: &Arc<Mutex<Vec<String>>>) -> ProviderProbe {
    let asked = Arc::clone(asked);
    Arc::new(move |provider: &str, _language: Language| {
        lock(&asked).push(provider.to_owned());
        Ok(ProviderAnswer {
            exit_code: 0,
            stdout: format!(r#"{{"status":"ready","provider":"{provider}"}}"#),
        })
    })
}

/// The one run of the sources' script, answering `DOCUMENT` and recording that
/// it ran.
fn node(ran: &Arc<Mutex<usize>>) -> NodeProbe {
    let ran = Arc::clone(ran);
    Arc::new(move |_script: &Path, _language: Language| {
        *lock(&ran) += 1;
        Ok(DOCUMENT.to_owned())
    })
}

/// `gentle-shell --version`, which names both versions in one run.
fn versions(ran: &Arc<Mutex<usize>>) -> VersionProbe {
    let ran = Arc::clone(ran);
    Arc::new(move |_language: Language| {
        *lock(&ran) += 1;
        Ok("gentle-shell 4.7.0\npi 1.5.2\n".to_owned())
    })
}

/// One GET per address, answering the manifest and the registry and recording
/// every URL it was asked for.
fn web(asked: &Arc<Mutex<Vec<String>>>) -> WebGet {
    let asked = Arc::clone(asked);
    Arc::new(move |url: &str| {
        lock(&asked).push(url.to_owned());
        match url {
            REGISTRY => Ok(r#"{"version":"2.0.5"}"#.to_owned()),
            SHELL_ENTRY => Ok(r#"{"version":"4.8.0"}"#.to_owned()),
            MANIFEST => Ok(r#"{"version":"0.2.0"}"#.to_owned()),
            other => Err(format!("{other} did not answer")),
        }
    })
}

/// One load's reads, every seam recording what it was asked.
struct Seams {
    reads: Reads,
    asked: Arc<Mutex<Vec<String>>>,
    node_runs: Arc<Mutex<usize>>,
    version_runs: Arc<Mutex<usize>>,
    web_gets: Arc<Mutex<Vec<String>>>,
}

impl Seams {
    /// The reads of a machine whose every source answers.
    fn healthy() -> Self {
        let asked = Arc::new(Mutex::new(Vec::new()));
        let node_runs = Arc::new(Mutex::new(0));
        let version_runs = Arc::new(Mutex::new(0));
        let web_gets = Arc::new(Mutex::new(Vec::new()));
        Self {
            reads: Reads {
                files: machine(),
                auth: Some(auth_store()),
                agent: Some(agent()),
                probe: probe(&asked),
                versions: versions(&version_runs),
                web: web(&web_gets),
                node: node(&node_runs),
                desk_version: Some("0.1.9".to_owned()),
                desk_manifest: Some(MANIFEST.to_owned()),
                language: Language::En,
            },
            asked,
            node_runs,
            version_runs,
            web_gets,
        }
    }

    /// The same reads, with the file reader replaced.
    fn with_files(mut self, files: FileRead) -> Self {
        self.reads.files = files;
        self
    }

    /// The same reads, with the sources' run replaced.
    fn with_node(mut self, node: NodeProbe) -> Self {
        self.reads.node = node;
        self
    }

    /// The same reads, with the version probe replaced.
    fn with_versions(mut self, versions: VersionProbe) -> Self {
        self.reads.versions = versions;
        self
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

fn snapshot(seams: &Seams) -> SettingsSnapshot {
    gather(config(), &seams.reads)
}

#[test]
fn one_load_answers_every_section_of_the_dialog() {
    let seams = Seams::healthy();

    let read = snapshot(&seams);

    // The half this run already holds travels as it is: the dialog draws the same
    // theme, language, home, version and gate it had before the load answered.
    assert_eq!(read.theme, ThemeView::Gentle);
    assert_eq!(read.language, Some(LanguageView::Es));
    assert_eq!(read.notifications, NotificationsMode::Off);
    assert_eq!(read.shell.pi_path.as_deref(), Some("C:/pi/agent"));
    assert_eq!(read.shell.choice, Some(HomeChoiceView::Link));
    assert_eq!(read.app_version.as_deref(), Some("0.1.9"));
    assert_eq!(read.open_chats, 2);
    assert_eq!(
        read.config_notice.as_deref(),
        Some("the config file was set aside")
    );

    // Account: the store's names, each with its own check's answer.
    assert_eq!(read.auth.status, AuthStatusView::Ready);
    assert_eq!(
        read.auth
            .providers
            .iter()
            .map(|provider| (provider.name.as_str(), provider.status))
            .collect::<Vec<_>>(),
        vec![
            ("anthropic", ProviderStatusView::Ready),
            ("commandcode", ProviderStatusView::Ready),
        ]
    );

    // Usage: the one document the sources printed, reduced to what the pane draws.
    assert_eq!(read.usage.status, UsageStatusView::Installed);
    assert_eq!(read.usage.providers.len(), 1);
    assert_eq!(read.usage.providers[0].provider, "commandcode");
    assert_eq!(read.usage.providers[0].status, UsageProviderStatusView::Ok);
    assert_eq!(
        read.usage.providers[0].plan.as_deref(),
        Some("individual goat")
    );
    assert_eq!(read.usage.providers[0].limits[0].windows[0].label, "5h");
    assert_eq!(
        read.usage.providers[0].limits[0].windows[0].used_percent,
        4.5
    );

    // Updates: the rows the one version run and the two GETs answer.
    assert_eq!(read.updates.status, UpdatesStatusView::Ready);
    assert_eq!(read.updates.desk.latest.as_deref(), Some("0.2.0"));
    assert_eq!(read.updates.desk.update_available, Some(true));
    assert_eq!(read.updates.pi.current.as_deref(), Some("1.5.2"));
    assert_eq!(read.updates.pi.latest.as_deref(), Some("2.0.5"));
    assert_eq!(read.updates.pi.update_available, Some(true));
    // The shell's row, measured against npm's stable entry (0.8.1 S6): 4.7.0
    // against 4.8.0, so the stable upgrade is offered.
    assert_eq!(read.updates.shell.current.as_deref(), Some("4.7.0"));
    assert_eq!(read.updates.shell.latest.as_deref(), Some("4.8.0"));
    assert_eq!(read.updates.shell.update_available, Some(true));
    assert_eq!(read.updates.shell.reason, None);
    // And the load says when it read all this (S8).
    assert!(read.updates.checked_ms.is_some_and(|ms| ms > 0.0));
}

#[test]
fn one_load_asks_every_source_once_and_no_more() {
    let seams = Seams::healthy();

    let _ = snapshot(&seams);

    assert_eq!(
        *lock(&seams.asked),
        vec!["anthropic".to_owned(), "commandcode".to_owned()],
        "one check per provider the store names, in its order"
    );
    assert_eq!(*lock(&seams.node_runs), 1, "the sources run once");
    assert_eq!(*lock(&seams.version_runs), 1, "one version probe");
    assert_eq!(
        *lock(&seams.web_gets),
        vec![
            MANIFEST.to_owned(),
            REGISTRY.to_owned(),
            SHELL_ENTRY.to_owned()
        ],
        "the desk's manifest, pi's registry and the shell's, one GET each"
    );
}

#[test]
fn a_store_that_cannot_be_read_leaves_the_other_sections_whole() {
    // The auth store is missing from the table, so its read fails; the shell's
    // settings and the sources are there.
    let seams = Seams::healthy().with_files(store(vec![
        ("settings.json", Some(SETTINGS)),
        ("usage-json.ts", Some("// the entry point\n")),
    ]));

    let read = snapshot(&seams);

    match read.auth.status {
        AuthStatusView::Failed(reason) => assert!(reason.contains("auth.json"), "{reason}"),
        other => panic!("a store that could not be read must say so, got {other:?}"),
    }
    assert_eq!(read.usage.status, UsageStatusView::Installed);
    assert_eq!(read.usage.providers.len(), 1);
    assert_eq!(read.updates.status, UpdatesStatusView::Ready);
    assert_eq!(read.updates.pi.latest.as_deref(), Some("2.0.5"));
    // The dialog's own half is untouched by a read that failed.
    assert_eq!(read.theme, ThemeView::Gentle);
    assert_eq!(read.app_version.as_deref(), Some("0.1.9"));
}

#[test]
fn a_usage_run_that_fails_says_why_and_leaves_the_other_sections_whole() {
    let failing: NodeProbe = Arc::new(|_script: &Path, _language: Language| {
        Err("could not run (node not found)".to_owned())
    });
    let seams = Seams::healthy().with_node(failing);

    let read = snapshot(&seams);

    assert_eq!(
        read.usage.status,
        UsageStatusView::Failed("could not run (node not found)".to_owned())
    );
    assert_eq!(read.auth.status, AuthStatusView::Ready);
    assert_eq!(read.auth.providers.len(), 2);
    assert_eq!(read.updates.status, UpdatesStatusView::Ready);
    assert_eq!(read.updates.desk.update_available, Some(true));
}

#[test]
fn a_version_probe_that_fails_leaves_only_its_own_rows_unknown() {
    let failing: VersionProbe =
        Arc::new(|_language: Language| Err("the shell could not run".to_owned()));
    let seams = Seams::healthy().with_versions(failing);

    let read = snapshot(&seams);

    // Both rows the one probe fills are unknown, and neither claims an update.
    assert_eq!(read.updates.status, UpdatesStatusView::Ready);
    assert_eq!(read.updates.pi.current, None);
    assert_eq!(read.updates.pi.update_available, None);
    assert_eq!(read.updates.shell.current, None);
    // The desk's own row is a GET, so it is still read.
    assert_eq!(read.updates.desk.latest.as_deref(), Some("0.2.0"));
    // And the other two sections answered in full.
    assert_eq!(read.auth.status, AuthStatusView::Ready);
    assert_eq!(read.usage.status, UsageStatusView::Installed);
}

#[test]
fn no_value_of_the_auth_store_reaches_the_snapshot() {
    let seams = Seams::healthy();
    let read = snapshot(&seams);

    // The wire shape, not just the struct: what the page receives carries names,
    // states and reasons, and never a key, a token or a store type.
    let carried = serde_json::to_string(&read).expect("the snapshot is serializable");
    assert!(!carried.contains("SECRET"), "{carried}");
    assert!(!carried.contains("api_key"), "{carried}");
    assert!(!carried.contains("oauth"), "{carried}");
}

/// The same Settings, with a version the user skipped (0.8.0 settings-polish S1).
fn skipping(version: &str) -> SettingsConfig {
    SettingsConfig {
        skipped_update: Some(version.to_owned()),
        ..config()
    }
}

#[test]
fn one_load_carries_the_skipped_version_and_still_reports_it_as_published() {
    let seams = Seams::healthy();

    let read = gather(skipping("2.0.5"), &seams.reads);

    assert_eq!(read.updates.skipped.as_deref(), Some("2.0.5"));
    assert_eq!(read.updates.pi.latest.as_deref(), Some("2.0.5"));
    assert_eq!(
        read.updates.pi.update_available,
        Some(true),
        "the dialog's own load is an explicit read: it never hides the skipped version"
    );
}

#[test]
fn one_load_of_a_run_that_skips_nothing_carries_no_skip() {
    let read = snapshot(&Seams::healthy());

    assert_eq!(read.updates.skipped, None);
}
