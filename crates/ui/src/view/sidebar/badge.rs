//! What one sidebar row draws for its chat's state (S4).
//!
//! This is the rule that turns a [`ChatActivity`] into plain drawing data — a
//! short mark, the colour family it draws in and whether the title is bold —
//! so the row itself only renders what is decided here. Idle draws nothing:
//! most rows in a big home are idle, and a mark would carry no information.

use gentle_app::ChatActivity;

/// Which family of the theme's colours a badge and its title draw in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColourRole {
    /// The accent the open chat's row is highlighted with: something is
    /// running there.
    Accent,
    /// A finished reply is waiting to be read.
    Success,
    /// A prompt is queued for its turn, but nothing is failed (S7).
    Muted,
    /// An extension dialog wants an answer.
    Warning,
    /// The chat's child is gone until it is restarted.
    Error,
}

/// The one short thing a row draws beside its title for its chat's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Badge {
    /// The mark itself: at most two characters wide, so it costs the title
    /// [`Badge::TITLE_ROOM`] at most instead of rewriting the row.
    pub mark: &'static str,
    /// The colour family the mark draws in.
    pub role: ColourRole,
    /// Whether the title is drawn bold, so a finished reply is seen without
    /// reading a mark; only the unread case asks for it.
    pub bold_title: bool,
}

impl Badge {
    /// How many characters of the title's line a badge costs, no matter which
    /// one: the title truncates this much sooner and the row keeps its height
    /// and its single line.
    pub const TITLE_ROOM: usize = 2;

    /// The badge for one chat's state; `None` draws nothing (Idle).
    pub fn on(activity: ChatActivity) -> Option<Self> {
        Some(match activity {
            ChatActivity::Idle => return None,
            ChatActivity::Working => Self {
                mark: "●",
                role: ColourRole::Accent,
                bold_title: false,
            },
            ChatActivity::FinishedUnread => Self {
                mark: "●",
                role: ColourRole::Success,
                bold_title: true,
            },
            ChatActivity::Queued => Self {
                // A clock-like ellipsis: its turn is on its way, so the mark
                // stays quiet, in the secondary colour.
                mark: "…",
                role: ColourRole::Muted,
                bold_title: false,
            },
            ChatActivity::NeedsAttention => Self {
                mark: "!",
                role: ColourRole::Warning,
                bold_title: false,
            },
            ChatActivity::Ended => Self {
                mark: "×",
                role: ColourRole::Error,
                bold_title: false,
            },
        })
    }
}

#[cfg(test)]
mod tests;
