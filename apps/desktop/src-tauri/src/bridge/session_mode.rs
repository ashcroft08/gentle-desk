//! One chat's session permission mode, as the shell holds it (composer-dock T3b,
//! L7, L8).
//!
//! The mode belongs to a SESSION, so the only authority on it is that chat's own
//! child: `get_permission_mode` reads it, `set_permission_mode { mode }` asks for
//! a change, and nothing of it is global (L7). The view is SHELL-owned, next to
//! the chat profile's: `crates/app` carries no permission mode, and the reducer
//! knows nothing about one.
//!
//! Four rules, and they are the whole of this module's business (L7):
//!
//! - the mode the label draws changes only on the CONFIRMED answer - the state
//!   the owner re-read after a write, or what a read reported - never
//!   optimistically, and never because a click asked for it;
//! - a failure shows its own code and leaves the mode exactly as it was;
//! - a read the child did not answer draws nothing at all, because with no mode
//!   there is no control to draw;
//! - `persist` is never sent, so the saved default stays untouched: the request
//!   carries the mode alone (`crates/gentle-core/src/rpc/permission.rs`).
//!
//! The mode travels as the channel's own string and is kept RAW
//! ([`SessionModeState::mode`]): a mode this build does not know is shown as it
//! arrived, never hidden and never mapped onto one the app does know.
//!
//! A chat has at most ONE session-mode command in flight
//! ([`AppStateHolder::begin_chat_command`], the same cap the profile commands
//! keep), and an answer that arrives for a chat that closed or was forgotten is
//! dropped rather than installed - the rule the chat profile's view already
//! follows.

use std::collections::BTreeMap;

use gentle_app::ChatId;
use gentle_core::rpc::permission::{PermissionOutcome, SessionMode};

use super::{lock, AppStateHolder};
use crate::contract::{SessionModeView, ViewState};

/// What one chat's session mode has come to (composer-dock T3b, L7).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct SessionModeState {
    /// The mode the chat's own child last reported, in the channel's own string.
    /// `None` until an exchange answers, and that is exactly when no control is
    /// drawn.
    pub(super) mode: Option<String>,
    /// The code of the last attempt that wrote nothing: the owner's own
    /// (`permission_denied`, `permission_unavailable`, `invalid_request`, ...) or
    /// `no_answer`/`unsupported` when the child did not answer at all.
    pub(super) refusal: Option<String>,
    /// How many WRITE attempts this chat has answered: a read is no attempt.
    pub(super) attempt: u32,
}

/// The session-mode state of every chat that answered, one entry per chat.
pub(super) type SessionModes = BTreeMap<String, SessionModeState>;

/// Folds one exchange's answer into `state` (composer-dock T3b, L7).
///
/// The mode is written by ONE case only: an answer, which is the state the owner
/// reported (a read) or re-read after the attempt (a write). Every other case
/// leaves the mode exactly as it was and says what happened, because a failure
/// shows its code and changes nothing.
fn fold(state: &mut SessionModeState, change: Option<SessionMode>, outcome: &PermissionOutcome) {
    match (change, outcome) {
        // The owner's own state: what the label draws, and no refusal.
        (_, PermissionOutcome::Answered(answered)) => {
            state.mode = Some(answered.mode.clone());
            state.refusal = None;
            if change.is_some() {
                state.attempt = state.attempt.saturating_add(1);
            }
        }
        // A refusal is the child's own answer about the ATTEMPT: its code is what
        // the page shows, and the mode it drew stays (L7).
        (Some(_), PermissionOutcome::Rejected(error)) => {
            state.refusal = Some(error.code.clone());
            state.attempt = state.attempt.saturating_add(1);
        }
        // A read the child refused still has no mode to draw: it says the code
        // and draws no control, now or later.
        (None, PermissionOutcome::Rejected(error)) => state.refusal = Some(error.code.clone()),
        // A write the child does not offer wrote nothing, and the page says so.
        (Some(_), PermissionOutcome::Unsupported) => {
            state.refusal = Some(UNSUPPORTED.to_owned());
        }
        // A write nobody answered - a timeout, an inactive host, a closed child,
        // an answer this contract does not read, a request that never left - is
        // still a write that did NOT happen.
        (Some(_), _) => state.refusal = Some(NO_ANSWER.to_owned()),
        // A read nobody answered draws nothing at all: with no mode there is no
        // control, and nothing is said about a mode this app never read.
        (None, _) => {}
    }
}

impl AppStateHolder {
    /// Records what one read, or one asked-for change, of `chat`'s session mode
    /// came to, and answers the slice to push.
    ///
    /// `None` - and no push at all - when the chat is no longer open: an answer
    /// that lands after its chat closed may not re-install what the close
    /// dropped. Either way the chat's one in-flight session-mode slot comes free
    /// here, because this IS that command's answer.
    pub fn note_session_mode(
        &self,
        chat: &ChatId,
        change: Option<SessionMode>,
        outcome: &PermissionOutcome,
    ) -> Option<ViewState> {
        self.note_session_mode_in(chat, change, outcome, None)
    }

