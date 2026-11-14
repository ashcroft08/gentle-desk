//! The Usage pane's slice: the subscription usage the shell's own sources
//! report, located and run through seams.
//!
//! The rules are `gentle_runtime::usage`'s (the locator and the strict parser);
//! this module turns their answers into the contract's view and supplies the
//! real node run. Nothing here writes anything, and nothing here reads a chat.

use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gentle_runtime::usage as rule;

use super::{FileRead, NodeProbe};
use crate::contract::{
    UsageLimitView, UsageProviderStatusView, UsageProviderView, UsageStatusView, UsageView,
    UsageWindowView,
};

/// The shell's own package list, beside its `auth.json` in the agent folder.
pub const SETTINGS_FILE: &str = "settings.json";

/// How long one run of the sources' script may take. The package gives each
/// provider 10 s of its own and runs them together; 30 s leaves room for a slow
/// machine and still answers in a time a person waits.
pub const NODE_BUDGET: Duration = Duration::from_secs(30);

/// How often the run's own process is looked at while it works.
const POLL: Duration = Duration::from_millis(50);

/// The pane's slice: where the sources stand, and what they answered when they
/// ran.
///
/// The four locator states are distinct on purpose. `Missing` means this shell
/// does not have the package (no entry in its settings, no checkout, no
/// script), which is what the pane's install button is for. `Unreadable` means
/// something the locator needed could not be read - the settings file, the
/// script, the agent folder itself - and `Failed` means the sources ran and
/// their answer could not be used. Both of the last two say why; a person can
/// act on a sentence and not on a code.
pub fn view(files: &FileRead, home: Option<&Path>, node: &NodeProbe, now_ms: u64) -> UsageView {
    let Some(home) = home else {
        return unreadable(
            "the shell's agent folder could not be found on this machine".to_owned(),
        );
    };
    let settings = match files(&home.join(SETTINGS_FILE)) {
        Ok(text) => text,
        Err(reason) => return unreadable(reason),
    };
    let Some(script) = rule::script(settings.as_deref(), home) else {
        return missing("the usage sources are not installed for this shell");
    };
    // The checkout is read for the script's presence only: a folder pi has not
    // cloned, or one without the headless entry point, is the same missing
    // package to the pane.
    match files(&script) {
        Ok(Some(_)) => {}
        Ok(None) => return missing("the usage sources are not installed for this shell"),
        Err(reason) => return unreadable(reason),
    }
    let stdout = match node(&script) {
        Ok(stdout) => stdout,
        Err(reason) => return failed(reason),
    };
    match rule::parse(&stdout) {
        Ok(document) => UsageView {
            status: UsageStatusView::Installed,
            providers: document
                .providers
                .into_iter()
                .map(|provider| provider_view(now_ms, provider))
                .collect(),
        },
        Err(reason) => failed(reason),
    }
}

fn missing(reason: &str) -> UsageView {
    UsageView {
        status: UsageStatusView::Missing(reason.to_owned()),
        providers: Vec::new(),
    }
}

fn unreadable(reason: String) -> UsageView {
    UsageView {
        status: UsageStatusView::Unreadable(reason),
        providers: Vec::new(),
    }
}

fn failed(reason: String) -> UsageView {
    UsageView {
        status: UsageStatusView::Failed(reason),
        providers: Vec::new(),
    }
}

/// One provider of the document as the pane draws it: its name, its status, its
/// plan when it reports one, and one row per window with the countdown the
/// clock gives that window's reset.
fn provider_view(now_ms: u64, provider: rule::Provider) -> UsageProviderView {
    UsageProviderView {
        provider: provider.provider,
        status: match provider.status {
            rule::ProviderStatus::Ok => UsageProviderStatusView::Ok,
            rule::ProviderStatus::NoData => UsageProviderStatusView::NoData,
            rule::ProviderStatus::Failed => UsageProviderStatusView::Failed,
        },
        plan: provider.plan,
        error: provider.error,
        limits: provider
            .limits
            .into_iter()
            .map(|limit| UsageLimitView {
                name: limit.name,
                limit_reached: limit.limit_reached,
                windows: limit
                    .windows
                    .into_iter()
                    .map(|window| UsageWindowView {
                        label: window.label,
                        used_percent: window.used_percent,
                        resets_in: rule::reset_label(now_ms, window.reset_at),
                    })
                    .collect(),
            })
            .collect(),
    }
}

