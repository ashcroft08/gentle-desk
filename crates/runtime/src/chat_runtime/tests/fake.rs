//! A spawner that starts no process: it records which chats were started, what
//! was written to each and which children were shut down, so the runtime is
//! tested without a child.
//!
//! A SCRIPTED spawner also answers the `/gentle:desktop` channel: the child reads
//! the prompt it was written and pushes the answer frame pi would push, so the
//! chat's profile read (profiles-editor T2b) is proven on the wire - the request
//! ids, the order and the bodies - with no process and no clock behind it.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gentle_backend::chat_host::{ChatKey, Child, Spawner, Writer};
use gentle_backend::process::{ProcessEvent, Shutdown};
use gentle_core::rpc::{decode_line, RpcCommand, COMMAND, SCHEMA};
use serde_json::{json, Value};

/// What was written to one child, shared between the child and the test.
type Written = Arc<Mutex<Vec<RpcCommand>>>;
/// The children shut down so far, shared between the children and the test.
type Stopped = Arc<Mutex<Vec<ChatKey>>>;

/// What one scripted child answers to one desktop request.
#[derive(Debug, Clone)]
pub enum Reply {
    /// An `ok: true` answer carrying this `data`.
    Data(Value),
    /// An `ok: false` answer with this error code (`invalid_data`, `not_found`).
    Refused { code: &'static str },
}

/// What a scripted child answers, by method and the request's own params.
///
/// The test names the answer it is about; the child's envelope, its `requestId`
/// included, is built here, so no test writes a frame by hand.
pub type Script = Arc<dyn Fn(&str, &Value) -> Reply + Send + Sync>;

#[derive(Default)]
pub struct FakeSpawner {
    started: Mutex<Vec<(ChatKey, Written)>>,
    /// The folder each start was given, in the order of `started`.
    folders: Mutex<Vec<Option<PathBuf>>>,
    stopped: Stopped,
    failure: Mutex<Option<String>>,
    /// Whether its children answer as interactive RPC hosts, the marker the
    /// desktop channel is active behind.
    interactive: bool,
    /// What its children answer the desktop channel. `None` is a child that
    /// ignores `/gentle:desktop` entirely.
    script: Option<Script>,
}

impl FakeSpawner {
    /// The chats started so far, in order.
    pub fn started(&self) -> Vec<ChatKey> {
        let started = self.started.lock().unwrap();
        started.iter().map(|(key, _)| key.clone()).collect()
    }

    /// The folder every start was given, in the order of [`FakeSpawner::started`].
    pub fn folders(&self) -> Vec<Option<PathBuf>> {
        self.folders.lock().unwrap().clone()
    }

    /// The chats whose child was shut down, in order.
    pub fn stopped(&self) -> Vec<ChatKey> {
        self.stopped.lock().unwrap().clone()
    }

    /// Makes the next start fail with `message`.
    pub fn fail_next(&self, message: &str) {
        *self.failure.lock().unwrap() = Some(message.to_owned());
    }

    /// The same spawner, whose children answer the desktop channel with
    /// `script`. A child that answers the channel runs with the interactive-host
    /// marker set (what activates `/gentle:desktop`), so this reports it.
    pub fn answering(mut self, script: Script) -> Self {
        self.interactive = true;
        self.script = Some(script);
        self
    }

