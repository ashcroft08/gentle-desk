//! Running the per-chat commands the reducer answers, off the window's thread.
//!
//! The ROLE is the iced shell's (`crates/ui/src/app/chat_command.rs`), never its
//! code: a command is run through [`ChatExecutor`], and what came of it is
//! folded back as the reducer inputs it implies ([`Ran::inputs`]), exactly the
//! ones the iced shell folds after its task answers - `ChatOpened` for a chat
//! whose child is live, `ChatFailed` for one whose command did not reach it,
//! and, for a fresh child, the follow-up request `request_after_open` asks for
//! plus the 20 s net of [`opening_net`].
//!
//! The executor is the seam: [`runtime::RuntimeExecutor`] runs the real
//! [`gentle_runtime::chat_runtime::ChatRuntime`], and a test substitutes a
//! recording fake, so nothing here starts a child.
//!
//! [`ChatRuntime::execute`](gentle_runtime::chat_runtime::ChatRuntime::execute)
//! blocks for as long as a child takes to start, so the caller runs one command
//! per worker thread. This module itself spawns no thread but the net's.

use std::time::Duration;

use gentle_app::{ChatCommand, ChatId, Input};
use gentle_backend::chat_host::Opened;
use gentle_runtime::chat_runtime::{request_after_open, Outcome};

mod runtime;

pub use runtime::RuntimeExecutor;

/// What runs one chat command. Blocking by design: an open waits for the child
/// to come up, so the caller runs it on a worker thread, never inside an async
/// command on an executor thread of the window.
pub trait ChatExecutor: Send + Sync {
    fn execute(&self, command: ChatCommand) -> Outcome;
}

/// What running one command came to, as the inputs to fold, in order. Nothing
/// here touches state or the window: the caller folds and pushes the slice.
#[derive(Debug, Default)]
pub struct Ran {
    /// The reducer inputs the outcome implies, to fold one after another.
    pub inputs: Vec<Input>,
    /// A fresh child's start only proved its process was created, which takes
    /// milliseconds while the child takes seconds to speak. Its opening phase
    /// therefore ends with the chat's own first event, and the caller arms
    /// [`opening_net`] for this chat as the net for one that never speaks.
    pub confirm_after: Option<ChatId>,
}

/// Runs one command and everything its outcome asks for next, collecting the
/// inputs the outcome implies. The follow-up of a fresh child runs on this same
/// thread, as the iced shell asked for it from the outcome's own task.
pub fn run_one(command: ChatCommand, executor: &dyn ChatExecutor) -> Ran {
    let mut ran = Ran::default();
    run_into(command, executor, &mut ran);
    ran
}

fn run_into(command: ChatCommand, executor: &dyn ChatExecutor, ran: &mut Ran) {
    match executor.execute(command) {
        Outcome::Opened { chat, how } => match how {
            // A fresh child: the host confirmed the process only. The chat is
            // asked for what the window has to show, and stays opening.
            Opened::Started => {
                for next in request_after_open(&chat, how) {
                    run_into(next, executor, ran);
                }
                ran.confirm_after = Some(chat);
            }
            // Already live, or its start was discarded while starting: nothing
            // waits for that child any more, so it is confirmed at once.
            _ => ran.inputs.push(Input::ChatOpened { chat }),
        },
        // A failed open ends the opening phase too: `ChatFailed` clears it.
        Outcome::OpenFailed { chat, message } | Outcome::SendFailed { chat, message } => {
            ran.inputs.push(Input::ChatFailed { chat, message });
        }
        Outcome::Sent => {}
    }
}

/// The safety net of a fresh start: 20 s after the host confirmed the child was
/// created, the chat is confirmed opened whatever it did. A child that started
/// but never speaks would otherwise leave its chat opening forever; a child
/// that spoke already cleared the mark, and for it this confirmation changes
/// nothing. The wait runs on its own thread, so it never holds one of the
/// window's.
pub fn opening_net(chat: ChatId, after: impl FnOnce(Input) + Send + 'static) {
    const SILENCE_TOLERATED: Duration = Duration::from_secs(20);
    let spawned = std::thread::Builder::new()
        .name("chat-opening-wait".to_owned())
        .spawn(move || {
            std::thread::sleep(SILENCE_TOLERATED);
            after(Input::ChatOpened { chat });
        });
    if let Err(error) = spawned {
        // The net is a backup: the chat's own first event ends the phase, so a
        // thread that cannot start is a log line, never a stuck chat.
        eprintln!("chat opening net unavailable: {error}");
    }
}
