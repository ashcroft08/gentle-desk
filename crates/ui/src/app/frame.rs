//! The frame: which widgets make the page, in which order.
//!
//! Pure drawing over the state and the two bits of window geometry the app
//! keeps. Nothing here decides anything: a click leaves as a component event and
//! comes back through `App::update`.

use iced::widget::{column, row, text};
use iced::Element;

use super::{visible, App};
use crate::message::Message;
use crate::view::{
    banner, composer, dialog, header, helper_thread, helpers, notices, sidebar, thread,
};
use gentle_runtime::clock;

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        let chat = visible::chat(&self.app);
        // A chat whose child is being started has no history on screen yet and
        // cannot be talked to; the line says so without a spinner or a banner.
        let opening = visible::opening(&self.app).then(|| text("Opening chat…"));
        let mut page = column![
            header::view().map(Message::Header),
            notices::view(&self.app.notices).map(Message::Notices),
        ];
        if let Some(line) = opening {
            page = page.push(line);
        }
        page = page.push(thread::view(&chat.messages, &self.cache).map(Message::Thread));
        // Only a failure that ends everything (no launcher) is a banner now: a
        // child that died is a per-chat notice with its Restart (S3).
        if let Some(error) = &self.fatal {
            page = page.push(banner::view(error));
        }
        if let Some(pending) = chat.pending_dialogs.first() {
            page = page.push(dialog::view(pending, &self.dialog_text).map(Message::Dialog));
        }
        let can_send = visible::can_send(&self.app, &self.input, self.fatal.is_some());
        // A queued chat (S7) shows the same Stop as a working one: its button
        // cancels the queued prompt instead of sending it.
        let page = page.push(
            composer::view(&self.input, visible::chat_busy(&self.app), can_send)
                .map(Message::Composer),
        );
        let sessions = sidebar::view(sidebar::Props {
            visible: self.app.visible_sessions(),
            total: self.app.sessions.len(),
            query: &self.app.session_query,
            list_status: self.app.list_status,
            current: visible::current_session(&self.app),
            now_ms: clock::now_ms(),
            offset: self.sidebar_offset,
            // The plain activity snapshot the rows' badges draw from (S4): one
            // entry per chat that is not idle, not the whole app state.
            activities: self.app.activity_by_path(),
        });
        let mut layout = row![sessions.map(Message::Sidebar), page];
        // The open helper's thread replaces the list; a task that vanished falls back to it.
        let open = self
            .app
            .selected_helper
            .as_ref()
            .and_then(|id| chat.helpers.tasks.iter().find(|task| &task.id == id));
        if let Some(task) = open {
            layout = layout.push(helper_thread::view(task).map(Message::HelperThread));
        } else if helpers::is_visible(&chat.helpers) {
            layout = layout.push(helpers::view(&chat.helpers).map(Message::Helpers));
        }
        layout.into()
    }
}
