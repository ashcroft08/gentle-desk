//! Transient notices from extensions, each dismissible.

use gentle_app::{Notice, NoticeLevel};
use iced::widget::{button, column, container, row, text};
use iced::{Element, Length, Theme};

#[derive(Debug, Clone)]
pub enum Event {
    Dismiss(u64),
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
    container(
        row![
            text(&notice.text).size(14).width(Length::Fill),
            button(text("×").size(14))
                .style(button::text)
                .on_press(Event::Dismiss(notice.id)),
        ]
        .align_y(iced::Alignment::Center),
    )
    .padding([4, 12])
    .width(Length::Fill)
    .style(style)
    .into()
}
