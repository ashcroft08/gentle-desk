//! The packages chain, against the REAL `gentle-shell` (packages-status T7, L2,
//! L3).
//!
//! Everything else in this suite drives a stand-in child, and the stand-in does
//! not speak `/gentle:desktop` at all: the real pipeline - a real launcher, a
//! real child, a real `capabilities`/`get_update_status` exchange, pi's real
//! package check - has never run end to end. These two tests are that run, by
//! hand, on the machine that has the shell installed:
//!
//! ```sh
//! timeout 240 cargo test -j 2 -p gentle-desk-app --test live_packages -- --ignored --nocapture
//! ```
//!
//! Both are `#[ignore]`d: they cost a real child, a real registry check and the
//! machine's own state. The pipeline they build is the app's - `commands::Bridge`
//! with a recorder where the window's emit step goes, the pump armed with the
//! same recorder, the production packages trigger
//! ([`Bridge::arm_packages_check`](gentle_desk_lib::commands::Bridge::arm_packages_check)),
//! a fresh `new-N` chat opened and started the lazy way (`OpenNewChat`, then the
//! composer's own `EnsureStarted`) - and the two homes the app can run with:
//!
//! - [`the_real_chain_lists_extensions_in_an_isolated_home`] - `--home <scratch>`,
//!   so no pi home of the user's is touched.
//! - [`the_real_chain_names_the_extension_that_is_behind_in_a_link_home`] - the
//!   maintainer's own setup (`shell_home: link`): the user's `~/.pi/agent`, where
//!   `@estebanforge/pi-antigravity-bridge` is behind (L3).
//!
//! What they prove is the RUST half: the trigger fires on the child's first
//! event, the gate names the chat, the child answers the channel, `PackagesView`
//! is derived and HELD, and `ViewState.updates.packages` is pushed with the
//! extension in `behind`. They prove nothing about the web side (the gear's dot,
//! the pane's block, which slice the sidebar reads).

use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use gentle_app::ChatId;
use gentle_desk_lib::commands::{Bridge as AppBridge, Emit};
use gentle_desk_lib::contract::{PackagesView, UiIntent, ViewState};
use gentle_desk_lib::events::Push;

/// The whole chain's budget: a child speaks at ~12 s and pi's own refresh is
/// bounded at 15 s, so 120 s is far more than the chain needs and still a bound.
const BUDGET: Duration = Duration::from_secs(120);

/// The environment is process-global and the two tests must not see each other's
/// child, so only one of them runs at a time.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// One test's isolated world: its own config folder, its own `LOCALAPPDATA`, and
/// for the isolated run its own gentle-shell home. `GENTLE_SHELL_BIN` is cleared
/// so the launcher is resolved the way the app resolves it (PATH, then
/// `%APPDATA%\npm`).
struct Harness {
    _serial: MutexGuard<'static, ()>,
    root: PathBuf,
    /// The env names this test set, removed when it ends.
    set: Vec<&'static str>,
}