    /// What was written to the child of `key`.
    pub fn written_to(&self, key: &ChatKey) -> Vec<RpcCommand> {
        let started = self.started.lock().unwrap();
        started
            .iter()
            .find(|(known, _)| known == key)
            .map(|(_, commands)| commands.lock().unwrap().clone())
            .unwrap_or_default()
    }
}

impl Spawner for FakeSpawner {
    fn spawn(&self, key: &ChatKey, folder: Option<&Path>) -> io::Result<Box<dyn Child>> {
        if let Some(message) = self.failure.lock().unwrap().take() {
            return Err(io::Error::other(message));
        }
        self.folders
            .lock()
            .unwrap()
            .push(folder.map(Path::to_path_buf));
        let commands = Arc::new(Mutex::new(Vec::new()));
        self.started
            .lock()
            .unwrap()
            .push((key.clone(), Arc::clone(&commands)));
        // The child keeps the writing half of its own channel, so its event
        // stream stays open and the host reads it as alive until it is shut down.
        let (events, incoming) = mpsc::channel();
        Ok(Box::new(FakeChild {
            key: key.clone(),
            commands,
            answers: events.clone(),
            script: self.script.clone(),
            stopped: Arc::clone(&self.stopped),
            events: Some(incoming),
            _alive: events,
        }))
    }

    fn interactive(&self) -> bool {
        self.interactive
    }
}

struct FakeChild {
    key: ChatKey,
    commands: Written,
    /// Where the child pushes what it says, as a live child would.
    answers: Sender<ProcessEvent>,
    /// What it answers the desktop channel, if it answers at all.
    script: Option<Script>,
    stopped: Stopped,
    /// Handed to the host once, on the first `take_events`.
    events: Option<Receiver<ProcessEvent>>,
    /// Keeps the child's own event stream open while it lives.
    _alive: Sender<ProcessEvent>,
}

struct FakeWriter {
    commands: Written,
    answers: Sender<ProcessEvent>,
    script: Option<Script>,
}

impl Writer for FakeWriter {
    fn send(&self, command: &RpcCommand) -> io::Result<()> {
        // Recorded before the answer goes out, so a test that reads the child's
        // log after an exchange always sees the request that caused it.
        self.commands.lock().unwrap().push(command.clone());
        if let Some(answer) = self
            .script
            .as_ref()
            .and_then(|script| answer_frame(script, command))
        {
            let _ = self.answers.send(answer);
        }
        Ok(())
    }
}

/// The event a scripted child pushes for `command`: the desktop answer the script
/// gives it in the frame pi pushes, or nothing for a request it ignores or a
/// command that is not a desktop prompt.
fn answer_frame(script: &Script, command: &RpcCommand) -> Option<ProcessEvent> {
    let message = command.prompt_text()?;
    let envelope = message.strip_prefix(COMMAND)?.trim_start();
    let request: Value = serde_json::from_str(envelope).ok()?;
    let request_id = request.get("requestId")?.as_str()?.to_owned();
    let method = request.get("method")?.as_str()?.to_owned();
    let params = request.get("params").cloned().unwrap_or(Value::Null);
    let payload = match script(&method, &params) {
        // The answer is correlated by the request's OWN id, as gentle-shell's is.
        Reply::Data(data) => json!({
            "schema": SCHEMA,
            "kind": "response",
            "requestId": request_id,
            "ok": true,
            "data": data,
        }),
        Reply::Refused { code } => json!({
            "schema": SCHEMA,
            "kind": "response",
            "requestId": request_id,
            "ok": false,
            "error": { "code": code, "message": format!("the scripted child refused: {code}") },
        }),
    };
    let frame = json!({
        "type": "extension_ui_request",
        "id": "ui-1",
        "method": "setWidget",
        "widgetKey": "gentle-desktop-rpc",
        "widgetLines": [payload.to_string()],
    });
    Some(ProcessEvent::Output(decode_line(&frame.to_string())))
}

impl Child for FakeChild {
    fn writer(&self) -> Option<Arc<dyn Writer>> {
        Some(Arc::new(FakeWriter {
            commands: Arc::clone(&self.commands),
            answers: self.answers.clone(),
            script: self.script.clone(),
        }))
    }

    fn take_events(&mut self) -> Receiver<ProcessEvent> {
        self.events.take().unwrap_or_else(|| mpsc::channel().1)
    }

    fn shutdown(self: Box<Self>, _grace: Duration) -> io::Result<Shutdown> {
        self.stopped.lock().unwrap().push(self.key.clone());
        Ok(Shutdown::Killed)
    }
}
