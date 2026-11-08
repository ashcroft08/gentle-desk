//! What the app listens to: every child's tagged events, the watcher's batches,
//! the refresh timer, window close.

use std::hash::{Hash, Hasher};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gentle_backend::chat_host::TaggedEvent;
use iced::{stream, window, Subscription};

use crate::message::Message;
use crate::watch_source::{self, WatchSource};
use gentle_runtime::chat_runtime::ChatRuntime;
use gentle_runtime::clock;

/// The one stream every child's events arrive on, with the runtime that knows
/// which chat each event belongs to. Handed to the subscription once.
#[derive(Clone, Default)]
pub struct EventSource(Arc<Mutex<Option<ChatEvents>>>);

/// The receiver and the translator, which must travel together: translating an
/// event needs the chat registry the runtime holds.
struct ChatEvents {
    runtime: Arc<ChatRuntime>,
    events: Receiver<TaggedEvent>,
}

impl EventSource {
    /// Hands over the tagged stream of every child and the runtime that reads it.
    pub fn fill(&self, runtime: Arc<ChatRuntime>, events: Receiver<TaggedEvent>) {
        *self.0.lock().unwrap() = Some(ChatEvents { runtime, events });
    }
}

impl Hash for EventSource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // One source per run: a constant identity keeps the subscription alive.
        "gentle-shell-events".hash(state);
    }
}

/// Everything the window listens to. `delay` is the next full pass and comes
/// from the app state: iced rebuilds the timer when the duration changes.
pub fn subscription(
    source: &EventSource,
    watch: &WatchSource,
    delay: Duration,
) -> Subscription<Message> {
    Subscription::batch([
        child_output(source.clone()),
        watch_source::watch_source(watch.clone()),
        // Watcher batches cover what changed; a slow pass also reconciles what
        // a watcher can miss, and is the only update without one.
        iced::time::every(delay).map(|_| Message::Tick),
        window::close_requests().map(|_| Message::CloseRequested),
    ])
}

fn child_output(source: EventSource) -> Subscription<Message> {
    Subscription::run_with(source, |source| {
        let chat_events = source.0.lock().unwrap().take();
        stream::channel(1024, async move |mut output| {
            use iced::futures::{executor::block_on, SinkExt};
            let Some(ChatEvents { runtime, events }) = chat_events else {
                return;
            };
            // A plain thread blocks on the std channel; `send` applies
            // backpressure instead of dropping events.
            std::thread::spawn(move || {
                for tagged in events {
                    let converted = runtime.convert(tagged, clock::now_ms());
                    let Some(converted) = converted else {
                        continue; // a chat nobody knows, or a line we do not model
                    };
                    if block_on(output.send(Message::Chat(converted))).is_err() {
                        break;
                    }
                }
            });
            std::future::pending::<()>().await;
        })
    })
}
