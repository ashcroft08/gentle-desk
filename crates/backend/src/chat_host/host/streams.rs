//! The one ordered stream every child's events arrive on: handing it to its
//! single consumer, polling it, and the forwarder thread per child that feeds it.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use super::super::TaggedEvent;
use super::state::Tag;
use super::ChatHost;
use crate::process::ProcessEvent;

impl ChatHost {
    /// Hands the tagged event stream to another owner. What stays behind is a
    /// closed stream, so there is never a second consumer.
    pub fn take_events(&mut self) -> Receiver<TaggedEvent> {
        let mut guard = self.events_rx.lock().unwrap_or_else(|p| p.into_inner());
        std::mem::replace(&mut *guard, mpsc::channel().1)
    }

    /// The next tagged event, for a caller that polls instead of owning the
    /// stream.
    pub fn recv_timeout(&self, wait: Duration) -> Result<TaggedEvent, RecvTimeoutError> {
        self.events_rx
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .recv_timeout(wait)
    }

    /// Forwards one child's event stream onto the host's channel, tagging every
    /// event with its chat and saying whether its closure is one the host asked
    /// for, and clears the child's alive flag when it goes.
    ///
    /// A forwarder that cannot be started is not left silent: the child is
    /// marked gone and one unexpected `Closed` is published at once, so the shell
    /// offers a Restart instead of waiting forever for a chat that will never
    /// answer (R4-003).
    pub(super) fn forward(
        &self,
        tag: Tag,
        alive: Arc<AtomicBool>,
        stopped: Arc<AtomicBool>,
        events: Receiver<ProcessEvent>,
    ) {
        let sender = self.events_tx.clone();
        let initial_key = tag.lock().unwrap_or_else(|p| p.into_inner()).clone();
        let watched = Arc::clone(&alive);
        let spawned = thread::Builder::new()
            .name("chat-host-events".to_owned())
            .spawn(move || {
                while let Ok(event) = events.recv() {
                    let closed = event == ProcessEvent::Closed;
                    if closed {
                        watched.store(false, Ordering::SeqCst);
                    }
                    let key = tag.lock().unwrap_or_else(|p| p.into_inner()).clone();
                    let tagged = TaggedEvent {
                        key,
                        event,
                        expected: closed && stopped.load(Ordering::SeqCst),
                    };
                    if sender.send(tagged).is_err() {
                        break; // the host is gone
                    }
                }
                watched.store(false, Ordering::SeqCst);
            });
        if spawned.is_err() {
            alive.store(false, Ordering::SeqCst);
            let _ = self.events_tx.send(TaggedEvent {
                key: initial_key,
                event: ProcessEvent::Closed,
                expected: false,
            });
        }
    }
}
