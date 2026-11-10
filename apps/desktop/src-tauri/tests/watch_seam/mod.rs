//! The seams the watcher wiring tests drive: a fake watcher source the test
//! fills with batches, a safety-net delay the test names, and the passes it ran.
//!
//! Nothing here touches the user's home: `session_seam::Home` is a temp
//! directory, the emit step is a recorder and the only real threads are the
//! ones the service under test starts.
#![allow(dead_code)] // each test binary uses one part of it

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use gentle_backend::session_watch::{Batch, SessionWatcher, WatchError};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::{ListStatusView, ViewState};
use gentle_desk_lib::sessions::service::RunPass;
use gentle_desk_lib::sessions::timer::TickDelay;
use gentle_desk_lib::sessions::watch::StartWatch;
use gentle_desk_lib::sessions::{startup, SessionsService, Wiring};
use gentle_runtime::refresh_flow::Flow;

use super::session_seam::{self, gate, Home, Recorder, LIMIT};

/// How long a test waits for something its own threads must do before it calls
/// it "nothing else happened".
pub const SETTLE: Duration = Duration::from_millis(250);

/// How long the live test waits for a real file notice: Windows/NTFS hands the
/// notice of a change over seconds late (about 3 s here, over 10 s on a CI
/// runner), so this is generous on purpose and nothing waits it out in full.
pub const LIVE: Duration = Duration::from_secs(60);

/// The watcher, replaced: every start hands the run a channel the test pushes
/// batches into, plus a real (inert) [`SessionWatcher`] over an empty directory,
/// because the flow keeps that handle and what the app drops is what the test
/// drops.
pub struct FakeSource {
    /// The batches of the running watch, replaced on every start.
    batches: Mutex<Option<Sender<Batch>>>,
    starts: AtomicUsize,
    /// What a start answers while it is set.
    refusal: Mutex<Option<String>>,
    /// The directory of the inert real watcher.
    empty: tempfile::TempDir,
}

impl FakeSource {
    pub fn new() -> Self {
        Self {
            batches: Mutex::new(None),
            starts: AtomicUsize::new(0),
            refusal: Mutex::new(None),
            empty: tempfile::tempdir().expect("a temp empty directory for the inert watcher"),
        }
    }

    /// The seam the service starts its watcher through.
    pub fn seam(self: &Arc<Self>) -> StartWatch {
        let source = Arc::clone(self);
        Arc::new(move |_dir| source.start())
    }

    /// How many times a watcher was asked for.
    pub fn starts(&self) -> usize {
        self.starts.load(Ordering::SeqCst)
    }

    /// The batches of the running watch.
    pub fn batches(&self) -> Sender<Batch> {
        lock(&self.batches)
            .clone()
            .expect("a watch started before its batches are used")
    }

    /// Ends the running watch: the batch channel disconnects, as it does when
    /// the platform watcher goes away.
    pub fn lose(&self) {
        lock(&self.batches).take();
    }

    /// Every start fails with `message`, until [`FakeSource::allow`].
    pub fn refuse(&self, message: &str) {
        *lock(&self.refusal) = Some(message.to_owned());
    }

    pub fn allow(&self) {
        *lock(&self.refusal) = None;
    }

    /// One start: the refusal, or a channel and a real handle over nothing.
    fn start(&self) -> Result<(SessionWatcher, Receiver<Batch>), WatchError> {
        self.starts.fetch_add(1, Ordering::SeqCst);
        if let Some(message) = lock(&self.refusal).clone() {
            return Err(WatchError::Create(message));
        }
        // The receiver of the real watcher is dropped with this closure: nothing
        // writes to its directory, so it reports nothing either.
        let (watcher, _unused) = SessionWatcher::start(self.empty.path())
            .map_err(|error| WatchError::Create(error.to_string()))?;
        let (batches, of_batch) = channel();
        *lock(&self.batches) = Some(batches);
        Ok((watcher, of_batch))
    }
}

/// A run wired the way the app wires it: the fake watcher source, a pass that
/// counts itself, and the recorder of every slice pushed.
pub struct Wired {
    pub service: Arc<SessionsService>,
    pub holder: Arc<AppStateHolder>,
    pub recorder: Recorder,
    source: Arc<FakeSource>,
    passes: Arc<AtomicUsize>,
    /// Every answer the flow gave the timer, in order.
    seen: Arc<Mutex<Vec<(bool, Duration)>>>,
}

impl Wired {
    /// How many passes the run started.
    pub fn passes(&self) -> usize {
        self.passes.load(Ordering::SeqCst)
    }

    pub fn source(&self) -> &Arc<FakeSource> {
        &self.source
    }

    /// Every delay the safety net read from the flow, as `(is_watching, delay)`.
    pub fn delays(&self) -> Vec<(bool, Duration)> {
        lock(&self.seen).clone()
    }

    /// Waits for `count` passes to have started.
    pub fn wait_passes(&self, count: usize) {
        wait_until(
            LIMIT,
            || self.passes() >= count,
            || format!("{count} passes, saw {}", self.passes()),
        );
    }

    /// Waits for the watcher to have been asked for `count` times.
    pub fn wait_starts(&self, count: usize) {
        wait_until(
            LIMIT,
            || self.source.starts() >= count,
            || format!("{count} watcher starts, saw {}", self.source.starts()),
        );
    }