    /// [`note_session_mode`](Self::note_session_mode) for an answer that knows
    /// which incarnation of its chat the command started in ([`session_epoch`]):
    /// an answer from an older one belongs to a chat that closed, even when the
    /// same chat id is open again, and is dropped. The epoch is compared under
    /// the same lock the fold takes, and the close bumps it under that lock too,
    /// so no close can slip between the check and the fold (R3-001).
    pub fn note_session_mode_in(
        &self,
        chat: &ChatId,
        change: Option<SessionMode>,
        outcome: &PermissionOutcome,
        started_in: Option<u64>,
    ) -> Option<ViewState> {
        // The app state is the authority on the open set, read before any shell
        // lock is taken, as the chat profile's own note does.
        if !self.is_open(chat) {
            self.forget_session_mode(chat);
            lock(&self.profile_work).remove(chat.as_str());
            return None;
        }
        {
            let mut modes = lock(&self.session_modes);
            if started_in.is_some_and(|epoch| epoch != self.session_epoch(chat)) {
                return None;
            }
            let state = modes.entry(chat.as_str().to_owned()).or_default();
            fold(state, change, outcome);
        }
        self.free_session_mode_slot(chat);
        Some(self.view())
    }

    /// Records that a session-mode command could not even START (the worker
    /// thread it runs on could not be created), and answers the slice to push.
    ///
    /// A WRITE that never ran wrote nothing, so it answers exactly what a write
    /// with no answer leaves: `no_answer`, with the mode kept as it is. A READ
    /// that never ran read nothing and says nothing: the page keeps what it drew,
    /// and only the chat's in-flight slot comes free.
    pub fn note_session_mode_unstarted(
        &self,
        chat: &ChatId,
        change: Option<SessionMode>,
    ) -> Option<ViewState> {
        if !self.is_open(chat) {
            self.forget_session_mode(chat);
            lock(&self.profile_work).remove(chat.as_str());
            return None;
        }
        self.free_session_mode_slot(chat);
        if change.is_none() {
            return None;
        }
        {
            let mut modes = lock(&self.session_modes);
            let state = modes.entry(chat.as_str().to_owned()).or_default();
            state.refusal = Some(NO_ANSWER.to_owned());
        }
        Some(self.view())
    }

    /// The session-mode slice as it stands: the FOREGROUND chat's own state, with
    /// the chat it belongs to.
    ///
    /// `None` while nothing answered, and for a chat whose mode is still unknown:
    /// an unknown mode is no control, never a placeholder. The projection onto the
    /// foreground chat is checked again by
    /// [`view_state`](crate::state_view::view_state), which only draws it while
    /// the chat it names is the foreground one.
    pub(super) fn session_mode(&self) -> Option<(String, SessionModeView)> {
        let chat = lock(&self.app).chats.foreground()?.as_str().to_owned();
        let modes = lock(&self.session_modes);
        let state = modes.get(&chat)?;
        let view = SessionModeView {
            mode: state.mode.clone()?,
            refusal: state.refusal.clone(),
            attempt: state.attempt,
        };
        Some((chat, view))
    }

    /// Drops that chat's session mode with the chat it was read for, and frees
    /// the slot any unanswered command held.
    pub(super) fn forget_session_mode(&self, chat: &ChatId) {
        // Under the modes lock, so an answer's epoch check and its fold cannot
        // straddle a close.
        let mut modes = lock(&self.session_modes);
        modes.remove(chat.as_str());
        *lock(&self.session_epochs).entry(chat.as_str().to_owned()).or_default() += 1;
    }

    /// Which incarnation of the chat a session-mode command starts in: it changes
    /// every time the chat's mode is forgotten, so an answer that carries an older
    /// one belongs to a chat that closed, even if the same chat is open again.
    pub fn session_epoch(&self, chat: &ChatId) -> u64 {
        lock(&self.session_epochs).get(chat.as_str()).copied().unwrap_or(0)
    }

    /// Gives back the chat's one in-flight session-mode slot: this command has
    /// answered, whatever it answered, so the chat may be asked again.
    fn free_session_mode_slot(&self, chat: &ChatId) {
        let mut work = lock(&self.profile_work);
        if let Some(state) = work.get_mut(chat.as_str()) {
            state.session_mode = false;
        }
    }
}

/// The code a write that never reached a child leaves: nothing was written, and
/// the page says so in one short line it words itself.
const NO_ANSWER: &str = "no_answer";

/// The code a write the child does not advertise leaves: the method is not part
/// of that child's contract, so nothing was written there either.
const UNSUPPORTED: &str = "unsupported";
