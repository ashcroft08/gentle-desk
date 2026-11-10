//! Windowless tests of what a close does to a run that is idle (T8a): stopping
//! new work, and a close with nothing live finishing at once.
//!
//! The real steps of the bridge over an executor that never opened a chat: no
//! child, no pump thread, no cache and no config path, so nothing here waits on
//! anything.

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use gentle_app::ChatCommand;
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::close::{BridgeSteps, CloseFlow, Deadlines, CLOSING};
use gentle_desk_lib::config::ConfigWriter;
use gentle_desk_lib::effects::{ChatExecutor, RuntimeExecutor};
use gentle_desk_lib::events::EventPump;
use gentle_desk_lib::pending::PendingFolders;
use gentle_runtime::chat_runtime::Outcome;

/// How long a test waits for a close that has nothing to wait for.
const LIMIT: Duration = Duration::from_secs(10);

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

#[test]
fn a_close_with_nothing_live_finishes_at_once() {
    // The real steps over a run that never opened a chat and never wrote a
    // config: no child, no pump thread, no cache, no config path.
    let holder = Arc::new(AppStateHolder::default());
    let pump = Arc::new(EventPump::new(Arc::clone(&holder)));
    let executor = Arc::new(RuntimeExecutor::new(
        Arc::clone(&pump),
        Arc::new(PendingFolders::default()),
        Arc::clone(&holder),
    ));
    let exited = Arc::new(Mutex::new(false));
    let steps = BridgeSteps::new(
        Arc::clone(&executor),
        pump,
        None,
        ConfigWriter::spawn(None),
        Box::new({
            let exited = Arc::clone(&exited);
            move || *lock(&exited) = true
        }),
    );
    let flow = CloseFlow::new(Arc::new(steps), Deadlines::default());

    let started = Instant::now();
    assert!(flow.request());
    let deadline = Instant::now() + LIMIT;
    while !*lock(&exited) {
        assert!(Instant::now() < deadline, "the close never exited");
        std::thread::sleep(Duration::from_millis(5));
    }

    assert!(
        started.elapsed() < Duration::from_secs(2),
        "a close with nothing live waited on a deadline"
    );
    assert!(executor.is_closed(), "the close stopped new work");
}

#[test]
fn a_closed_executor_starts_nothing_new() {
    let holder = Arc::new(AppStateHolder::default());
    let executor = RuntimeExecutor::new(
        Arc::new(EventPump::new(Arc::clone(&holder))),
        Arc::new(PendingFolders::default()),
        Arc::clone(&holder),
    );
    let chat = gentle_app::ChatId::of_session("/sessions/a.jsonl");

    assert!(!executor.is_closed(), "a fresh run accepts work");
    executor.stop_work();
    assert!(executor.is_closed());

    // The refusal comes before anything is resolved or started: a chat asked for
    // during the shutdown answers the closing line, never a launcher error and
    // never a child.
    match executor.execute(ChatCommand::OpenProcess { chat: chat.clone() }) {
        Outcome::OpenFailed {
            chat: failed,
            message,
        } => {
            assert_eq!(failed, chat);
            assert_eq!(message, CLOSING);
        }
        other => panic!("a closed run must refuse the open, got {other:?}"),
    }
}