impl Harness {
    /// `home` is `None` for the isolated run (a scratch `--home`), `Some("link")`
    /// for the user's own pi.
    fn new(home: Option<&str>) -> Self {
        let serial = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let root = scratch_root();
        std::fs::create_dir_all(root.join("appdata")).expect("the scratch APPDATA is creatable");
        std::fs::create_dir_all(root.join("localappdata"))
            .expect("the scratch LOCALAPPDATA is creatable");
        let mut set = vec!["APPDATA", "LOCALAPPDATA"];
        std::env::set_var("APPDATA", root.join("appdata"));
        std::env::set_var("LOCALAPPDATA", root.join("localappdata"));
        // The launcher: the REAL one, resolved from PATH and the npm global
        // folder. An override left in the environment would not be the app's own
        // resolution, and this run must be it.
        std::env::remove_var("GENTLE_SHELL_BIN");
        match home {
            None => {
                // A scratch home is a FIRST run for the shell, which would install
                // its companion packages there (one network round trip per test).
                // This run reads a package listing, it does not provision a home.
                std::env::set_var("GENTLE_SHELL_NO_AUTO_SETUP", "1");
                set.push("GENTLE_SHELL_NO_AUTO_SETUP");
                let home = root.join("home");
                std::fs::create_dir_all(&home).expect("the scratch home is creatable");
                std::env::set_var("GENTLE_SHELL_HOME", &home);
                std::env::set_var("PI_CODING_AGENT_DIR", root.join("pi"));
                set.extend(["GENTLE_SHELL_HOME", "PI_CODING_AGENT_DIR"]);
            }
            Some(choice) => {
                // The app's home choice is the config's, and the environment must
                // not name a home of its own: `--link` reads the user's pi.
                std::env::remove_var("GENTLE_SHELL_HOME");
                std::env::remove_var("PI_CODING_AGENT_DIR");
                set.extend(["GENTLE_SHELL_HOME", "PI_CODING_AGENT_DIR"]);
                let dir = root.join("appdata").join("gentle-desk");
                std::fs::create_dir_all(&dir).expect("the scratch config folder is creatable");
                std::fs::write(
                    dir.join("config.json"),
                    format!("{{\"shell_home\":\"{choice}\"}}"),
                )
                .expect("the scratch config is writable");
            }
        }
        Self {
            _serial: serial,
            root,
            set,
        }
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        for name in &self.set {
            std::env::remove_var(name);
        }
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// A fresh directory of the system temp dir, never inside the worktree.
fn scratch_root() -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "gentle-desk-live-packages-{}-{stamp}",
        std::process::id()
    ))
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// Each stage the chain must reach, in the order it must reach them, with the
/// instant it happened. Printed in full when a stage does not come.
#[derive(Default)]
struct Trace {
    start: Mutex<Option<Instant>>,
    lines: Mutex<Vec<String>>,
    pushes: Mutex<Vec<ViewState>>,
}

impl Trace {
    fn begin(&self) {
        *lock(&self.start) = Some(Instant::now());
    }

    fn note(&self, what: impl AsRef<str>) {
        let ms = lock(&self.start)
            .map(|start| start.elapsed().as_millis())
            .unwrap_or(0);
        lock(&self.lines).push(format!("{ms:>7} ms  {}", what.as_ref()));
    }

    /// Records one pushed slice and names what it carries.
    fn push(&self, view: ViewState) {
        let n = {
            let mut pushes = lock(&self.pushes);
            pushes.push(view.clone());
            pushes.len()
        };
        self.note(format!(
            "push #{n}: open_chats={} foreground={:?} activity={} messages={} packages={}",
            view.open_chats,
            view.foreground_chat,
            view.activity.len(),
            view.messages.len(),
            show(&view.updates.packages)
        ));
    }

    fn pushed(&self) -> Vec<ViewState> {
        lock(&self.pushes).clone()
    }

    fn dump(&self) -> String {
        lock(&self.lines).join("\n")
    }
}

/// One packages block, as a line: the states the pane words, nothing raw.
fn show(packages: &PackagesView) -> String {
    let behind: Vec<String> = packages
        .behind
        .iter()
        .map(|item| match &item.current {
            Some(current) => format!("{} {}", item.name, current),
            None => item.name.clone(),
        })
        .collect();
    format!(
        "{{available={} unknown={} unchecked={} truncated={} behind={:?}}}",
        packages.available, packages.unknown, packages.unchecked, packages.truncated, behind
    )
}

/// The app's own bridge, windowless: `Bridge::new` with a recorder in place of
/// the window's emit step, the pump armed with the same recorder, and the
/// packages trigger armed by the production wiring.
struct Live {
    /// The bridge the app builds. Dropped last: its `Drop` stops the children.
    app: AppBridge,
    trace: Arc<Trace>,
}

