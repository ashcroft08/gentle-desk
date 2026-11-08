//! Turning what a child says into something the app can take.

use gentle_app::{ChatId, Input};
use gentle_backend::chat_host::TaggedEvent;
use gentle_backend::process::ProcessEvent;
use gentle_core::rpc::Decoded;

use super::ChatRuntime;

/// One child event, translated.
#[derive(Debug, Clone, PartialEq)]
pub enum Converted {
    /// A decoded protocol event, ready for the reducer with its arrival time.
    Input(Input),
    /// A diagnostic line from the child's stderr.
    Stderr { chat: ChatId, line: String },
    /// The child's output ended: it is gone or going. `expected` is true when the
    /// host stopped it on purpose (idle reap, cap, close, shutdown), which is the
    /// difference between a chat that is just idle and one that died (S3).
    Closed { chat: ChatId, expected: bool },
}

impl ChatRuntime {
    /// Translates one tagged event. `None` for an event of a chat nobody
    /// registered and for a stdout line that is not a known event.
    pub fn convert(&self, tagged: TaggedEvent, now_ms: u64) -> Option<Converted> {
        let TaggedEvent {
            key,
            event,
            expected,
        } = tagged;
        let chat = self.keys().id_of(&key)?;
        match event {
            ProcessEvent::Output(Decoded::Event(event)) => {
                Some(Converted::Input(Input::ChatEventAt {
                    chat,
                    event,
                    now_ms,
                }))
            }
            ProcessEvent::Output(Decoded::Unknown(_)) => None,
            ProcessEvent::Stderr(line) => Some(Converted::Stderr { chat, line }),
            ProcessEvent::Closed => Some(Converted::Closed { chat, expected }),
        }
    }
}
