//! Windowless tests of the REAL `Bridge::dispatch` (backlog 31, shell-choice
//! S2/L6): the intent path, the reducer fold, the per-chat commands and the
//! emit step, with a recorder where the running app pushes `state-changed`.
//!
//! The bridge's emit step is a seam (`Emit`), installed at startup with the
//! window handle the running app has and with a recorder here: the slices these
//! tests read are the exact payloads the web side would have received, pushed
//! through the same path, and no Tauri app is ever built.
//!
//! Every test sets its own `APPDATA`, `LOCALAPPDATA`, `PI_CODING_AGENT_DIR` and
//! `GENTLE_SHELL_HOME` to a temp directory, so the real config, the real session
//! cache and the user's own pi are never read or written. The environment is
//! process-global, so only one test of this file runs at a time.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use gentle_app::{ChatId, Input, Intent};
use gentle_desk_lib::commands::{Bridge, Emit};
use gentle_desk_lib::contract::{InstallView, UiIntent, ViewState};
use gentle_desk_lib::shell_install::Installer;

/// Every wait of these tests: the only one is the install thread.
const WAIT: Duration = Duration::from_secs(10);

/// The bytes of a file the drop names: the image store checks the extension and
/// never decodes, so the content is whatever the test writes.
const PNG: &[u8] = b"\x89PNG\r\n\x1a\none";

static ENV_LOCK: Mutex<()> = Mutex::new(());

/// One test's isolated world: its own app config, local app data, pi folder and
/// gentle-shell home, all under a temp directory outside the worktree. Dropping
/// it removes every variable and the whole directory.
struct Harness {
    _serial: MutexGuard<'static, ()>,
    root: PathBuf,
    /// The pi folder `detect_pi` finds, so the app asks the home question for a
    /// reason of its own and not because this machine happens to have pi.
    pi: PathBuf,
}

