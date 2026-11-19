//! Painting a stored chat's thread from its session FILE before its child
//! answers (`chat-open-latency.md` T1, T2, S1-S4).
//!
//! A stored chat's first frame used to be an empty thread for as long as its
//! child took to start - measured at three seconds warm (`chat-open-latency.md`,
//! L2). The file already holds those turns, so the shell reads it ONCE, off the
//! window's thread, and pushes what it found as a PROVISIONAL transcript; the
//! child's `get_messages` answer replaces it and always wins.
//!
//! The state here is SHELL-owned, next to the session mode's and the branch's:
//! `crates/app` carries no file, and the reducer's transcript is the only
//! authority on what the chat really holds. What this module keeps is a
//! fallback, and four rules say when it may be drawn:
//!
//! - the rows are the FOREGROUND chat's, and only while its thread holds no row
//!   of its own ([`provisional_transcript`]): once the reducer has a row, that
//!   row is what the thread draws, file or no file;
//! - once the child's `get_messages` answer has been folded the rows are neither
//!   drawn nor re-installed, which is why the reducer keeps that fact at all
//!   ([`Chats::history_loaded`](gentle_app::Chats::history_loaded)): an empty
//!   answer is an answer, and emptiness cannot stand for "not loaded yet";
//! - a read that lands for a chat that closed - or closed and REOPENED, which is
//!   a new incarnation - is dropped ([`SessionRead::started_in`], the epoch
//!   `bridge::session_mode` also uses, compared under the same lock the install
//!   takes and bumped under it by the close);
//! - a file that is missing, unreadable, being written, past the cap, or holds
//!   nothing the thread shows draws nothing and says nothing: no row and no
//!   notice, which is exactly what the chat drew before this module existed.
//!
//! One open asks for one read. A fresh chat has no file, so it asks for none.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Component, Path};
use std::sync::Arc;

use gentle_app::ChatId;
use gentle_core::chat::{messages_from_session_file, ChatMessage};
use gentle_core::i18n::Language;

use super::{lock, AppStateHolder};
use crate::contract::ViewState;
use crate::settings::FileRead;
use crate::view::MESSAGES_PUSHED;

/// What runs one finished read, off the window's thread: the push that carries
/// what it found. Installed once at startup, like the branch chip's
/// ([`AppStateHolder::arm_branch_push`]), because the read that answers it runs
/// on a worker and has no other way to reach the window.
pub type TranscriptPush = Arc<dyn Fn(SessionRows) + Send + Sync>;

/// One chat's session file, claimed by one open of that chat.
///
/// It names the file, the incarnation of the chat the open started, and the seam
/// the file is read through - nothing else, and no text: the file is read once,
/// on the thread [`read`](Self::read) runs on.
pub struct SessionRead {
    /// The chat this read is for.
    pub chat: ChatId,
    /// The incarnation the read started in ([`AppStateHolder::session_epoch`]).
    pub started_in: u64,
    path: String,
    files: FileRead,
    language: Language,
}

impl SessionRead {
    /// Reads the file once and answers the rows it paints.
    ///
    /// Every reason a read comes up with nothing answers the same empty list: a
    /// file that is not there, one this run cannot read, one being written (its
    /// last line is not JSON yet) and one past
    /// [`SESSION_FILE_MAX_BYTES`](gentle_core::chat::SESSION_FILE_MAX_BYTES).
    /// None of them is news for the user, so none of them is an error here.
    pub fn read(self) -> SessionRows {
        let text = (self.files)(Path::new(&self.path), self.language)
            .ok()
            .flatten();
        SessionRows {
            chat: self.chat,
            started_in: self.started_in,
            rows: text
                .map(|text| messages_from_session_file(&text))
                .unwrap_or_default(),
        }
    }
}

/// A finished read: the chat it was for, the incarnation it started in, and the
/// rows it found - empty when it found nothing to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionRows {
    pub chat: ChatId,
    pub started_in: u64,
    pub rows: Vec<ChatMessage>,
}

/// One chat's provisional rows, and whether this incarnation already asked for
/// them: one read per open, and the marker goes with the chat when it closes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct Provisional {
    /// The tail of the rows the file held, already cut to the window a push
    /// carries.
    rows: Vec<ChatMessage>,
    /// The one read of this chat's current open has been claimed.
    started: bool,
}

/// Every chat that was read, one entry per chat.
pub(super) type Transcripts = BTreeMap<String, Provisional>;

impl AppStateHolder {
    /// Installs what pushes a finished read (called once at startup, beside the
    /// branch chip's own sink).
    pub fn arm_transcript_push(&self, push: TranscriptPush) {
        *lock(&self.transcript_push) = Some(push);
    }