/// The real node run: `node <script>`, with the budget above.
pub fn node_probe() -> NodeProbe {
    Arc::new(|script: &Path| run(script, NODE_BUDGET))
}

/// Runs the sources' script and answers the document line it printed.
///
/// A run that cannot start, exits non-zero (the package's own contract: a
/// non-zero exit means no document at all), or outruns `budget` is the `Err`,
/// and an overrun process is stopped rather than left behind. Its answer is read
/// on threads of its own while it works, so a long document cannot fill a pipe
/// and deadlock the run.
fn run(script: &Path, budget: Duration) -> Result<String, String> {
    use std::process::{Command, Stdio};

    let mut child = Command::new("node")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("could not start ({error})"))?;
    let stdout = child.stdout.take().map(drained);
    let stderr = child.stderr.take().map(drained);
    let deadline = Instant::now() + budget;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if Instant::now() >= deadline {
                    // The child is ours: it is stopped here, never left running
                    // behind a pane that has already given up on it.
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "did not answer within {} seconds",
                        budget.as_secs()
                    ));
                }
                std::thread::sleep(POLL);
            }
            Err(error) => return Err(format!("could not run ({error})")),
        }
    };
    let out = joined(stdout);
    let err = joined(stderr);
    if !status.success() {
        let code = status
            .code()
            .map_or_else(|| "no exit code".to_owned(), |code| code.to_string());
        return Err(if err.trim().is_empty() {
            format!("stopped without printing a document ({code})")
        } else {
            format!("stopped without printing a document ({})", err.trim())
        });
    }
    Ok(out)
}

/// Reads a child's stream to a string on a thread of its own.
fn drained(mut stream: impl Read + Send + 'static) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        let mut text = String::new();
        let _ = stream.read_to_string(&mut text);
        text
    })
}