impl Harness {
    fn new() -> Self {
        let serial = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let root = scratch_root();
        let pi = root.join("pi");
        for dir in [
            root.join("appdata"),
            root.join("localappdata"),
            root.join("home"),
            pi.clone(),
        ] {
            std::fs::create_dir_all(&dir).expect("the scratch directory is creatable");
        }
        std::env::set_var("APPDATA", root.join("appdata"));
        std::env::set_var("LOCALAPPDATA", root.join("localappdata"));
        std::env::set_var("GENTLE_SHELL_HOME", root.join("home"));
        std::env::set_var("PI_CODING_AGENT_DIR", &pi);
        Self {
            _serial: serial,
            root,
            pi,
        }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    /// The same world with no `APPDATA` at all: there is nowhere to write a
    /// config, which is the one fact the run below differs by.
    fn without_appdata(self) -> Self {
        std::env::remove_var("APPDATA");
        self
    }

    /// The path of the pi folder as the view carries it, compared by its text.
    fn pi_text(&self) -> String {
        self.pi.to_string_lossy().into_owned()
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        for name in [
            "APPDATA",
            "LOCALAPPDATA",
            "GENTLE_SHELL_HOME",
            "PI_CODING_AGENT_DIR",
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
        "gentle-desk-bridge-dispatch-{}-{stamp}",
        std::process::id()
    ))
}

/// The emit seam of a test: every slice the bridge pushes, in order.
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

/// The last slice the bridge pushed.
fn last(views: &Arc<Mutex<Vec<ViewState>>>) -> ViewState {
    lock(views)
        .last()
        .cloned()
        .expect("the intent path pushed a slice")
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

/// A stand-in install: it answers what the test told it to and counts the runs,
/// so a second request for the same install would be visible. A successful one
/// puts the launcher there (`FakeInstall::providing`), which is the fact the
/// completion probe is asked about.
struct FakeInstall {
    outcome: Result<(), String>,
    runs: Arc<AtomicUsize>,
    /// Set to `true` by a successful install: the launcher this machine lacked
    /// is there once npm is done.
    provides: Option<Arc<AtomicBool>>,
}

impl FakeInstall {
    fn answering(outcome: Result<(), String>, runs: &Arc<AtomicUsize>) -> Arc<dyn Installer> {
        Arc::new(Self {
            outcome,
            runs: Arc::clone(runs),
            provides: None,
        })
    }

    /// The same install, which also makes the launcher resolve: what a real
    /// `npm install -g gentle-pi` does to the machine.
    fn providing(
        outcome: Result<(), String>,
        runs: &Arc<AtomicUsize>,
        launcher_found: &Arc<AtomicBool>,
    ) -> Arc<dyn Installer> {
        Arc::new(Self {
            outcome,
            runs: Arc::clone(runs),
            provides: Some(Arc::clone(launcher_found)),
        })
    }
}

impl Installer for FakeInstall {
    fn install(&self) -> Result<(), String> {
        self.runs.fetch_add(1, Ordering::SeqCst);
        if self.outcome.is_ok() {
            if let Some(found) = self.provides.as_ref() {
                found.store(true, Ordering::SeqCst);
            }
        }
        self.outcome.clone()
    }
}

#[test]
fn the_home_question_reaches_the_web_only_when_the_launcher_resolves() {
    let harness = Harness::new();
    let (emit, views) = recorder();
    // The launcher probe is the bridge's seam, exactly as the startup probe is:
    // this one answers what the test says, and it is asked once when it is
    // installed, so the two runs differ in one fact.
    let found = Arc::new(AtomicBool::new(false));
    let probe = Arc::clone(&found);
    let bridge = Bridge::new()
        .with_emit(emit)
        .with_launcher(Arc::new(move || probe.load(Ordering::SeqCst)));

    // The run found pi and nobody chose yet, so the question is due as soon as
    // the launcher is there.
    assert_eq!(
        bridge.holder.view().shell.pi_path.as_deref(),
        Some(harness.pi_text().as_str()),
        "the run found the pi folder at startup"
    );
    assert!(
        !bridge.holder.view().shell.launcher_found,
        "the run starts with the answer of the installed probe, never the machine's"
    );

    // No launcher: the install screen is what the user sees, and the question is
    // not asked (shell-choice S2, the field report of L6).
    bridge
        .dispatch(UiIntent::CheckShellAgain)
        .expect("a launcher check answers");
    let missing = last(&views);
    assert!(
        !missing.shell.launcher_found,
        "the probe said there is none"
    );
    assert!(
        !missing.shell.ask,
        "nothing runs before gentle-shell is there, so nothing is asked"
    );

    // The launcher is there now: the same push says the question is due, so the
    // welcome screen reaches the web from the real intent path.
    found.store(true, Ordering::SeqCst);
    bridge
        .dispatch(UiIntent::CheckShellAgain)
        .expect("a launcher check answers");
    let there = last(&views);
    assert!(there.shell.launcher_found);
    assert_eq!(there.shell.choice, None, "nothing is chosen yet");
    assert!(
        there.shell.ask,
        "pi found, no choice and the launcher there: the three conditions of S2"
    );
}

#[test]
fn an_install_reaches_the_web_as_running_then_done_and_the_completion_probe_runs() {
    let _harness = Harness::new();
    let (emit, views) = recorder();
    let found = Arc::new(AtomicBool::new(false));
    let probe = Arc::clone(&found);
    let runs = Arc::new(AtomicUsize::new(0));
    // How often the launcher was asked: once when the seam was installed, as
    // `Bridge::new` asks it, and once more only when the install ends well.
    let probes = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&probes);
    let bridge = Bridge::new()
        .with_emit(emit)
        .with_launcher(Arc::new(move || {
            counted.fetch_add(1, Ordering::SeqCst);
            probe.load(Ordering::SeqCst)
        }))
        .with_installer(FakeInstall::providing(Ok(()), &runs, &found));

    let answer = bridge
        .dispatch(UiIntent::InstallShell)
        .expect("the install starts");
    assert_eq!(
        answer.shell.install,
        InstallView::Running,
        "the answer to the click says the install is running"
    );
    assert_eq!(last(&views).shell.install, InstallView::Running);

    // The install ends on its own thread; a SUCCESSFUL one re-probes the
    // launcher, so the screen the user is looking at moves on by itself.
    let done = wait_for("the install to reach the web as done", || {
        lock(&views)
            .iter()
            .rev()
            .find(|view| view.shell.install == InstallView::Done)
            .cloned()
    });
    assert_eq!(last(&views), done, "the done slice is the newest push");
    assert_eq!(
        probes.load(Ordering::SeqCst),
        2,
        "the startup probe and one completion probe, on the install's own thread"
    );
    assert!(
        done.shell.launcher_found,
        "the completion probe ran and found the launcher it just installed"
    );
    assert!(
        done.shell.ask,
        "pi found, no choice and now the launcher: the question is due without a second click"
    );
    assert_eq!(runs.load(Ordering::SeqCst), 1, "one click, one install");
}

#[test]
fn a_failed_install_reaches_the_web_and_probes_nothing() {
    let _harness = Harness::new();
    let (emit, views) = recorder();
    let found = Arc::new(AtomicBool::new(false));
    let probe = Arc::clone(&found);
    let runs = Arc::new(AtomicUsize::new(0));
    let probes = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&probes);
    let bridge = Bridge::new()
        .with_emit(emit)
        .with_launcher(Arc::new(move || {
            counted.fetch_add(1, Ordering::SeqCst);
            probe.load(Ordering::SeqCst)
        }))
        .with_installer(FakeInstall::answering(
            Err("npm is not installed".to_owned()),
            &runs,
        ));

