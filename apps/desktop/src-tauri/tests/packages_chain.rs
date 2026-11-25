//! The packages chain's two halves meeting (packages-status T1, S2, S7, T7).
//!
//! The live run in `live_packages.rs` proves the chain against the real shell.
//! This file is the same chain's cheap half: no child, no network, no config - the
//! facts that live inside the app's own plumbing.
//!
//! [`the_pumps_runtime_outlives_the_stream_it_hands_over`] is the regression test
//! for the defect the live run found (L3): the pump's translator TAKES the tagged
//! stream when it starts, and the runtime was kept inside that same value - so
//! `EventPump::runtime()`, which every packages read and the run's own check ask
//! the gate through, answered `None` for the whole life of a run whose child was
//! alive and talking. Every call site that only *wants* to ask a child then
//! declined (`PackagesRead::Unasked`), which is exactly the maintainer's report:
//! chats open, children working, and the Extensions block stuck in its "this
//! shell cannot tell" shape.
//!
//! The child answer itself is not scripted here - the stand-in does not speak
//! `/gentle:desktop` - so these tests assert the pump and the gate, and
//! `crates/runtime`'s own `packages` tests assert the wire.

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gentle_app::{ChatCommand, ChatId};
use gentle_backend::chat_host::{
    ChatHost, ChatKey, Child, Clock, Limits, ManualClock, Spawner, TaggedEvent,
};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::{UiIntent, ViewState};
use gentle_desk_lib::events::{EventPump, Push};
use gentle_runtime::chat_runtime::{ChatRuntime, Outcome};

/// A spawner that starts nothing: these tests never need a child, and "nothing
/// ran" is what its counter says.
struct NoChild {
    spawns: Arc<AtomicUsize>,
}

impl Spawner for NoChild {
    fn spawn(&self, _key: &ChatKey, _folder: Option<&Path>) -> std::io::Result<Box<dyn Child>> {
        self.spawns.fetch_add(1, Ordering::SeqCst);
        Err(std::io::Error::other("this test starts no child"))
    }

    fn interactive(&self) -> bool {
        true
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// A runtime with no children, the stream a pump reads, and how many children its
/// spawner was asked for.
fn runtime() -> (Arc<ChatRuntime>, Receiver<TaggedEvent>, Arc<AtomicUsize>) {
    let spawns = Arc::new(AtomicUsize::new(0));
    let host = ChatHost::with_parts(
        Limits::default(),
        Arc::new(NoChild {
            spawns: Arc::clone(&spawns),
        }) as Arc<dyn Spawner>,
        Arc::new(ManualClock::new()) as Arc<dyn Clock>,
    );
    let (runtime, events) = ChatRuntime::new(host);
    (Arc::new(runtime), events, spawns)
}

/// packages-status T1, S2, S7 - the defect the live run found: the runtime the
/// pump is fed must still be there after the pump's threads take the stream.
///
/// The Settings load's packages read and the run's automatic check both reach a
/// child THROUGH `EventPump::runtime()`, and both are read on a run whose pump has
/// long started (that is the whole point: a live child). A pump that hands its
/// runtime away with the stream answers "no host" forever, and every read
/// declines.
#[test]
fn the_pumps_runtime_outlives_the_stream_it_hands_over() {
    let (runtime, events, _spawns) = runtime();
    let pump = Arc::new(EventPump::new(Arc::new(AppStateHolder::default())));

    // The order the app takes them in: the host is built and fed (no emit step
    // exists yet at that moment), and the window's push arrives at startup.
    pump.feed(Arc::clone(&runtime), events);
    pump.arm(Box::new(|_view| {}) as Push);

    // Both halves are in, so the translator took the stream on that call; the
    // runtime is what must outlive it.
    let kept = pump
        .runtime()
        .expect("the runtime the pump was fed is still there after the pump started");
    assert!(
        Arc::ptr_eq(&kept, &runtime),
        "the pump answers the very runtime it was fed"
    );

    pump.stop();
    assert!(
        pump.join(Duration::from_secs(3)),
        "the pump's threads end on the stop signal"
    );
}

/// packages-status T7: the gate names the chat the runtime really opened, on a
/// STARTED pump - the two facts the live chain needs and the two the old
/// `runtime()` broke apart.
#[test]
fn a_started_pump_still_names_the_chat_the_runtime_opened() {
    let (runtime, events, spawns) = runtime();
    let holder = Arc::new(AppStateHolder::default());
    let pump = Arc::new(EventPump::new(Arc::clone(&holder)));
    let pushed: Arc<Mutex<Vec<ViewState>>> = Arc::new(Mutex::new(Vec::new()));
    let push: Push = {
        let kept = Arc::clone(&pushed);
        Box::new(move |view| lock(&kept).push(view))
    };
    pump.feed(Arc::clone(&runtime), events);
    pump.arm(push);
    // The app's own lazy start: the click only ALLOCATES the chat.
    let view = holder.dispatch_commands(UiIntent::OpenNewChat).0;
    let fresh = ChatId::of_text(
        view.foreground_chat
            .as_deref()
            .expect("a new chat is in front"),
    )
    .expect("a fresh chat is the app's own id form");

    // The composer's ask: the runtime keys the chat before it tries to start it,
    // and this spawner starts nothing, so no child exists to ask.
    let ran = runtime.execute(ChatCommand::OpenProcess {
        chat: fresh.clone(),
    });
    assert!(
        matches!(ran, Outcome::OpenFailed { .. }),
        "this spawner starts no child: {ran:?}"
    );
    assert_eq!(spawns.load(Ordering::SeqCst), 1, "one ask, one spawn try");

    let kept = pump.runtime().expect("the runtime outlives the start");
    assert_eq!(
        kept.packages_chat(Some(&fresh)),
        Some(fresh),
        "the gate names the fresh chat the host has a key for"
    );

    pump.stop();
    let _ = pump.join(Duration::from_secs(3));
}
