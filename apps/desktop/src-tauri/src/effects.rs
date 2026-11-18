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
use gentle_runtime::chat_runtime::{
    request_after_open, ChainWriteResult, ChatConfigRead, ChatProfileApply, ModeWriteResult,
    Outcome,
};

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
    /// The profile configuration that arrived, with the chat it answers for
    /// (profiles-editor T2b-2), in the order the run answered. It is NOT a
    /// reducer input: the view is the shell's own (`bridge::chat_profile`), so
    /// it travels beside the inputs for the caller to note on its own. Every
    /// read the run answered is carried (T2b-3): two reads must not collapse
    /// into the last one.
    pub chat_profile: Vec<(ChatId, ChatConfigRead)>,
    /// Every profile-mode save one run answered, with the chat it belongs to and
    /// the profile configuration the save re-read (profiles-editor T3-mode).
    /// Shell-owned like [`Self::chat_profile`]: whether the write failed is a
    /// state of the tab's own, not the reducer's.
    pub chat_mode: Vec<(ChatId, ModeWriteResult, ChatConfigRead)>,
    /// Every profile APPLY one run answered, with the chat it belongs to
    /// (profiles-editor T3b): the channel's own dialog, a typed refusal, or an
    /// attempt that ran together with the configuration it re-read. Shell-owned
    /// like the two above: the reducer knows nothing about a profile.
    pub chat_apply: Vec<(ChatId, ChatProfileApply)>,
    /// Every fallback-chain save one run answered, with the chat it belongs to and
    /// the profile configuration the save re-read (profiles-editor T5).
    /// Shell-owned like the two above: whether the chain write failed is a state
    /// of the tab's own, not the reducer's.
    pub chat_chain: Vec<(ChatId, ChainWriteResult, ChatConfigRead)>,
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
        Outcome::Opened { chat, how } => {
            // A fresh child: the host confirmed the process only, so the chat
            // stays opening. An already-live child is confirmed at once — but
            // either way the chip's questions go out (R3-001): a chat that was
            // away from the foreground cleared its chip and needs the state
            // and both lists again.
            for next in request_after_open(&chat, how) {
                run_into(next, executor, ran);
            }
            if how == Opened::Started {
                ran.confirm_after = Some(chat);
            } else {
                ran.inputs.push(Input::ChatOpened { chat });
            }
        }
        // A failed open ends the opening phase too: `ChatFailed` clears it.
        Outcome::OpenFailed { chat, message } | Outcome::SendFailed { chat, message } => {
            ran.inputs.push(Input::ChatFailed { chat, message });
        }
        Outcome::Sent => {}
        // The host's working mark is not the app's: the reducer already holds it
        // and folds nothing back, so the answer is only that the mark landed.
        Outcome::Working { .. } => {}
        // The user's Close chat stopped that child: folded as the EXPECTED close
        // the host reports, so no death notice is left for a chat the user put
        // away (0.1.14). The child's own `Closed` event folds the same input
        // again, which changes nothing.
        Outcome::Closed { chat } => ran.inputs.push(Input::ChatClosed {
            chat,
            expected: true,
        }),
        // The user's stop of ONE helper, and the child accepted it: that helper
        // reads `Cancelled` from now on (the `Ended` mark both windows draw),
        // while the chat keeps running (helper-stop S2).
        Outcome::Stopped { chat, helper_id } => {
            ran.inputs.push(Input::HelperStopped { chat, helper_id })
        }
        // No stop happened - no child, an inactive host, a timeout, a closed
        // child or a refusal - and today's behaviour is exactly that: the helper
        // keeps the mark it had and nothing is said (S7).
        Outcome::NotStopped { .. } => {}
        // A chat's profile configuration arrived (profiles-editor T2b-2): the
        // shell's own view takes it, not the reducer, so nothing is folded here
        // and the read travels beside the inputs. A read that reached no store
        // leaves the tab exactly as it was drawn (S6, L11-4).
        Outcome::ChatProfile { chat, read } => ran.chat_profile.push((chat, *read)),
        // One profile's orchestration mode was saved, or it was not and the
        // result says why (profiles-editor T3-mode): the shell's own state, kept
        // beside the read so the tab can say the mode could not be saved and
        // draw the truth the re-read brought.
        Outcome::ChatMode { chat, result, read } => ran.chat_mode.push((chat, result, *read)),
        // One saved profile was applied, or the answer is the child's own dialog
        // or a typed refusal (profiles-editor T3b): the shell's own state, kept
        // whole so the page can draw the dialog verbatim and the notice.
        Outcome::ChatApply { chat, attempt } => ran.chat_apply.push((chat, *attempt)),
        // One fallback chain was saved, or it was not and the result says why
        // (profiles-editor T5): the same shell-owned shape as the mode's, kept
        // apart so the chain editor's notice never lights the mode switch's.
        Outcome::ChatChain { chat, result, read } => ran.chat_chain.push((chat, result, *read)),
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