    bridge
        .dispatch(UiIntent::InstallShell)
        .expect("the install starts");

    let failed = wait_for("the install to reach the web as failed", || {
        lock(&views)
            .iter()
            .rev()
            .find(|view| matches!(view.shell.install, InstallView::Failed { .. }))
            .cloned()
    });
    match failed.shell.install {
        InstallView::Failed { message } => assert_eq!(message, "npm is not installed"),
        other => panic!("a failed install must say why, got {other:?}"),
    }
    // A failure says nothing about the machine, so nothing is probed and the
    // view keeps claiming what it knew.
    assert_eq!(
        probes.load(Ordering::SeqCst),
        1,
        "only the probe of the startup answer: a failed install probes nothing"
    );
    assert!(!failed.shell.launcher_found);
    assert_eq!(runs.load(Ordering::SeqCst), 1, "one click, one install");
}

#[test]
fn restarting_the_app_raises_the_relaunch_flag_the_close_reads() {
    let _harness = Harness::new();
    let (emit, _views) = recorder();
    let bridge = Bridge::new().with_emit(emit);

    assert!(!bridge.relaunch_requested(), "the run starts by leaving");

    bridge
        .dispatch(UiIntent::RestartApp)
        .expect("a restart is accepted");

    assert!(
        bridge.relaunch_requested(),
        "the close flow's last step restarts the app because of this flag"
    );
}

#[test]
fn a_run_with_no_settings_folder_is_told_once_and_keeps_working() {
    // With no `APPDATA` there is no config path at all: the real bridge starts
    // on the empty config in memory and says so in one line, instead of starting
    // silent. It never refuses to work either.
    let harness = Harness::new().without_appdata();
    let (emit, views) = recorder();
    let bridge = Bridge::new().with_emit(emit);

    let notice = bridge
        .holder
        .view()
        .config_notice
        .expect("a run with no settings folder is told");
    assert!(notice.is_ascii(), "got {notice:?}");
    assert!(!notice.contains('\n'), "one line, got {notice:?}");
    assert!(
        notice.contains("settings folder could not be found"),
        "the line names the folder, got {notice:?}"
    );

    // The run works: an ordinary intent still folds and pushes, the slice it
    // pushes carries the same one line, and the notice is no push of its own.
    bridge
        .dispatch(UiIntent::DismissNotice { id: 0 })
        .expect("dismissing an id nothing holds changes nothing");
    assert_eq!(lock(&views).len(), 1, "one fold, one push");
    let pushed = last(&views);
    assert_eq!(pushed.config_notice.as_deref(), Some(notice.as_str()));
    assert_eq!(
        pushed.config_notice,
        bridge.holder.view().config_notice,
        "the same one line, not a second copy of it"
    );

    // Nothing was written where a settings folder would have been.
    assert!(
        !harness.root().join("appdata").join("gentle-desk").exists(),
        "a run with no settings folder writes nothing"
    );
}

#[test]
fn an_ordinary_intent_still_pushes_the_slice_it_folded() {
    // The path every intent takes: fold, push once, run what the fold asked for
    // (nothing for this one). The emit seam changed where the slice goes, never
    // whether it goes.
    let _harness = Harness::new();
    let (emit, views) = recorder();
    let bridge = Bridge::new().with_emit(emit);

    let answer = bridge
        .dispatch(UiIntent::DismissNotice { id: 0 })
        .expect("dismissing an id nothing holds changes nothing");

    assert_eq!(lock(&views).len(), 1, "one fold, one push");
    assert_eq!(last(&views), answer, "the command answers what it pushed");
    assert_eq!(answer, bridge.holder.view());
}

#[test]
fn attaching_and_removing_an_image_still_push_their_slice() {
    // The image commands lost the window handle and kept their behavior: the
    // drop's paths are read, the composer's slice is pushed, and removing the
    // image pushes the slice without it.
    let harness = Harness::new();
    let (emit, views) = recorder();
    let bridge = Bridge::new().with_emit(emit);
    // A foreground chat, seeded through the reducer as the sidebar would open
    // one: no child is started, so nothing here needs a model.
    bridge
        .holder
        .fold_input(Input::Intent(Intent::OpenChat(ChatId::fresh())));

    let path = harness.root().join("shot.png");
    std::fs::write(&path, PNG).expect("the dropped file is writable");
    let dropped = bridge
        .attach_images(&[path.to_string_lossy().into_owned()])
        .expect("a dropped image is attached");
    assert_eq!(dropped.images.len(), 1, "the chat holds the dropped image");
    assert_eq!(last(&views).images, dropped.images);

    let id = dropped.images[0].id.clone();
    let removed = bridge
        .remove_image(&id)
        .expect("an attached image is taken");
    assert!(removed.images.is_empty(), "the removed image is gone");
    assert_eq!(last(&views).images, removed.images);
}
