//! Bringing the children's events back into the pushed state (S2).
//!
//! The ROLE is the iced shell's (`crates/ui/src/subscription.rs::child_output`,
//! `app/chat_update.rs::on_chat`, `app/chat_closed.rs`), never its code: one
//! dedicated thread blocks on the tagged stream of every child, translates each
//! event with [`ChatRuntime::convert`] and the runtime clock, and a second one
//! folds the matching reducer input and pushes the slice as `state-changed`.
//! Neither runs on an async command or on one of the window's threads: a child
//! takes seconds to speak, and a blocked thread must never be the one that draws.
//!
//! The seam is [`spawn_pump`]: it takes a plain channel of ALREADY TRANSLATED
//! values and the emit step as a closure, so a test feeds events without a child
//! and records the slices without a window.
//!
//! [`EventPump`] is where the two threads start, exactly once and lazily: the
//! first of `arm` (the window's emit step, installed at startup) and `feed` (the
//! tagged stream plus the runtime, handed over when `RuntimeExecutor` first
//! builds its host) that finds the other one there starts them.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;

use gentle_app::Input;
use gentle_backend::chat_host::TaggedEvent;
use gentle_runtime::chat_runtime::{ChatRuntime, Converted};
use gentle_runtime::clock;

use crate::bridge::AppStateHolder;
use crate::contract::ViewState;

/// The one push of the pump: `state-changed` in the running app, a recorder in
/// a test.
pub type Push = Box<dyn Fn(ViewState) + Send + 'static>;

/// The translated stream and the runtime that translated it.
struct Tagged {
    runtime: Arc<ChatRuntime>,
    events: Receiver<TaggedEvent>,
}

/// The one place the event threads start, and the reason they start once.
#[derive(Default)]
pub struct EventPump {
    holder: Arc<AppStateHolder>,
    push: Mutex<Option<Push>>,
    tagged: Mutex<Option<Tagged>>,
    started: Mutex<bool>,
}

impl EventPump {
    /// A pump that folds into `holder`, with no emit step and no source yet:
    /// both arrive during startup, in either order.
    pub fn new(holder: Arc<AppStateHolder>) -> Self {
        Self {
            holder,
            ..Self::default()
        }
    }

    /// The window's emit step, installed once at startup.
    pub fn arm(&self, push: Push) {
        *lock(&self.push) = Some(push);
        self.start_once();
    }

    /// The stream of every child's tagged events and the runtime that reads it,
    /// handed over once, when the executor builds its host.
    pub fn feed(&self, runtime: Arc<ChatRuntime>, events: Receiver<TaggedEvent>) {
        *lock(&self.tagged) = Some(Tagged { runtime, events });
        self.start_once();
    }

    /// Starts the translating thread and the pump behind it when both halves are
    /// here. Called from both `arm` and `feed`, so whichever arrives last starts
    /// them; the started flag keeps the other call, and every later one, out.
    fn start_once(&self) {
        let mut started = lock(&self.started);
        if *started {
            return;
        }
        // Check BOTH halves are there before taking either: taking one while the
        // other is missing would throw it away and the pump could never start.
        if lock(&self.push).is_none() || lock(&self.tagged).is_none() {
            return;
        }
        let (Some(push), Some(tagged)) = (push_of(&self.push), tagged_of(&self.tagged)) else {
            return;
        };
        *started = true;
        let (into, from) = channel();
        spawn_translator(tagged, into);
        match spawn_pump(from, Arc::clone(&self.holder), push) {
            Ok(_) => {}
            // Nothing can fall back: the executor keeps building children, so
            // the missing pump is a log line, never a panic. Its channel is
            // dropped here, and the translator ends on its first failed send.
            Err(error) => eprintln!("child event pump unavailable: {error}"),
        }
    }
}

/// The emit step, taken out: it is used once, by the pump's thread.
fn push_of(push: &Mutex<Option<Push>>) -> Option<Push> {
    lock(push).take()
}

/// The tagged stream and its runtime, taken out for the translator's thread.
fn tagged_of(tagged: &Mutex<Option<Tagged>>) -> Option<Tagged> {
    lock(tagged).take()
}

/// The translating thread: blocks on the tagged stream, converts each event and
/// hands it to the pump. Ends when the stream disconnects - every child host is
/// gone - so the pump behind it ends too, with no busy loop and no panic.
fn spawn_translator(tagged: Tagged, into: Sender<Converted>) {
    let spawned = std::thread::Builder::new()
        .name("chat-events-translate".to_owned())
        .spawn(move || {
            let Tagged { runtime, events } = tagged;
            for event in events {
                let Some(converted) = runtime.convert(event, clock::now_ms()) else {
                    continue; // a chat nobody knows, or a line we do not model
                };
                if into.send(converted).is_err() {
                    break; // the pump is gone: nobody is left to fold
                }
            }
        });
    if let Err(error) = spawned {
        eprintln!("child event translator unavailable: {error}");
    }
}

/// The pump on its own thread. Its handle is what a test joins with a timeout,
/// and dropping the sender of `source` is what ends it.
pub fn spawn_pump(
    source: Receiver<Converted>,
    holder: Arc<AppStateHolder>,
    push: Push,
) -> std::io::Result<JoinHandle<()>> {
    std::thread::Builder::new()
        .name("chat-events".to_owned())
        .spawn(move || pump(source, &holder, push))
}

/// The pump itself: one translated event in, one pushed slice out, until the
/// channel disconnects. Folding goes through the same reducer the intents use.
pub fn pump(source: Receiver<Converted>, holder: &AppStateHolder, push: Push) {
    for converted in source {
        let input = input_of(converted);
        push(holder.fold_input(input));
    }
}

/// The reducer input one translated event folds as, named the way the iced
/// shell's `on_chat` names it: a decoded event, one stderr line, a closed child.
fn input_of(converted: Converted) -> Input {
    match converted {
        Converted::Input(input) => input,
        Converted::Stderr { chat, line } => {
            eprintln!("[{}] {line}", chat.label());
            Input::ChatStderr { chat, line }
        }
        Converted::Closed { chat, expected } => {
            if expected {
                eprintln!("[{}] child stopped by the host", chat.label());
            } else {
                eprintln!("[{}] child exited", chat.label());
            }
            Input::ChatClosed { chat, expected }
        }
    }
}

/// A lock that survives a poisoned mutex: nothing here may panic because a
/// thread died holding one.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}
