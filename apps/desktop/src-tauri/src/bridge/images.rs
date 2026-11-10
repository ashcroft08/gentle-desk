//! The images of the foreground chat, as the shell's own slice of the state.
//!
//! The store is [`crate::images`]; it lives on the holder because the slice
//! carries it twice: the composer draws what the foreground chat holds, and the
//! thread draws the images a sent message carried. Neither is the reducer's
//! (S2: `crates/app` is not changed by this slice), so the shell keeps them
//! here, keyed by the chat id the view already names, and the executor takes
//! them when it runs that chat's send.
//!
//! A drop names PATHS and nothing else, and the paths come from Tauri's own drop
//! event: a browser `File` has no path to give. The bytes are read here, in Rust
//! (S4), once, and they never travel back up.

use gentle_app::ChatId;

use super::{lock, AppStateHolder};
use crate::contract::ViewState;
use crate::images::Attachment;

impl AppStateHolder {
    /// Attaches the files the drop named to the FOREGROUND chat, and answers the
    /// slice to push: the thumbnails it now holds, or the line that says why a
    /// file was refused. No foreground chat takes nothing.
    pub fn attach_images(&self, paths: &[String]) -> ViewState {
        if let Some(chat) = self.foreground() {
            self.images.attach(chat.as_str(), paths);
        }
        self.view()
    }

    /// Takes one attached image away before its send, and answers the slice to
    /// push. An id no chat holds changes nothing.
    pub fn remove_image(&self, id: &str) -> ViewState {
        self.images.remove(id);
        self.view()
    }

    /// The images `chat` holds, for the send that is being run for it now: the
    /// take is what makes them that chat's and that send's only (S5).
    pub fn take_images(&self, chat: &ChatId) -> Vec<Attachment> {
        self.images.take(chat.as_str())
    }

    /// Remembers that these images went out with `chat`'s last message, so the
    /// thread draws the thumbnails the composer showed.
    ///
    /// The reducer appends the user message BEFORE it answers the send, and a
    /// chat runs one turn at a time, so the last message of the chat at the
    /// moment its send runs is the one these images belong to.
    pub fn note_images_sent(&self, chat: &ChatId, attachments: &[Attachment]) {
        let message = {
            let app = lock(&self.app);
            app.chats
                .state(chat)
                .and_then(|thread| thread.messages.last())
                .map(|message| message.id.clone())
        };
        if let Some(message) = message {
            self.images.bind(chat.as_str(), &message, attachments);
        }
    }

    /// The chat the composer belongs to, taken off the lock before any image is
    /// read.
    fn foreground(&self) -> Option<ChatId> {
        lock(&self.app).chats.foreground().cloned()
    }
}
