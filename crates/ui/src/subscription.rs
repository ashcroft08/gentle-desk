//! What the app listens to: the child's output, the refresh timer, window close.

use std::hash::{Hash, Hasher};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gentle_backend::process::ProcessEvent;
use iced::{stream, window, Subscription};

use crate::message::Message;

/// The output stream of the child, handed to the subscription once.
#[derive(Clone, Default)]
pub struct EventSource(Arc<Mutex<Option<Receiver<ProcessEvent>>>>);

impl EventSource {
    pub fn fill(&self, events: Receiver<ProcessEvent>) {
        *self.0.lock().unwrap() = Some(events);
    }
}

impl Hash for EventSource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // One source per run: a constant identity keeps the subscription alive.
        "gentle-shell-events".hash(state);
    }
}

pub fn subscription(source: &EventSource) -> Subscription<Message> {
    Subscription::batch([
        child_output(source.clone()),
        // Sessions change outside this window too (other terminals, other apps).
        iced::time::every(Duration::from_secs(5)).map(|_| Message::Tick),
        window::close_requests().map(|_| Message::CloseRequested),
    ])
}

fn child_output(source: EventSource) -> Subscription<Message> {
    Subscription::run_with(source, |source| {
        let receiver = source.0.lock().unwrap().take();
        stream::channel(1024, async move |mut output| {
            use iced::futures::{executor::block_on, SinkExt};
            let Some(receiver) = receiver else { return };
            // A plain thread blocks on the std channel; `send` applies
            // backpressure instead of dropping events.
            std::thread::spawn(move || {
                for event in receiver {
                    if block_on(output.send(Message::Process(event))).is_err() {
                        break;
                    }
                }
            });
            std::future::pending::<()>().await;
        })
    })
}
