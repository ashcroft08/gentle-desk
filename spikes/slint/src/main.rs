//! Slint spike: message thread + markdown + streaming over the real core.

use std::rc::Rc;
use std::time::Duration;

use gentle_core::chat::{ChatRole, ChatState};
use slint::{ComponentHandle, ModelRc, StyledText, VecModel};

slint::include_modules!();

fn to_msgs(state: &ChatState) -> Vec<Msg> {
    state
        .messages
        .iter()
        .map(|message| {
            let user = message.role == ChatRole::User;
            Msg {
                user,
                plain: message.text.clone().into(),
                // Slint has no incremental markdown: the whole text is re-parsed per update.
                styled: if user {
                    StyledText::default()
                } else {
                    StyledText::from_markdown(&message.text)
                        .unwrap_or_else(|_| StyledText::from_plain_text(&message.text))
                },
            }
        })
        .collect()
}

fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    let weak = app.as_weak();
    spike_common::spawn_feed(
        move |state| {
            let weak = weak.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(app) = weak.upgrade() {
                    app.set_messages(ModelRc::from(Rc::new(VecModel::from(to_msgs(&state)))));
                }
            });
        },
        3,
        Duration::from_millis(25),
    );
    app.run()
}
