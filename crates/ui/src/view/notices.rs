//! Transient notices from extensions, and the recoverable report of a chat's
//! death: each is dismissible, and a notice that knows its fix carries that one
//! action (S3).

use gentle_app::{ChatId, Notice, NoticeAction, NoticeLevel};
use iced::widget::{button, column, container, row, text};
use iced::{Element, Length, Theme};

#[derive(Debug, Clone)]
pub enum Event {
    Dismiss(u64),
    /// The notice's action: start that chat's child again and reopen it.
    Restart(ChatId),
}

pub fn view(notices: &[Notice]) -> Element<'_, Event> {
    column(notices.iter().map(notice)).spacing(4).into()
}

fn notice(notice: &Notice) -> Element<'_, Event> {
    let style: fn(&Theme) -> container::Style = match notice.level {
        NoticeLevel::Info => container::rounded_box,
        NoticeLevel::Warning => container::warning,
        NoticeLevel::Error => container::danger,
    };
    let mut line = row![text(&notice.text).size(14).width(Length::Fill)];
    if let Some(action) = &notice.action {
        line = line.push(action_button(action));
    }
    line = line.push(
        button(text("×").size(14))
            .style(button::text)
            .on_press(Event::Dismiss(notice.id)),
    );
    container(line.align_y(iced::Alignment::Center).spacing(8))
        .padding([4, 12])
        .width(Length::Fill)
        .style(style)
        .into()
}

/// The one button a notice may offer, named after what it does rather than after
/// the notice: the user restarts that chat, not the message.
fn action_button(action: &NoticeAction) -> Element<'_, Event> {
    match action {
        NoticeAction::RestartChat(chat) => button(text("Restart").size(13))
            .style(button::secondary)
            .on_press(Event::Restart(chat.clone()))
            .into(),
    }
}