impl Live {
    fn new(trace: Arc<Trace>) -> Self {
        let emit: Emit = {
            let trace = Arc::clone(&trace);
            Arc::new(move |view: &ViewState| {
                trace.push(view.clone());
                Ok(())
            })
        };
        // The harness's own arguments are libtest's (`--ignored`, `--nocapture`),
        // not the flags a windowed app is started with: the real launcher gets the
        // empty list, which is what a shortcut-started app forwards.
        let app = AppBridge::new()
            .with_emit(emit)
            .with_forwarded_args(Vec::<OsString>::new());
        let push: Push = {
            let trace = Arc::clone(&trace);
            Box::new(move |view| trace.push(view))
        };
        app.pump.arm(push);
        // The same step `arm_events` runs at startup, which is where the trigger
        // that makes the check fire on a child's first event is installed.
        app.arm_packages_check();
        trace.note("packages trigger armed (the production `arm_packages_check`)");
        Self { app, trace }
    }

    /// A fresh chat, the way the window opens one (welcome-screen T4, R2): the
    /// click ALLOCATES it and starts nothing, and the composer's own ask starts
    /// its child.
    fn open_new_chat(&self) -> String {
        let allocated = self
            .app
            .dispatch(UiIntent::OpenNewChat)
            .expect("the bridge accepts an intent while it is open");
        let chat = allocated
            .foreground_chat
            .clone()
            .expect("a new chat is the foreground one");
        self.trace.note(format!(
            "OpenNewChat: open_chats={} foreground={chat} (it is allocated; NO child yet)",
            allocated.open_chats
        ));
        let started = self
            .app
            .dispatch(UiIntent::EnsureStarted {
                chat_id: chat.clone(),
            })
            .expect("the bridge accepts the composer's ask");
        self.trace.note(format!(
            "EnsureStarted {chat}: open_chats={} (the child is being started)",
            started.open_chats
        ));
        self.trace.note(format!("gate now: {}", self.gate()));
        chat
    }

    /// The gate as it stands right now: the runtime the pump holds, the chat the
    /// foreground id names, and the chat the runtime would ask. Read-only, and the
    /// same two calls `asking_chat` makes.
    fn gate(&self) -> String {
        let view = self.app.holder.view();
        let Some(runtime) = self.app.pump.runtime() else {
            return format!(
                "open_chats={} foreground={:?} host=NONE (EventPump::runtime() is None, so \
                 no child of this run ever started)",
                view.open_chats, view.foreground_chat
            );
        };
        let named = view
            .foreground_chat
            .as_deref()
            .and_then(ChatId::of_text)
            .and_then(|chat| runtime.packages_chat(Some(&chat)));
        format!(
            "open_chats={} foreground={:?} host=present chat_named={:?}",
            view.open_chats,
            view.foreground_chat,
            named.map(|chat| chat.as_str().to_owned())
        )
    }

    fn held(&self) -> Option<PackagesView> {
        self.app.holder.held_packages()
    }

    /// The explicit path, the pane's "Comprobar de nuevo": the bridge's own
    /// command, which returns at once and pushes its answer.
    fn check_again(&self, from: usize) {
        self.trace.note("check_packages() (the explicit ask)");
        self.app.check_packages();
        let deadline = Instant::now() + BUDGET;
        let pushed = self.wait_pushed(deadline, from, |view| view.updates.packages.available);
        self.trace.note(format!(
            "explicit check answered: packages={}",
            show(&pushed.updates.packages)
        ));
    }

