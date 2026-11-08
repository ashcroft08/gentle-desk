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
}
