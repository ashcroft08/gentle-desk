//! Bridging the session watcher into the app.
//!
//! The watcher (`gentle_backend::session_watch`) reports debounced [`Batch`]es
//! of changed session files on a plain channel. A subscription turns them into
//! [`Message::SessionsChanged`] and, when the watcher goes away, one
//! [`Message::WatcherLost`] so the app can fall back to polling.
//!
//! iced identifies a subscription by what it is built from, so the identity
//! here is the generation: starting a new watcher bumps it, which gives the
//! subscription a fresh stream reading the new receiver. A receiver is taken
//! out of the slot once, exactly like the child's output (`EventSource`).

use std::hash::{Hash, Hasher};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

use gentle_backend::session_watch::Batch;
use iced::{stream, Subscription};

use crate::message::Message;

/// The batches of the running watcher, if there is one.
#[derive(Clone, Default)]
pub struct WatchSource {
    slot: Arc<Mutex<Option<Receiver<Batch>>>>,
    /// Bumped for every new watcher: a new subscription identity, a new stream.
    generation: u32,
}

impl WatchSource {
    /// Hands over the batches of a newly started watcher.
    pub fn fill(&mut self, batches: Receiver<Batch>) {
        self.generation = self.generation.wrapping_add(1);
        *self.slot.lock().unwrap() = Some(batches);
    }
}

impl Hash for WatchSource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        "session-watch".hash(state);
        // Same watcher, same stream; a new watcher must not keep the old one.
        self.generation.hash(state);
    }
}

/// The watcher's batches as messages. Without a watcher the stream is inert.
pub fn watch_source(source: WatchSource) -> Subscription<Message> {
    Subscription::run_with(source, |source| {
        let batches = source.slot.lock().unwrap().take();
        stream::channel(64, async move |mut output| {
            use iced::futures::{executor::block_on, SinkExt};
            let Some(batches) = batches else { return };
            // A plain thread blocks on the std channel, so `send` applies
            // backpressure instead of dropping a batch.
            std::thread::spawn(move || {
                for batch in batches {
                    if block_on(output.send(Message::SessionsChanged(batch))).is_err() {
                        return;
                    }
                }
                // The watcher stopped: the app polls until one starts again.
                let _ = block_on(output.send(Message::WatcherLost));
            });
            std::future::pending::<()>().await;
        })
    })
}