    /// Waits until a slice pushed at index `from` or later carries a listing.
    fn wait_pushed(
        &self,
        deadline: Instant,
        from: usize,
        keep: impl Fn(&ViewState) -> bool,
    ) -> ViewState {
        loop {
            let pushed = self.trace.pushed();
            if let Some(view) = pushed.iter().skip(from).rev().find(|view| keep(view)) {
                return view.clone();
            }
            if Instant::now() >= deadline {
                panic!(
                    "waited {BUDGET:?} for a pushed slice with the listing and it never came.\n\
                     === timeline ===\n{}\n=== end ===\nlast gate: {}",
                    self.trace.dump(),
                    self.gate()
                );
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Waits for the automatic check to come to something: the block the run holds
    /// is set for EVERY outcome the check thread produces (a listing, a deadline, a
    /// refusal and a decline alike), so this is the one fact that separates "the
    /// trigger fired and its thread finished" from "the trigger never fired".
    fn wait_check_thread(&self, deadline: Instant) -> PackagesView {
        loop {
            if let Some(held) = self.held() {
                self.trace
                    .note(format!("automatic check finished, held={}", show(&held)));
                return held;
            }
            if Instant::now() >= deadline {
                panic!(
                    "waited {BUDGET:?} for the automatic check to come to anything and no \
                     block was ever held: the trigger never fired.\n\
                     === timeline ===\n{}\n=== end ===\nlast gate: {}",
                    self.trace.dump(),
                    self.gate()
                );
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

impl Drop for Live {
    fn drop(&mut self) {
        // The app's own shutdown, so no child outlives this run.
        self.app.executor.shutdown();
        self.app.pump.stop();
        self.trace.note("child(ren) stopped (executor shutdown)");
    }
}

/// The chain both tests run: allocate and start a fresh chat, then wait for the
/// pushed slice to carry a real listing, then ask again explicitly.
fn run_chain(home: Option<&str>, label: &str) -> (PackagesView, PackagesView) {
    let _harness = Harness::new(home);
    let trace = Arc::new(Trace::default());
    let live = Live::new(Arc::clone(&trace));
    trace.begin();
    trace.note(format!(
        "live run: {label}; config folder APPDATA={}",
        std::env::var("APPDATA").unwrap_or_default()
    ));
    assert!(
        live.app.holder.view().fatal.is_none(),
        "the launcher did not resolve at startup: {:?}\n{}",
        live.app.holder.view().fatal,
        trace.dump()
    );

    let chat = live.open_new_chat();
    let deadline = Instant::now() + BUDGET;

    // 1. The automatic check: a child event fires the trigger, which runs the
    //    explicit check on a thread of its own.
    let held = live.wait_check_thread(deadline);
    trace.note(format!("gate once the check answered: {}", live.gate()));

    // 2. The pushed slice: the block rides in `ViewState.updates.packages`, which
    //    is what the gear's dot and the pane read.
    let pushed = live.wait_pushed(deadline, 0, |view| view.updates.packages.available);

    // 3. The explicit path, the pane's own ask.
    let before = trace.pushed().len();
    live.check_again(before);

    let panicked = pushed.foreground_chat.as_deref() != Some(chat.as_str());
    assert!(
        !panicked,
        "the listing was pushed for another chat: {:?}",
        pushed.foreground_chat
    );
    println!(
        "=== live packages timeline ({label}) ===\n{}\n=== end ===",
        trace.dump()
    );
    (held, pushed.updates.packages)
}

/// packages-status T7, S7: a fresh chat in an ISOLATED gentle-shell home. The
/// shell is the real one; the home is this test's, so nothing of the user's pi is
/// read and nothing is written to it.
#[test]
#[ignore = "needs the real gentle-shell on PATH; run by hand"]
fn the_real_chain_lists_extensions_in_an_isolated_home() {
    let (held, pushed) = run_chain(None, "isolated home");

    assert!(
        held.available,
        "the automatic check produced a real listing: {}",
        show(&held)
    );
    assert!(
        pushed.available,
        "the pushed slice carries the listing: {}",
        show(&pushed)
    );
    assert!(
        !pushed.unknown,
        "a listing pi produced is never 'the read could not run': {}",
        show(&pushed)
    );
}

/// packages-status T7, S7, L3: the maintainer's own setup - the user's pi
/// (`--link`), where `@estebanforge/pi-antigravity-bridge` is behind. This is the
/// one run that reproduces the report with the real extension list.
#[test]
#[ignore = "needs the real gentle-shell, the user's pi home and the extension behind; run by hand"]
fn the_real_chain_names_the_extension_that_is_behind_in_a_link_home() {
    let (held, pushed) = run_chain(Some("link"), "link home (the user's pi)");

    assert!(
        held.available && pushed.available,
        "the check produced a listing and it was pushed: held={} pushed={}",
        show(&held),
        show(&pushed)
    );
    assert!(
        !pushed.unchecked,
        "pi's own check ran, so no item is still unknown: {}",
        show(&pushed)
    );
    assert!(
        pushed
            .behind
            .iter()
            .any(|item| item.name.contains("antigravity")),
        "the extension the maintainer sees in the terminal is in `behind`; this assertion is \
         the machine's own state (run `gentle-shell` and read pi's notice if it fails): {}",
        show(&pushed)
    );
}

/// packages-status T7, L3 - the maintainer's "I opened several chats" and the
/// phantom of the second report, against the real shell:
///
/// 1. one chat is opened the lazy way and its child really starts (chat X);
/// 2. a SECOND chat is allocated and never started (chat Y), and it is in front -
///    the composer of a brand new chat is what the user sees;
/// 3. the explicit check must still reach X's child and answer a listing: the
///    gate falls back to the open chat the host can ask;
/// 4. closing Y (a chat whose child never started) takes the app back to ONE open
///    chat - the number the Updates pane's terminal gate reads - and the listing
///    the run holds is untouched.
#[test]
#[ignore = "needs the real gentle-shell on PATH; run by hand"]
fn the_gate_reaches_a_live_chat_with_a_never_started_chat_in_front() {
    let _harness = Harness::new(None);
    let trace = Arc::new(Trace::default());
    let live = Live::new(Arc::clone(&trace));
    trace.begin();
    trace.note("live run: several chats (a live one and a never-started one in front)");

    // Chat X: allocated, then started by its own composer ask.
    let x = live.open_new_chat();
    let deadline = Instant::now() + BUDGET;
    live.wait_check_thread(deadline);
    live.wait_pushed(deadline, 0, |view| view.updates.packages.available);
    trace.note(format!("chat X={x} is live; gate: {}", live.gate()));

    // Chat Y: allocated and NEVER started, and it is the one in front.
    let allocated = live
        .app
        .dispatch(UiIntent::OpenNewChat)
        .expect("the bridge accepts an intent while it is open");
    let y = allocated
        .foreground_chat
        .clone()
        .expect("a new chat is the foreground one");
    assert_eq!(
        allocated.open_chats, 2,
        "two chats are open, one has a child"
    );
    trace.note(format!(
        "chat Y={y} is allocated in front and starts NO child; gate: {}",
        live.gate()
    ));

    // The gate must reach X, the chat the host really has a child for.
    let before = trace.pushed().len();
    live.check_again(before);
    let pushed = live.wait_pushed(Instant::now() + BUDGET, before, |view| {
        view.updates.packages.available
    });
    assert!(
        pushed.updates.packages.available,
        "the check reached a live chat although the chat in front never started one: {}",
        show(&pushed.updates.packages)
    );

    // Closing the never-started chat takes the app back to one open chat.
    let closed = live
        .app
        .dispatch(UiIntent::CloseChat { chat_id: y.clone() })
        .expect("the bridge accepts the close");
    trace.note(format!(
        "CloseChat {y}: open_chats={} foreground={:?}",
        closed.open_chats, closed.foreground_chat
    ));
    assert_eq!(
        closed.open_chats, 1,
        "the chat that never started a child leaves the open set: it is what the Updates pane \
         counts"
    );
    assert_ne!(
        closed.foreground_chat.as_deref(),
        Some(y.as_str()),
        "the closed chat is not in front any more"
    );

    println!(
        "=== live packages timeline (several chats) ===\n{}\n=== end ===",
        trace.dump()
    );
}
