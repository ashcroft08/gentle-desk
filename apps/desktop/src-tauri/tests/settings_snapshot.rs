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
    AuthStatusView, GeneralView, HomeChoiceView, InstallView, LanguageView, NotificationsMode,
    PackagesView, ProviderStatusView, ShellView, ThemeView, ToolKind, ToolState, UpdatesStatusView,
    UsageProviderStatusView, UsageStatusView,
};
use gentle_desk_lib::settings::{
    FileRead, NodeProbe, ProviderAnswer, ProviderProbe, ToolAnswer, ToolProbe, VersionProbe,
    WebGet, SHELL_REGISTRY,
};
use gentle_desk_lib::settings_snapshot::{gather, Reads, SettingsConfig, SettingsSnapshot};
use gentle_runtime::chat_runtime::{PackagesItem, PackagesRead, PackagesState, PackagesStatus};

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

/// The environment check's two bounded probes (general T2, S3): what `node
/// --version` and `git --version` answered, and every tool the load asked about
/// in the order it asked.
fn tools(asked: &Arc<Mutex<Vec<ToolKind>>>, node: ToolAnswer, git: ToolAnswer) -> ToolProbe {
    let asked = Arc::clone(asked);
    Arc::new(move |tool: ToolKind, _language: Language| {
        lock(&asked).push(tool);
        match tool {
            ToolKind::Node => node.clone(),
            ToolKind::Git => git.clone(),
            // S3: the shell's and pi's versions are the Updates rows' own, so a
            // load that spawns them again is the bug this fake refuses.
            other => panic!("the environment check must not spawn {other:?} again"),
        }
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

/// The extensions read of one load (packages-status T1): the answer a test names,
/// and the number of times the load asked for it.
pub type PackagesProbe = Arc<dyn Fn() -> PackagesRead + Send + Sync>;

/// A packages seam answering `read` and recording every ask.
fn packages(read: PackagesRead, asked: &Arc<Mutex<usize>>) -> PackagesProbe {
    let asked = Arc::clone(asked);
    Arc::new(move || {
        *lock(&asked) += 1;
        read.clone()
    })
}

/// The one open chat's child, reporting one extension behind and one up to date.
fn behind() -> PackagesRead {
    PackagesRead::Read(PackagesStatus {
        available: true,
        items: vec![
            PackagesItem {
                name: "gentle-pi".to_owned(),
                current: Some("4.9.0".to_owned()),
                state: PackagesState::UpdateAvailable,
            },
            PackagesItem {
                name: "pi-usage-sources".to_owned(),
                current: Some("0.2.0".to_owned()),
                state: PackagesState::NoUpdateReported,
            },
        ],
        truncated: false,
        checked_ms: Some(1_700_000_000_000.0),
    })
}

/// One load's reads, every seam recording what it was asked.
struct Seams {
    reads: Reads,
    asked: Arc<Mutex<Vec<String>>>,
    node_runs: Arc<Mutex<usize>>,
    version_runs: Arc<Mutex<usize>>,
    web_gets: Arc<Mutex<Vec<String>>>,
    packages_asked: Arc<Mutex<usize>>,
    tools_asked: Arc<Mutex<Vec<ToolKind>>>,
}

impl Seams {
    /// The reads of a machine whose every source answers.
    fn healthy() -> Self {
        let asked = Arc::new(Mutex::new(Vec::new()));
        let node_runs = Arc::new(Mutex::new(0));
        let version_runs = Arc::new(Mutex::new(0));
        let web_gets = Arc::new(Mutex::new(Vec::new()));
        let packages_asked = Arc::new(Mutex::new(0));
        let tools_asked = Arc::new(Mutex::new(Vec::new()));
        Self {
            reads: Reads {
                files: machine(),
                auth: Some(auth_store()),
                agent: Some(agent()),
                probe: probe(&asked),
                versions: versions(&version_runs),
                web: web(&web_gets),
                node: node(&node_runs),
                tools: tools(
                    &tools_asked,
                    ToolAnswer::Installed("v24.11.1".to_owned()),
                    ToolAnswer::Installed("2.51.0".to_owned()),
                ),
                packages: packages(behind(), &packages_asked),
                desk_version: Some("0.1.9".to_owned()),
                desk_manifest: Some(MANIFEST.to_owned()),
                language: Language::En,
            },
            asked,
            node_runs,
            version_runs,
            web_gets,
            packages_asked,
            tools_asked,
        }
    }

    /// The same reads, with the environment check's two probes replaced.
    fn with_tools(mut self, node: ToolAnswer, git: ToolAnswer) -> Self {
        self.reads.tools = tools(&self.tools_asked, node, git);
        self
    }

    /// The same reads, with the open chat's answer replaced (a run with no chat
    /// open asks nothing at all).
    fn with_packages(mut self, read: PackagesRead) -> Self {
        self.reads.packages = packages(read, &self.packages_asked);
        self
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
    // The Extensions block (packages-status T1, S1): the one chat's own child
    // reported two extensions, and only the one pi marks is behind.
    assert!(read.updates.packages.available);
    assert!(!read.updates.packages.unknown);
    assert!(!read.updates.packages.truncated);
    assert_eq!(read.updates.packages.checked, Some(1_700_000_000_000.0));
    assert_eq!(
        read.updates
            .packages
            .behind
            .iter()
            .map(|behind| (behind.name.as_str(), behind.current.as_deref()))
            .collect::<Vec<_>>(),
        vec![("gentle-pi", Some("4.9.0"))]
    );
    // And the load says when it read all this (S8).
    assert!(read.updates.checked_ms.is_some_and(|ms| ms > 0.0));
}

#[test]
fn the_extensions_are_read_once_and_carried_into_the_snapshot() {
    let seams = Seams::healthy();

    let read = snapshot(&seams);

    assert_eq!(
        *lock(&seams.packages_asked),
        1,
        "one read per dialog open, and never a second one (S2)"
    );
    assert_eq!(read.updates.packages.behind.len(), 1);
}

#[test]
fn a_load_with_no_chat_to_ask_carries_the_honest_block_and_reads_nothing() {
    // S3 and S5: nothing was asked, so the snapshot carries that VALUE - no
    // absence, and never a shape that reads as "nothing is behind".
    let seams = Seams::healthy().with_packages(PackagesRead::Unasked);

    let read = snapshot(&seams);

    assert_eq!(read.updates.packages, PackagesView::unasked());
    assert!(!read.updates.packages.available, "nothing was read");
    assert!(
        !read.updates.packages.unknown,
        "the block draws its own sentence here, not a failed read (S3)"
    );
    assert!(read.updates.packages.behind.is_empty());
    assert_eq!(read.updates.packages.checked, None);
    assert_eq!(*lock(&seams.packages_asked), 1, "the load still asked once");
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
        *lock(&seams.packages_asked),
        1,
        "the extensions are read once, like every other source"
    );
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

/// One load's environment check, as the page draws it: (tool, state, version).
fn check_rows(view: &GeneralView) -> Vec<(ToolKind, ToolState, Option<String>)> {
    view.tools
        .iter()
        .map(|row| (row.tool, row.state, row.version.clone()))
        .collect()
}

/// The same Settings, on a machine whose launcher does not resolve.
fn without_launcher() -> SettingsConfig {
    SettingsConfig {
        shell: ShellView {
            launcher_found: false,
            ..config().shell
        },
        ..config()
    }
}

/// general T2, S2, S3: the environment check rides the snapshot, and its four
/// rows come from the versions read the Updates rows already made plus the two
/// bounded probes.
#[test]
fn the_environment_check_carries_its_four_rows_in_the_tools_order() {
    let seams = Seams::healthy();

    let read = snapshot(&seams);

    assert_eq!(
        check_rows(&read.general),
        vec![
            (
                ToolKind::GentleShell,
                ToolState::Installed,
                Some("4.7.0".to_owned())
            ),
            (ToolKind::Pi, ToolState::Installed, Some("1.5.2".to_owned())),
            (
                ToolKind::Node,
                ToolState::Installed,
                Some("v24.11.1".to_owned())
            ),
            (
                ToolKind::Git,
                ToolState::Installed,
                Some("2.51.0".to_owned())
            ),
        ],
        "one row per tool, the shell first and git last"
    );
    // S4: the page's last-checked line is the same instant the Updates rows were
    // worded from, read once.
    assert_eq!(read.general.checked_ms, read.updates.checked_ms);
    assert!(read.general.checked_ms.is_some_and(|ms| ms > 0.0));
}

/// general T2, S3: the shell's and pi's rows are the Updates read's own versions,
/// so a load asks `node` and `git` once each and never the shell again.
#[test]
fn the_environment_check_asks_node_and_git_once_and_never_the_shell_again() {
    let seams = Seams::healthy();

    let read = snapshot(&seams);

    assert_eq!(
        *lock(&seams.tools_asked),
        vec![ToolKind::Node, ToolKind::Git],
        "the two bounded probes, in the order the rows are drawn"
    );
    assert_eq!(
        *lock(&seams.version_runs),
        1,
        "THE one `gentle-shell --version` run answers both pages"
    );
    assert_eq!(read.general.tools[0].version.as_deref(), Some("4.7.0"));
    assert_eq!(read.general.tools[1].version.as_deref(), Some("1.5.2"));
}

/// general T2, S2: with no launcher the shell and pi are MISSING - the one fact
/// that says this machine does not have them - while node's own probe still
/// answers for itself.
#[test]
fn a_machine_without_the_launcher_is_missing_and_leaves_the_other_rows_alone() {
    let seams = Seams::healthy();
    let failing: VersionProbe =
        Arc::new(|_language: Language| Err("gentle-shell launcher not found".to_owned()));
    let seams = seams.with_versions(failing);

    let read = gather(without_launcher(), &seams.reads);

    assert_eq!(read.general.tools[0].state, ToolState::Missing);
    assert_eq!(read.general.tools[0].version, None);
    assert_eq!(read.general.tools[1].state, ToolState::Missing);
    assert_eq!(read.general.tools[2].state, ToolState::Installed);
    assert_eq!(read.general.tools[3].state, ToolState::Installed);
    // The Updates rows still say what they could not read, in their own words
    // (S5): the environment check never rewrites them.
    assert_eq!(read.updates.shell.current, None);
    assert_eq!(read.auth.status, AuthStatusView::Ready);
    assert_eq!(read.usage.status, UsageStatusView::Installed);
}

/// S5: a shell that is there and printed nothing parseable is UNREADABLE - a
/// different fact from a missing shell - and pi keeps whatever the run said about
/// it.
#[test]
fn a_shell_that_printed_nothing_readable_is_unreadable_and_not_missing() {
    let seams = Seams::healthy();
    let nothing: VersionProbe =
        Arc::new(|_language: Language| Ok("home linked C:\\Users\\u\\.pi\\agent\n".to_owned()));
    let seams = seams.with_versions(nothing);

    let read = snapshot(&seams);

    assert_eq!(read.general.tools[0].state, ToolState::Unreadable);
    assert_eq!(read.general.tools[0].version, None);
    assert_eq!(read.general.tools[1].state, ToolState::Unreadable);
}

/// S6: git's row exists whatever its probe answers, and a missing git is that
/// row's state - nothing else of the snapshot takes a different path.
#[test]
fn a_git_that_is_not_there_is_only_its_own_row() {
    let healthy = snapshot(&Seams::healthy());
    let seams = Seams::healthy().with_tools(
        ToolAnswer::Installed("v24.11.1".to_owned()),
        ToolAnswer::Missing,
    );

    let read = snapshot(&seams);

    assert_eq!(read.general.tools.len(), 4, "the row exists anyway");
    assert_eq!(read.general.tools[3].tool, ToolKind::Git);
    assert_eq!(read.general.tools[3].state, ToolState::Missing);
    assert_eq!(read.general.tools[3].version, None);
    assert_eq!(read.general.tools[2].state, ToolState::Installed);
    // Nothing else moved: no section reads git, so nothing else can see it.
    assert_eq!(read.auth, healthy.auth);
    assert_eq!(read.usage, healthy.usage);
    assert_eq!(read.updates.desk, healthy.updates.desk);
    assert_eq!(read.updates.pi, healthy.updates.pi);
    assert_eq!(read.updates.shell, healthy.updates.shell);
    assert_eq!(read.updates.packages, healthy.updates.packages);
    assert_eq!(read.theme, healthy.theme);
}

/// S5: no path, no command line and no raw output of a tool travels with the
/// snapshot - not even the path the version probe prints on its own last line.
#[test]
fn no_path_and_no_raw_tool_output_reaches_the_snapshot() {
    let seams = Seams::healthy();
    let verbose: VersionProbe = Arc::new(|_language: Language| {
        Ok(concat!(
            "gentle-shell 4.7.0\n",
            "pi 1.5.2\n",
            "home linked C:\\Users\\someone\\.pi\\agent\n",
            "warning: stderr said C:\\Program Files\\node\\node.exe\n",
        )
        .to_owned())
    });
    let seams = seams.with_versions(verbose);

    let read = snapshot(&seams);
    let carried = serde_json::to_string(&read).expect("the snapshot is serializable");

    for secret in [
        "Users",
        "someone",
        "Program Files",
        "home linked",
        "warning",
        "node.exe",
        ".pi",
    ] {
        assert!(!carried.contains(secret), "{secret:?} reached the wire");
    }
    // And the rows are still drawn from the same answer.
    assert_eq!(read.general.tools[0].version.as_deref(), Some("4.7.0"));
    assert_eq!(read.general.tools[1].version.as_deref(), Some("1.5.2"));
}

/// Additive on the wire, like the Extensions block: a payload written before
/// this page existed reads as "nothing was read yet", never as four missing
/// tools.
#[test]
fn a_payload_without_the_environment_check_reads_as_nothing_read_yet() {
    let read = snapshot(&Seams::healthy());
    let mut payload = serde_json::to_value(&read).expect("the snapshot is serializable");
    payload
        .as_object_mut()
        .expect("the snapshot is an object")
        .remove("general");

    let parsed: SettingsSnapshot =
        serde_json::from_value(payload).expect("an older payload still reads");

    assert_eq!(parsed.general, GeneralView::default());
    assert_eq!(parsed.general.tools.len(), 0);
    assert_eq!(parsed.general.checked_ms, None);
    // The rest of the object is what it always was.
    assert!(parsed.updates.packages.available);
    assert_eq!(parsed.theme, ThemeView::Gentle);
}
