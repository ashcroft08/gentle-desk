//! What goes into `update` and what comes out of it.

use gentle_core::rpc::{RpcCommand, RpcEvent, UiAnswer};

use crate::chats::ChatId;
use crate::session_prepare::PreparedSessions;
use crate::state::SessionItem;

/// What the user asked for; the UI only ever emits these.
#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    SendPrompt(String),
    Abort,
    NewChat,
    /// Copy the raw markdown of one message.
    CopyMessage(String),
    AnswerDialog {
        id: String,
        answer: UiAnswer,
    },
    DismissNotice(u64),
    /// Reload the session list.
    RefreshSessions,
    /// Switch to a saved session and show its history.
    OpenSession(String),
    /// The user typed in the sidebar's search box.
    SetSessionQuery(String),
    /// Open the thread of the helper with this id.
    SelectHelper(String),
    /// Back from a helper's thread to the helpers list.
    CloseHelper,
    /// Ask the child for the models it offers (the chip's dropdown).
    RequestModels,
    /// Make a model the session's model.
    SelectModel {
        provider: String,
        model_id: String,
    },
    /// Set the session's thinking level.
    SelectThinkingLevel {
        level: String,
    },
    /// Make a chat the foreground one. Its child is started if it has none, and
    /// no other chat is stopped.
    OpenChat(ChatId),
    /// A chat that did not exist before, with an id of its own.
    OpenNewChat,
    /// Start that chat's child again and reopen the chat, after its child died
    /// (S3). This is the action of the death notice; it works on any chat, not
    /// only the foreground one.
    RestartChat(ChatId),
    /// Close that chat: stop its child and take the chat off the open set, which
    /// is the set the Updates pane's chats-closed gate counts (0.1.14). The chat
    /// is NOT deleted: nothing on disk changes and the session list keeps its
    /// row, so the row opens the chat again (through `OpenChat`, which starts a
    /// fresh child through `--session`). A chat that is running a turn is
    /// refused with the delete flow's own rule and words: the child is mid-turn
    /// and stopping it is the user's (S4).
    CloseChat(ChatId),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Input {
    Intent(Intent),
    Rpc(RpcEvent),
    /// An RPC event with the wall-clock time (epoch ms) it arrived at. The shell
    /// supplies it so `update` stays pure; only helper bookkeeping uses it.
    RpcAt(RpcEvent, u64),
    /// The shell finished listing sessions. Same behaviour as
    /// `SessionsPrepared(prepare_sessions(items))`, folding included.
    SessionsLoaded(Vec<SessionItem>),
    /// The shell listed the sessions and built their search data off the UI
    /// thread, so `update` does not fold anything.
    SessionsPrepared(PreparedSessions),
    /// The first full pass is still running: the same prepared list as
    /// `SessionsPrepared`, read so far, plus how far the pass got. Ignored once
    /// the final list has arrived.
    SessionsProgress {
        prepared: PreparedSessions,
        done: usize,
        total: usize,
    },
    /// One decoded event of one chat's child, tagged with the chat it came from.
    /// A chat the app has not heard of yet is created empty. The same event of
    /// another chat is another input, and the two never touch each other.
    ChatEvent {
        chat: ChatId,
        event: RpcEvent,
    },
    /// [`Input::ChatEvent`] with the wall-clock time (epoch ms) the event
    /// arrived at, so a helper that stops being reported in a background chat is
    /// stamped with when it ended. Otherwise identical.
    ChatEventAt {
        chat: ChatId,
        event: RpcEvent,
        now_ms: u64,
    },
    /// A command for one chat's child did not reach it (it could not be started,
    /// or its stdin was gone). A notice for that chat, never fatal for the whole
    /// window: the other chats keep their children.
    ChatFailed {
        chat: ChatId,
        message: String,
    },
    /// A new chat's child reported the session file it opened: the chat is the
    /// stored chat from now on.
    ChatRekeyed {
        from: ChatId,
        to: ChatId,
    },
    /// The window's working cap (S7), passed at boot from the host's
    /// `Limits::max_working`. The app enforces it: a prompt over the cap is
    /// queued, not refused. `0` means no cap is enforced.
    SetWorkingCap(usize),
    /// One chat's child is gone. `expected` is true when the host stopped it on
    /// purpose (idle reap, cap victim, close, shutdown): the chat is just idle
    /// and reopens silently. An unexpected close of any chat, foreground or
    /// background, ends that chat and leaves one notice naming it (S3).
    ChatClosed {
        chat: ChatId,
        expected: bool,
    },
    /// One non-empty line the chat's child printed on stderr. The app keeps the
    /// last few per chat in a bounded ring, so a death notice (S3) can say why
    /// the child died instead of only saying it died. Never a notice by itself,
    /// and never unbounded: at most three lines, each cut at 200 characters.
    ChatStderr {
        chat: ChatId,
        line: String,
    },
    /// The chat's child is up, freshly started or already live. The UI confirms
    /// an already-live chat (`Existing`) or a discarded start the moment the
    /// host answers; for a fresh child (`Started`) the host's answer only says
    /// the process was created — milliseconds —, not that the chat is
    /// answerable (that takes ~seconds), so it sends this input instead only as
    /// a 20 s safety net for a child that started but never speaks. Otherwise
    /// [`ChatActivity::Opening`] ends with the chat's own first event. A start
    /// that fails goes through [`Input::ChatFailed`], which ends the phase the
    /// same way.
    ChatOpened {
        chat: ChatId,
    },
}

/// A side effect for the shell to run.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Send(RpcCommand),
    CopyToClipboard(String),
    /// Refresh the session index and report back with `Input::SessionsLoaded`.
    RefreshSessions,
}

/// Everything one [`Input`] asks the shell to do.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Effects {
    /// The effects of the single chat the one-child shell ran.
    pub commands: Vec<Command>,
    /// The effects on one chat's child, for a shell that runs one per chat.
    pub chat_commands: Vec<ChatCommand>,
}

/// A side effect on one chat's child.
///
/// Separate from [`Command`] because that one belongs to the single child the
/// one-child shell ran. A shell that owns one child per chat runs these through
/// [`update_chats`](crate::update_chats); the single child keeps taking
/// [`Command::Send`], so nothing of the old shell changes.
#[derive(Debug, Clone, PartialEq)]
pub enum ChatCommand {
    /// Start that chat's child, or focus it when it is already up.
    OpenProcess { chat: ChatId },
    /// Route one RPC command to that chat's child and no other.
    Send { chat: ChatId, rpc: RpcCommand },
    /// Stop that chat's child and forget it in the host (0.1.14). The chat's
    /// session file is untouched: this is the child going away, not the chat. A
    /// chat with no child (never opened, already gone) is simply closed, so the
    /// command is safe to run twice.
    CloseProcess { chat: ChatId },
}

impl ChatCommand {
    /// The chat this command is for.
    pub fn chat(&self) -> &ChatId {
        match self {
            Self::OpenProcess { chat } | Self::Send { chat, .. } | Self::CloseProcess { chat } => {
                chat
            }
        }
    }
}