    /// Waits for the timer to have read this answer from the flow after the
    /// first `after` answers it already gave, so a test says which side of a
    /// change it means.
    pub fn wait_delay(&self, after: usize, watching: bool, delay: Duration) {
        wait_until(
            LIMIT,
            || {
                self.delays()
                    .iter()
                    .skip(after)
                    .any(|&read| read == (watching, delay))
            },
            || {
                format!(
                    "the timer to read {watching}/{delay:?}, saw {:?}",
                    self.delays()
                )
            },
        );
    }

    /// Waits, with the live deadline, until the pushed list calls itself
    /// complete with `total` sessions, and answers that slice.
    pub fn wait_total(&self, total: u32) -> ViewState {
        wait_until(
            LIVE,
            || self.total() == Some(total),
            || format!("the list to say {total}, saw {:?}", self.recorder.pushes()),
        );
        self.recorder
            .pushes()
            .pop()
            .expect("the list that settled the wait")
    }

    /// The total of the last pushed list, when that list calls itself complete.
    fn total(&self) -> Option<u32> {
        self.recorder
            .pushes()
            .last()
            .filter(|last| last.list_status == ListStatusView::Complete)
            .map(|last| last.total_sessions)
    }
}

/// A run over `home`, with the fake watcher and a safety net that waits `delay`:
/// a long one in the tests where only a batch may run a pass.
pub fn wired(home: &Home, delay: Duration) -> Wired {
    let (source, seen) = (
        Arc::new(FakeSource::new()),
        Arc::new(Mutex::new(Vec::new())),
    );
    let wiring = Wiring {
        start: source.seam(),
        delay: recording(delay, Arc::clone(&seen)),
    };
    wired_with(session_seam::flow_of(home), wiring, source, seen, None)
}

/// The same over a run with no pi home at all: no index, no sessions directory.
pub fn homeless(delay: Duration) -> Wired {
    let (source, seen) = (
        Arc::new(FakeSource::new()),
        Arc::new(Mutex::new(Vec::new())),
    );
    let wiring = Wiring {
        start: source.seam(),
        delay: recording(delay, Arc::clone(&seen)),
    };
    wired_with(
        Flow::new(None, None, None, None),
        wiring,
        source,
        seen,
        None,
    )
}

/// The same with the backend's real `notify` watcher and the flow's own delay:
/// the live run of the process-level test.
pub fn wired_real(home: &Home) -> Wired {
    let (source, seen) = (
        Arc::new(FakeSource::new()),
        Arc::new(Mutex::new(Vec::new())),
    );
    wired_with(
        session_seam::flow_of(home),
        Wiring::real(),
        source,
        seen,
        None,
    )
}

/// The same with a pass that blocks until the test releases it, so a test can
/// change the sessions while a pass holds the index. The gate it answers with is
/// what names the passes and lets them go.
pub fn wired_gated(home: &Home, delay: Duration) -> (gate::Gated, Wired) {
    let (source, seen) = (
        Arc::new(FakeSource::new()),
        Arc::new(Mutex::new(Vec::new())),
    );
    let (gated, run) = gate::gated_run();
    let wiring = Wiring {
        start: source.seam(),
        delay: recording(delay, Arc::clone(&seen)),
    };
    let wired = wired_with(session_seam::flow_of(home), wiring, source, seen, Some(run));
    (gated, wired)
}

/// One service over `flow`, armed with a recorder and counting its passes.
fn wired_with(
    flow: Flow,
    wiring: Wiring,
    source: Arc<FakeSource>,
    seen: Arc<Mutex<Vec<(bool, Duration)>>>,
    run: Option<RunPass>,
) -> Wired {
    let run = run.unwrap_or_else(|| startup::run_pass(flow.shared()));
    let (run, passes) = count(run);
    let holder = Arc::new(AppStateHolder::default());
    let service = SessionsService::wired(Arc::clone(&holder), flow, run, wiring);
    let recorder = Recorder::default();
    service.arm(recorder.push());
    Wired {
        service: Arc::new(service),
        holder,
        recorder,
        source,
        passes,
        seen,
    }
}

/// A pass that counts how many times it was asked to run.
fn count(run: RunPass) -> (RunPass, Arc<AtomicUsize>) {
    let passes = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&passes);
    let counted: RunPass = Arc::new(move |job, emit| {
        counter.fetch_add(1, Ordering::SeqCst);
        run(job, emit);
    });
    (counted, passes)
}

/// The safety net's delay: `fixed` every time, with every answer the flow gave
/// recorded, so a test never waits out the real 5 s poll and can still see the
/// cadence follow the watcher.
fn recording(fixed: Duration, seen: Arc<Mutex<Vec<(bool, Duration)>>>) -> TickDelay {
    Arc::new(move |flow: &Flow| {
        lock(&seen).push((flow.is_watching(), flow.tick_delay()));
        fixed
    })
}

/// Polls `done` until it holds, failing after `limit` with what `report` says.
fn wait_until(limit: Duration, done: impl Fn() -> bool, report: impl Fn() -> String) {
    let deadline = Instant::now() + limit;
    while !done() {
        assert!(
            Instant::now() < deadline,
            "waited {limit:?} for {}",
            report()
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// The path of a session file the tests name in a batch, so the batch is what
/// the watcher would have reported for it.
pub fn session_file(home: &Home, project: &str, name: &str) -> PathBuf {
    home.sessions().join(project).join(name)
}
