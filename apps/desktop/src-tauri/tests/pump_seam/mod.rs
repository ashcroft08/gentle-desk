//! The recording seam of the child-event pump tests, shared by them.
//!
//! A `Pump` is one pump thread fed by hand with translated values, recording
//! every slice it pushes: no child, no window, no launcher ever runs here.

use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use gentle_app::{ChatId, Input, Intent};
use gentle_core::rpc::{AssistantMessageEvent, RpcEvent, RpcMessage};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::ViewState;
use gentle_desk_lib::events::{spawn_pump, Push};
use gentle_runtime::chat_runtime::Converted;

/// How long a pump thread is given to end after its source closes.
const STOP_LIMIT: Duration = Duration::from_secs(5);

const SESSION_PATH: &str = "/sessions/a.jsonl";

/// A pump fed by hand, with the slices it pushed recorded in order.
pub struct Pump {
    source: Sender<Converted>,
    pushed: Arc<Mutex<Vec<ViewState>>>,
    thread: std::thread::JoinHandle<()>,
}

impl Pump {
    pub fn start(holder: &Arc<AppStateHolder>) -> Self {
        let (source, receiver) = channel();
        let pushed = Arc::new(Mutex::new(Vec::new()));
        let recorded = Arc::clone(&pushed);
        let push: Push = Box::new(move |view| {
            recorded.lock().expect("pushed mutex").push(view);
        });
        let thread =
            spawn_pump(receiver, Arc::clone(holder), push).expect("the pump thread starts");
        Self {
            source,
            pushed,
            thread,
        }
    }

    pub fn send(&self, converted: Converted) {
        self.source.send(converted).expect("the pump is listening");
    }

    /// Closes the source, which must end the thread, and answers what it pushed.
    pub fn finish(self) -> Vec<ViewState> {
        let Pump {
            source,
            pushed,
            thread,
        } = self;
        drop(source);
        join_within(thread);
        Arc::try_unwrap(pushed)
            .expect("the pump thread ended, so it holds no record")
            .into_inner()
            .expect("pushed mutex")
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
    let event_at = |event, now_ms| {
        Converted::Input(Input::ChatEventAt {
            chat: chat.clone(),
            event,
            now_ms,
        })
    };
    vec![
        event_at(RpcEvent::MessageStart(message), 7),
        event_at(
            RpcEvent::MessageUpdate(AssistantMessageEvent::TextDelta {
                content_index: 0,
                delta: text.to_owned(),
            }),
            8,
        ),
    ]
}