    /// Claims the ONE read of the foreground chat's session file, when there is
    /// one to claim.
    ///
    /// `None` - and no read at all - for: no foreground chat; a chat with no
    /// session file yet (a fresh chat, which has nothing to read); a chat whose
    /// thread already holds a row of its own, or whose child has answered
    /// `get_messages`, because in both cases the reducer is the truth and the
    /// file is not; a chat whose read of this open already started; a run with
    /// no session-file seam armed, or one whose store the chat's path is not
    /// inside.
    ///
    /// This never reads a byte: the caller runs [`SessionRead::read`] on a worker
    /// thread, which is why the intent path can call it and still answer.
    pub fn claim_transcript(&self) -> Option<SessionRead> {
        // The app state is the authority on all of this, and it is read before
        // any shell lock is taken, as the shell's other reads are.
        let (chat, path) = {
            let app = lock(&self.app);
            let chat = app.chats.foreground()?.clone();
            let path = chat.session_path()?.to_owned();
            let thread = app.chats.state(&chat)?;
            if !thread.items.is_empty() || app.chats.history_loaded(&chat) {
                return None;
            }
            (chat, path)
        };
        let armed = lock(&self.session_files).clone()?;
        // The id may be any string the web sent, so the read is confined to this
        // run's sessions store before the seam is reached - the same lexical
        // guard the Profile tab's read applies (`bridge::chat_profile`).
        if !session_file(&armed.store, Path::new(&path)) {
            return None;
        }
        let language = self.language();
        let started_in = {
            let mut held = lock(&self.transcripts);
            let state = held.entry(chat.as_str().to_owned()).or_default();
            if state.started {
                return None;
            }
            // The epoch is read UNDER the lock the install compares it under, so
            // no close can slip between this claim and the answer that close
            // must drop (R3-001 of `chat-transcript.md`).
            let epoch = self.session_epoch(&chat);
            state.started = true;
            epoch
        };
        Some(SessionRead {
            chat,
            started_in,
            path,
            files: armed.files,
            language,
        })
    }

    /// Starts the one read of the foreground chat's file, off this thread, when
    /// there is one to start. Called by the fold that opens a chat: a child takes
    /// seconds to answer, and this is what the thread paints meanwhile.
    pub(super) fn start_transcript_read(&self) {
        let Some(read) = self.claim_transcript() else {
            return;
        };
        let Some(push) = lock(&self.transcript_push).clone() else {
            return;
        };
        // No lock is held here and nothing waits: the file is read and pushed on
        // its own thread, and a push that finds the window gone pushes nothing.
        let _ = std::thread::Builder::new()
            .name("chat-transcript".to_owned())
            .spawn(move || push(read.read()));
    }

    /// Records what one read found and answers the slice to push, or `None` when
    /// it may not be installed at all.
    ///
    /// `None` - and no push - for a read that found nothing to draw (no file, an
    /// unreadable one, one being written, one past the cap: the chat keeps
    /// exactly what it drew), for a chat that is no longer open, for a chat whose
    /// child has answered `get_messages` in the meantime (the answer is the
    /// authority and a late read never re-installs itself), and for a read of an
    /// incarnation the chat has left - it closed, or closed and reopened.
    pub fn note_transcript(&self, read: &SessionRows) -> Option<ViewState> {
        if read.rows.is_empty() {
            return None;
        }
        // Read before any shell lock, as the session mode's own answer does.
        let (open, loaded) = {
            let app = lock(&self.app);
            (
                app.chats.state(&read.chat).is_some(),
                app.chats.history_loaded(&read.chat),
            )
        };
        if !open || loaded {
            return None;
        }
        {
            let mut held = lock(&self.transcripts);
            if read.started_in != self.session_epoch(&read.chat) {
                return None;
            }
            let rows = tail(&read.rows);
            held.entry(read.chat.as_str().to_owned()).or_default().rows = rows;
        }
        Some(self.view())
    }

    /// The provisional rows as they stand: the FOREGROUND chat's own, with the
    /// chat they were read for.
    ///
    /// `None` while nothing was read and for every chat that is not the
    /// foreground one. Whether they may actually be DRAWN is
    /// [`view_state`](crate::state_view::view_state)'s rule: it reads the app
    /// state it already holds and drops them the moment the chat has a row of its
    /// own or its child has answered.
    pub(super) fn provisional_transcript(&self) -> Option<(String, Vec<ChatMessage>)> {
        let chat = lock(&self.app).chats.foreground()?.as_str().to_owned();
        let held = lock(&self.transcripts);
        let state = held.get(&chat)?;
        Some((chat, state.rows.clone()))
    }

    /// Drops that chat's provisional rows with the chat they were read for, and
    /// bumps the incarnation the close ends. Read by the fold that takes a chat
    /// out of the open set (`Chats::close`).
    ///
    /// The bump happens under the rows' own lock, which is the lock every install
    /// compares it under: a read that started before this close can never be
    /// installed on the chat that comes back - a chat id that closes and reopens
    /// is a NEW incarnation, and a read of the old one belongs to a chat that is
    /// gone (R3-001). `bridge::session_mode`'s forget bumps the same counter for
    /// its own view at the same fold; the counter only ever moves forward, so the
    /// second bump costs nothing and both answers stay dropped.
    pub(super) fn forget_transcript(&self, chat: &ChatId) {
        let mut held = lock(&self.transcripts);
        held.remove(chat.as_str());
        *lock(&self.session_epochs)
            .entry(chat.as_str().to_owned())
            .or_insert(0) += 1;
    }
}

/// The tail of `rows` a push carries: the same window the child's own answer is
/// cut to (`MESSAGES_PUSHED`), so a long session's file costs a push no more
/// than its thread does.
fn tail(rows: &[ChatMessage]) -> Vec<ChatMessage> {
    let skip = rows.len().saturating_sub(MESSAGES_PUSHED);
    rows[skip..].to_vec()
}

/// Whether `path` may be a session file of `store`, by SPELLING alone: inside the
/// store, ending in `.jsonl` (the extension the session list itself matches
/// exactly), and with no `..` component anywhere in it.
///
/// The same guard the Profile tab's read applies (`bridge::chat_profile`), and
/// for the same reason: the id may be any string the web sent, the reader is a
/// seam, and the filesystem is never asked.
fn session_file(store: &Path, path: &Path) -> bool {
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return false;
    }
    if path.extension() != Some(OsStr::new("jsonl")) {
        return false;
    }
    path.starts_with(store)
}
