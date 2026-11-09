//! The seam of the session list tests, shared by them: a temporary pi home, a
//! recorder for the slices a pass pushes, and a fake pass that blocks until the
//! test releases it.
//!
//! Nothing here touches the user's home, starts a child or opens a window: a
//! temp directory holds the session files, and the emit step is a recorder. The
//! fake blocked pass lives in [`gate`].
#![allow(dead_code)] // each test binary uses one part of it

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use gentle_backend::session_index::SessionIndex;
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::{ListStatusView, ViewState};
use gentle_desk_lib::events::Push;
use gentle_desk_lib::sessions::service::RunPass;
use gentle_desk_lib::sessions::{startup, SessionsService};
use gentle_runtime::refresh_flow::Flow;
use gentle_runtime::session_refresh::Shared;

pub mod gate;
pub use gate::gated;

/// How long a test waits for a pass to report or a thread to answer.
pub const LIMIT: Duration = Duration::from_secs(20);

const HEADER: &str = r#"{"type":"session","version":3,"id":"s1","timestamp":"2026-01-01T00:00:00.000Z","cwd":"C:\\dev\\p"}"#;

fn message(text: &str) -> String {
    format!(
        r#"{{"type":"message","id":"m","timestamp":"2026-01-01T00:01:00.000Z","message":{{"role":"user","content":"{text}"}}}}"#
    )
}

/// A pi home with a `sessions/` directory and one file per session.
pub struct Home {
    dir: tempfile::TempDir,
}

impl Home {
    /// An empty directory: no sessions directory until a session is written, so
    /// a test can name a home whose list really has nothing to show.
    pub fn new() -> Self {
        Self {
            dir: tempfile::tempdir().expect("a temp home"),
        }
    }

    pub fn sessions(&self) -> PathBuf {
        self.dir.path().join("sessions")
    }

    /// A session file with a header and one user message, in `project`'s
    /// directory.
    pub fn session(&self, project: &str, name: &str, first: &str) {
        let dir = self.sessions().join(project);
        fs::create_dir_all(&dir).expect("the project directory");
        let body = format!("{HEADER}\n{}\n", message(first));
        fs::write(dir.join(name), body).expect("the session file");
    }

    /// `count` session files in one project, each with its own first message.
    pub fn many(&self, count: usize) {
        for index in 0..count {
            self.session("p", &format!("s{index:03}.jsonl"), &format!("chat {index}"));
        }
    }

    pub fn cache(&self) -> PathBuf {
        self.dir.path().join("data").join("session-index.json")
    }
}

/// The refresh flow of a home: its index, its cache and no watcher (the watcher
/// is not part of this slice).
pub fn flow_of(home: &Home) -> Flow {
    let index = SessionIndex::new(home.sessions());
    let shared = Shared::new(Arc::new(Mutex::new(index)), Some(home.cache()));
    Flow::new(Some(shared), Some(home.sessions()), None, None)
}

/// A service over `home` that runs the REAL pass, a holder to fold into, and the
/// recorder of what it pushes.
pub fn service(home: &Home) -> (SessionsService, Arc<AppStateHolder>, Recorder) {
    let flow = flow_of(home);
    let run = startup::run_pass(flow.shared());
    build(flow, run)
}

/// The same for a run with no pi home at all: the flow has no index, so it never
/// asks for a pass.
pub fn homeless() -> (SessionsService, Arc<AppStateHolder>, Recorder) {
    let flow = Flow::new(None, None, None, None);
    let run = startup::run_pass(None);
    build(flow, run)
}

pub fn build(flow: Flow, run: RunPass) -> (SessionsService, Arc<AppStateHolder>, Recorder) {
    let holder = Arc::new(AppStateHolder::default());
    let service = SessionsService::new(Arc::clone(&holder), flow, run);
    let recorder = Recorder::default();
    service.arm(recorder.push());
    (service, holder, recorder)
}

/// Every slice a service pushed, in order.
#[derive(Clone, Default)]
pub struct Recorder {
    pushes: Arc<Mutex<Vec<ViewState>>>,
}

impl Recorder {
    /// The emit step handed to the service in place of `state-changed`.
    pub fn push(&self) -> Push {
        let pushes = Arc::clone(&self.pushes);
        Box::new(move |view| {
            pushes.lock().expect("pushes mutex").push(view);
        })
    }

    pub fn pushes(&self) -> Vec<ViewState> {
        self.pushes.lock().expect("pushes mutex").clone()
    }

    /// Waits until the last push calls the list complete, then answers them all.
    pub fn wait_for_complete(&self) -> Vec<ViewState> {
        self.wait_until(|pushes| {
            pushes
                .last()
                .is_some_and(|view| view.list_status == ListStatusView::Complete)
        })
    }

    /// Waits until `done` holds for the recorded pushes.
    pub fn wait_until(&self, done: impl Fn(&[ViewState]) -> bool) -> Vec<ViewState> {
        let deadline = Instant::now() + LIMIT;
        loop {
            let pushes = self.pushes();
            if done(&pushes) {
                return pushes;
            }
            assert!(
                Instant::now() < deadline,
                "no push settled the list within {LIMIT:?}; got {} pushes",
                pushes.len()
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// How many pushes the recorder holds right now, without waiting.
    pub fn count(&self) -> usize {
        self.pushes.lock().expect("pushes mutex").len()
    }
}

/// The file name of one row's id, which is the session's path.
pub fn file_name(row_id: &str) -> &str {
    Path::new(row_id)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
}