/// The text one drained stream answered, or nothing for a stream that is gone.
fn joined(handle: Option<std::thread::JoinHandle<String>>) -> String {
    handle
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::settings::{FileRead, NodeProbe};

    const HOME: &str = "C:/Users/u/.pi/agent";

    /// The shell's settings as this machine writes them, reduced to the entry
    /// that matters.
    const SETTINGS: &str =
        r#"{"packages":["npm:gentle-pi","git:github.com/ashcroft08/pi-usage-sources"]}"#;

    /// The document the package printed on this machine, verbatim (the same one
    /// the runtime rules' tests use).
    const DOCUMENT: &str = r#"{"schema":"pi-usage-sources/usage/v1","generatedAt":1794637790033,"sources":[{"provider":"opencode-go","status":"no-data","fetchedAt":1794637790033,"limits":[]},{"provider":"antigravity","status":"ok","fetchedAt":1794637790033,"limits":[{"name":"Gemini Models","limitReached":false,"windows":[{"label":"week","usedPercent":0,"resetAt":1795242599000,"windowSeconds":604800}]},{"name":"Claude and GPT models","limitReached":false,"windows":[{"label":"week","usedPercent":0,"resetAt":1795242599000,"windowSeconds":604800}]}]},{"provider":"commandcode","status":"ok","plan":"individual goat","fetchedAt":1794637790033,"limits":[{"name":"command code","limitReached":false,"windows":[{"label":"5h","usedPercent":4.6771898400000005,"resetAt":1794644915319,"windowSeconds":18000},{"label":"week","usedPercent":88.272975324,"resetAt":1794727272498,"windowSeconds":604800}]},{"name":"credits","limitReached":false,"windows":[{"label":"billing","usedPercent":44.13765581735515,"resetAt":1796714153000,"windowSeconds":2592000}]}]},{"provider":"claude-bridge","status":"ok","plan":"pro","fetchedAt":1794633348955,"limits":[{"name":"claude","limitReached":false,"windows":[{"label":"5h","usedPercent":21,"resetAt":1794644400000,"windowSeconds":18000},{"label":"week","usedPercent":96,"resetAt":1794794400000,"windowSeconds":604800}]}]}]}"#;

    /// One clock for every expectation: before the resets the document names,
    /// so each countdown is positive and its text is pinned.
    const NOW: u64 = 1_794_640_000_000;

    fn home() -> PathBuf {
        PathBuf::from(HOME)
    }

    type Asked = Arc<Mutex<Vec<String>>>;

    fn asked() -> Asked {
        Arc::new(Mutex::new(Vec::new()))
    }

    /// A file reader that answers by file name - the shell's settings and the
    /// sources' own script - and records every path it was asked about.
    fn files(settings: Option<&'static str>, script: Option<&'static str>, log: Asked) -> FileRead {
        Arc::new(move |path: &Path| {
            log.lock()
                .expect("the log is not poisoned")
                .push(path.display().to_string());
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            Ok(match name {
                "settings.json" => settings.map(str::to_owned),
                "usage-json.ts" => script.map(str::to_owned),
                _ => None,
            })
        })
    }

    /// A reader that fails every path, as a file the user cannot read does.
    fn unreadable(reason: &'static str) -> FileRead {
        Arc::new(move |_path: &Path| Err(reason.to_owned()))
    }

    /// A node run that answers `answer` and counts how often it was asked.
    fn node(answer: Result<&'static str, &'static str>, ran: Arc<AtomicUsize>) -> NodeProbe {
        Arc::new(move |_script: &Path| {
            ran.fetch_add(1, Ordering::SeqCst);
            answer.map(str::to_owned).map_err(str::to_owned)
        })
    }

    fn never() -> Arc<AtomicUsize> {
        Arc::new(AtomicUsize::new(0))
    }

    #[test]
    fn the_installed_sources_reach_the_pane_provider_by_provider() {
        let log = asked();
        let ran = never();
        let view = view(
            &files(Some(SETTINGS), Some(DOCUMENT), Arc::clone(&log)),
            Some(&home()),
            &node(Ok(DOCUMENT), Arc::clone(&ran)),
            NOW,
        );

        assert_eq!(view.status, UsageStatusView::Installed);
        // Compared as paths, since Windows spells the same folder with either
        // separator.
        let seen = log.lock().expect("the log is not poisoned").clone();
        assert_eq!(
            Path::new(&seen[0]),
            home().join("settings.json"),
            "the locator reads the shell's own settings first"
        );
        assert_eq!(
            Path::new(&seen[1]),
            home()
                .join("git")
                .join("github.com")
                .join("ashcroft08")
                .join("pi-usage-sources")
                .join("usage-json.ts"),
            "then the sources' own script inside the checkout pi made"
        );
        assert_eq!(
            ran.load(Ordering::SeqCst),
            1,
            "one run when the user asks, and only one"
        );

        let names: Vec<&str> = view
            .providers
            .iter()
            .map(|provider| provider.provider.as_str())
            .collect();
        assert_eq!(
            names,
            vec!["opencode-go", "antigravity", "commandcode", "claude-bridge"]
        );

        let commandcode = &view.providers[2];
        assert_eq!(commandcode.status, UsageProviderStatusView::Ok);
        assert_eq!(commandcode.plan.as_deref(), Some("individual goat"));
        assert_eq!(commandcode.limits[0].name, "command code");
        assert!(!commandcode.limits[0].limit_reached);
        assert_eq!(commandcode.limits[0].windows[0].label, "5h");
        assert!((commandcode.limits[0].windows[0].used_percent - 4.6771898400000005).abs() < 1e-12);
        // 1 794 644 915 319 - 1 794 640 000 000 = 4 915 319 ms, under two hours.
        assert_eq!(
            commandcode.limits[0].windows[0].resets_in.as_deref(),
            Some("resets in 2h")
        );
    }

    #[test]
    fn a_no_data_provider_is_drawn_as_no_data_and_never_as_zero() {
        let view = view(
            &files(Some(SETTINGS), Some(DOCUMENT), asked()),
            Some(&home()),
            &node(Ok(DOCUMENT), never()),
            NOW,
        );

        let opencode = &view.providers[0];
        assert_eq!(opencode.status, UsageProviderStatusView::NoData);
        assert!(
            opencode.limits.is_empty(),
            "no-data carries no window a zero could be read out of"
        );
    }

    #[test]
    fn a_limit_the_provider_marked_as_reached_reaches_the_pane_marked() {
        let document = r#"{"schema":"pi-usage-sources/usage/v1","generatedAt":1,"sources":[
            {"provider":"claude-bridge","status":"ok","plan":"pro","fetchedAt":1,"limits":[
                {"name":"claude","limitReached":true,"windows":[
                    {"label":"week","usedPercent":96,"resetAt":null}]}]}]}"#;
        let view = view(
            &files(Some(SETTINGS), Some(document), asked()),
            Some(&home()),
            &node(Ok(document), never()),
            NOW,
        );

        let limit = &view.providers[0].limits[0];
        assert!(limit.limit_reached);
        assert_eq!(
            limit.windows[0].resets_in, None,
            "a provider that reports no reset time gets no countdown"
        );
    }

    #[test]
    fn no_entry_in_the_shells_settings_is_missing() {
        let ran = never();
        let view = view(
            &files(Some(r#"{"packages":["npm:gentle-pi"]}"#), None, asked()),
            Some(&home()),
            &node(Ok(DOCUMENT), Arc::clone(&ran)),
            NOW,
        );

        assert!(matches!(view.status, UsageStatusView::Missing(_)));
        assert!(view.providers.is_empty());
        assert_eq!(
            ran.load(Ordering::SeqCst),
            0,
            "nothing is run for a missing package"
        );
    }

    #[test]
    fn a_shell_without_settings_is_missing() {
        let view = view(
            &files(None, None, asked()),
            Some(&home()),
            &node(Ok(DOCUMENT), never()),
            NOW,
        );
        assert!(matches!(view.status, UsageStatusView::Missing(_)));
    }

    #[test]
    fn an_entry_whose_checkout_is_not_there_is_missing() {
        let ran = never();
        let view = view(
            &files(Some(SETTINGS), None, asked()),
            Some(&home()),
            &node(Ok(DOCUMENT), Arc::clone(&ran)),
            NOW,
        );

        assert!(matches!(view.status, UsageStatusView::Missing(_)));
        assert_eq!(ran.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn a_settings_file_that_cannot_be_read_says_why() {
        let view = view(
            &unreadable("permission denied"),
            Some(&home()),
            &node(Ok(DOCUMENT), never()),
            NOW,
        );
        assert_eq!(
            view.status,
            UsageStatusView::Unreadable("permission denied".to_owned())
        );
    }

    #[test]
    fn a_script_that_cannot_be_read_says_why() {
        let tally = Arc::new(AtomicUsize::new(0));
        let fail_after_settings: FileRead = {
            let count = Arc::clone(&tally);
            Arc::new(move |path: &Path| {
                let seen = count.fetch_add(1, Ordering::SeqCst);
                if seen == 0 {
                    Ok(Some(SETTINGS.to_owned()))
                } else {
                    Err(format!("{} could not be read", path.display()))
                }
            })
        };
        let view = view(
            &fail_after_settings,
            Some(&home()),
            &node(Ok(DOCUMENT), never()),
            NOW,
        );
        match view.status {
            UsageStatusView::Unreadable(reason) => {
                assert!(reason.contains("could not be read"), "{reason}")
            }
            other => panic!("an unreadable script must say why, got {other:?}"),
        }
    }

    #[test]
    fn no_agent_folder_is_said_plainly() {
        let view = view(
            &files(Some(SETTINGS), Some(DOCUMENT), asked()),
            None,
            &node(Ok(DOCUMENT), never()),
            NOW,
        );
        match view.status {
            UsageStatusView::Unreadable(reason) => {
                assert!(reason.contains("agent folder"), "{reason}")
            }
            other => panic!("no agent folder must say so, got {other:?}"),
        }
    }

    #[test]
    fn a_run_that_failed_is_failed_with_its_reason() {
        let view = view(
            &files(Some(SETTINGS), Some(DOCUMENT), asked()),
            Some(&home()),
            &node(Err("node is not installed"), never()),
            NOW,
        );
        assert_eq!(
            view.status,
            UsageStatusView::Failed("node is not installed".to_owned())
        );
        assert!(view.providers.is_empty());
    }

    #[test]
    fn a_document_of_another_schema_is_failed_and_named() {
        let wrong = r#"{"schema":"someone/else","generatedAt":1,"sources":[]}"#;
        let view = view(
            &files(Some(SETTINGS), Some(DOCUMENT), asked()),
            Some(&home()),
            &node(Ok(wrong), never()),
            NOW,
        );
        match view.status {
            UsageStatusView::Failed(reason) => assert!(reason.contains("someone/else"), "{reason}"),
            other => panic!("a wrong document must say why, got {other:?}"),
        }
    }

    #[test]
    fn a_waiting_view_carries_nothing_and_the_reading_one_says_it_is_running() {
        assert_eq!(
            UsageView::default(),
            UsageView {
                status: UsageStatusView::Unknown,
                providers: Vec::new()
            }
        );
        assert_ne!(
            UsageView {
                status: UsageStatusView::Reading,
                providers: Vec::new()
            },
            UsageView::default()
        );
    }
}
