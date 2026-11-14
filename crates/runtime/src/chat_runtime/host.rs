//! The host calls the shell makes outside a command: the working mark, the idle
//! reap and the shutdown of every child.

use gentle_app::ChatId;
use gentle_backend::chat_host::ChatKey;

use super::ChatRuntime;

impl ChatRuntime {
    /// Tells the host whether a chat is running a model call, as the app derives
    /// it from `agent_start` / `agent_end`. A working chat is never reaped and
    /// never used to make room for another one. `false` for a chat the host does
    /// not know.
    pub fn set_working(&self, chat: &ChatId, working: bool) -> bool {
        let key = self.keys().key_of(chat);
        key.is_some_and(|key| self.host.set_working(&key, working))
    }

    /// How many chats the host lets run model calls at once; the app enforces
    /// it with its own queue (S7).
    pub fn max_working(&self) -> usize {
        self.host.limits().max_working
    }

    /// Stops the idle background children and answers which chats went. The
    /// shell calls this on its timer.
    pub fn reap_idle(&self) -> Vec<ChatKey> {
        let reaped = self.host.reap_idle();
        for key in &reaped {
            eprintln!("reaped the idle child of {key}");
        }
        reaped
    }

    /// Stops every child. The shell calls this before the window exits, so no
    /// process outlives it (S5).
    pub fn shutdown(&self) {
        self.host.shutdown();
    }

    /// Stops one chat's child, if it has one, and forgets it in the host: the
    /// chat's next open starts a fresh child through `--session`, exactly as a
    /// reopen after a death does (0.1.14).
    ///
    /// The chat's session file is untouched - this is the child going away, not
    /// the chat - and no other chat is read or written. A chat the host does not
    /// know, and one whose child already went, answer `false`: closing is safe
    /// to ask for twice. Blocking for as long as the child takes its grace
    /// period (2 s), so the caller runs it off the window's thread.
    pub fn close(&self, chat: &ChatId) -> bool {
        let key = self.keys().key_of(chat);
        key.is_some_and(|key| self.host.close(&key))
    }
}
