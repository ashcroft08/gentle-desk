//! A fake pass that blocks until the test releases it, so a test can ask for a
//! second refresh while the first pass still holds the index.
//!
//! The fake replaces the real pass of [`crate::session_seam`]: it records the
//! jobs it was asked for, counts how many run at once, and reports one empty
//! complete list when the test lets it go.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gentle_app::prepare_sessions;
use gentle_backend::session_index::RefreshStats;
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::sessions::service::RunPass;
use gentle_desk_lib::sessions::SessionsService;
use gentle_runtime::refresh_queue::Job;
use gentle_runtime::session_refresh::{Refreshed, Update};

use super::{flow_of, Home, Recorder, LIMIT};

/// How many passes ran and how many ran at once.
#[derive(Default)]
pub struct Counts {
    /// The passes running right now.
    at_once: std::sync::atomic::AtomicUsize,
    /// The most that ever ran at once: more than one is the bug the flow exists
    /// to prevent.
    most: std::sync::atomic::AtomicUsize,
    /// The jobs the passes were asked for, in order.
    jobs: Mutex<Vec<Job>>,
}

impl Counts {
    pub fn most_at_once(&self) -> usize {
        self.most.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn jobs(&self) -> Vec<Job> {
        self.jobs.lock().expect("jobs mutex").clone()
    }
}

/// A run whose passes are fake and blocked until the test releases them.
pub struct Gated {
    counts: Arc<Counts>,
    entered: Receiver<()>,
    release: Sender<()>,
}

/// A gated service over `home`, the gate that drives it and the recorder.
pub fn gated(home: &Home) -> (Gated, SessionsService, Arc<AppStateHolder>, Recorder) {
    let (gate, run) = gated_run();
    let (service, holder, recorder) = super::build(flow_of(home), run);
    (gate, service, holder, recorder)
}

/// The fake pass alone, with the gate that drives it: a test that wires its own
/// service around it (the wiring tests) takes this pair.
pub fn gated_run() -> (Gated, RunPass) {
    let (entered_tx, entered) = channel();
    let (release, release_rx) = channel();
    let counts = Arc::new(Counts::default());
    let run = fake_pass(Arc::clone(&counts), entered_tx, Mutex::new(release_rx));
    (
        Gated {
            counts,
            entered,
            release,
        },
        run,
    )
}

impl Gated {
    /// Waits for a pass to enter, blocking on it as the real one blocks on disk.
    pub fn enter(&self) {
        self.entered
            .recv_timeout(LIMIT)
            .expect("a pass starts within the limit");
    }

    /// Lets the pass that entered finish.
    pub fn release(&self) {
        self.release.send(()).expect("a pass waits for its release");
    }

    pub fn counts(&self) -> Arc<Counts> {
        Arc::clone(&self.counts)
    }
}

/// The fake pass: it reports one empty complete list and blocks in between.
fn fake_pass(counts: Arc<Counts>, entered: Sender<()>, release: Mutex<Receiver<()>>) -> RunPass {
    Arc::new(move |job: Job, emit: &mut dyn FnMut(Update)| {
        let at_once = counts.at_once.fetch_add(1, ORDER) + 1;
        counts.most.fetch_max(at_once, ORDER);
        counts.jobs.lock().expect("jobs mutex").push(job.clone());
        let _ = entered.send(());
        let released = release.lock().expect("release mutex").recv_timeout(LIMIT);
        counts.at_once.fetch_sub(1, ORDER);
        released.expect("the test releases every pass");
        emit(Update::Refreshed(Ok(Refreshed {
            sessions: prepare_sessions(Vec::new()),
            took: Duration::ZERO,
            full: matches!(job, Job::Full),
            stats: RefreshStats::default(),
        })));
    })
}

/// The ordering every counter here uses: the test reads them after a channel
/// handover, so the count it sees is the one that happened.
use std::sync::atomic::Ordering::SeqCst as ORDER;
