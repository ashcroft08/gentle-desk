//! The recording seam of the child-event pump tests, shared by them.
//!
//! A `Pump` is one pump thread fed by hand with translated values, recording
//! every slice it pushes: no child, no window, no launcher ever runs here.
#![allow(dead_code)] // each test binary uses one part of it

use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use gentle_app::{ChatCommand, ChatId, Input, Intent};
use gentle_core::rpc::{AssistantMessageEvent, RpcEvent, RpcMessage};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::ViewState;
use gentle_desk_lib::events::{spawn_pump, Push};
use gentle_runtime::chat_runtime::Converted;

/// How long a pump thread is given to end after its source closes.
const STOP_LIMIT: Duration = Duration::from_secs(5);

const SESSION_PATH: &str = "/sessions/a.jsonl";

/// A pump fed by hand, with the slices it pushed recorded in order and the
/// per-chat commands it handed to the holder's sink recorded too.
pub struct Pump {
    source: Sender<Converted>,
    pushed: Arc<Mutex<Vec<ViewState>>>,
    delivered: Arc<Mutex<Vec<ChatCommand>>>,
    thread: std::thread::JoinHandle<()>,
}

impl Pump {
    /// Starts the pump over `holder` and arms the chat-commands sink to record.
    ///
    /// A real run's sink starts one `chat-command` worker per command; here the
    /// only thing a test can observe is what was handed over, so nothing is
    /// spawned, no child exists and no pipe is written.
    pub fn start(holder: &Arc<AppStateHolder>) -> Self {
        let (source, receiver) = channel();
        let pushed = Arc::new(Mutex::new(Vec::new()));
        let recorded = Arc::clone(&pushed);
        let push: Push = Box::new(move |view| {
            recorded.lock().expect("pushed mutex").push(view);
        });
        let delivered = Arc::new(Mutex::new(Vec::new()));
        let recorded_commands = Arc::clone(&delivered);
        holder.arm_chat_commands(Arc::new(move |commands| {
            recorded_commands
                .lock()
                .expect("delivered mutex")
                .extend(commands);
        }));
        let thread =
            spawn_pump(receiver, Arc::clone(holder), push).expect("the pump thread starts");
        Self {
            source,
            pushed,
            delivered,
            thread,
        }
    }

    pub fn send(&self, converted: Converted) {
        self.source.send(converted).expect("the pump is listening");
    }

    /// Closes the source, which must end the thread, and answers what it pushed.
    pub fn finish(self) -> Vec<ViewState> {
        self.finish_recording().0
    }

    /// [`Self::finish`], and the per-chat commands the armed sink was handed on
    /// the way (chat-event-commands, S5).
    pub fn finish_recording(self) -> (Vec<ViewState>, Vec<ChatCommand>) {
        let Pump {
            source,
            pushed,
            delivered,
            thread,
        } = self;
        drop(source);
        join_within(thread);
        let pushed = Arc::try_unwrap(pushed)
            .expect("the pump thread ended, so it holds no record")
            .into_inner()
            .expect("pushed mutex");
        // Drained rather than unwrapped: the holder keeps the recording sink
        // armed, so it still holds a handle to the record after the pump ends.
        let delivered = std::mem::take(&mut *delivered.lock().expect("delivered mutex"));
        (pushed, delivered)
    }
}

/// Waits for the thread, failing the test if it outlives its closed source.
fn join_within(thread: std::thread::JoinHandle<()>) {
    let deadline = Instant::now() + STOP_LIMIT;
    while !thread.is_finished() {
        assert!(
            Instant::now() < deadline,
            "the pump thread kept running after its source closed"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    thread.join().expect("the pump thread never panics");
}

/// The holder with the stored chat opened and confirmed, so it is the
/// foreground chat and it is idle.
pub fn open_stored() -> (Arc<AppStateHolder>, ChatId) {
    let holder = Arc::new(AppStateHolder::default());
    let chat = ChatId::of_session(SESSION_PATH);
    holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
    holder.fold_input(Input::ChatOpened { chat: chat.clone() });
    (holder, chat)
}

/// One assistant turn as the child reports it: the message opens and its text
/// arrives as a delta.
pub fn assistant_message(chat: &ChatId, text: &str) -> Vec<Converted> {
    let message = RpcMessage {
        role: "assistant".to_owned(),
        raw: serde_json::json!({}),
    };
    vec![
        event_at(chat, RpcEvent::MessageStart(message), 7),
        event_at(
            chat,
            RpcEvent::MessageUpdate(AssistantMessageEvent::TextDelta {
                content_index: 0,
                delta: text.to_owned(),
            }),
            8,
        ),
    ]
}

/// The same turn as it ENDS, the way a provider that answered well closes it:
/// the message's own authoritative text, no failure flag.
pub fn assistant_turn(chat: &ChatId, text: &str) -> Vec<Converted> {
    let mut turn = assistant_message(chat, text);
    turn.push(message_end(
        chat,
        serde_json::json!({ "stopReason": "stop" }),
    ));
    turn
}

/// A turn the provider failed: the model's own text, and the `stopReason`
/// `error` the reducer keeps as the message's error.
pub fn errored_turn(chat: &ChatId, text: &str, error: &str) -> Vec<Converted> {
    let mut turn = assistant_message(chat, text);
    turn.push(message_end(
        chat,
        serde_json::json!({ "stopReason": "error", "errorMessage": error }),
    ));
    turn
}

/// The end of a turn as the child reports it: `agent_settled`, the event the
/// reducer folds by asking that chat's child for its context reading.
pub fn settled(chat: &ChatId) -> Converted {
    Converted::Input(settled_input(chat))
}

/// The same settled turn as the reducer input, for a test that folds it through
/// an entry point of its own.
pub fn settled_input(chat: &ChatId) -> Input {
    Input::ChatEventAt {
        chat: chat.clone(),
        event: RpcEvent::AgentSettled,
        now_ms: 10,
    }
}

/// One event of `chat`'s child, as the translating thread reports it.
fn event_at(chat: &ChatId, event: RpcEvent, now_ms: u64) -> Converted {
    Converted::Input(Input::ChatEventAt {
        chat: chat.clone(),
        event,
        now_ms,
    })
}

/// The close of the message the turn opened, `raw` carrying what the provider
/// said about it.
fn message_end(chat: &ChatId, raw: serde_json::Value) -> Converted {
    event_at(
        chat,
        RpcEvent::MessageEnd(RpcMessage {
            role: "assistant".to_owned(),
            raw,
        }),
        9,
    )
}
