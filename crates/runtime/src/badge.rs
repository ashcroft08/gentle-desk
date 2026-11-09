//! What one chat's activity turns into for a list row (S6): the rule that
//! picks a semantic mark, a colour role and whether the title is bold, as
//! plain data for any window drawn on top of this crate.
//!
//! A mark is a meaning here, never a glyph: how it is drawn stays with the
//! window (today `crates/ui/src/view/sidebar/badge.rs`, tomorrow the Tauri
//! contract), so the two working and unread-reply states can end up with
//! different shapes in the new design without touching this rule.

use gentle_app::ChatActivity;

/// The short state name a row draws beside its chat's title.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeMark {
    /// A turn is running, in the foreground or in the background.
    Working,
    /// A finished reply is waiting to be read.
    NewReply,
    /// A prompt is queued for its turn, but nothing is failed.
    Queued,
    /// The chat's child is coming up from the click of it.
    Opening,
    /// An extension dialog wants an answer.
    NeedsAttention,
    /// The chat's child is gone until it is restarted.
    Ended,
}

/// Which family of the theme's colours a badge and its title draw in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColourRole {
    /// The accent the open chat's row is highlighted with: something is
    /// running there.
    Accent,
    /// A finished reply is waiting to be read.
    Success,
    /// A prompt is queued for its turn or a child is coming up, but nothing
    /// is failed.
    Muted,
    /// An extension dialog wants an answer.
    Warning,
    /// The chat's child is gone until it is restarted.
    Error,
}

/// The plain drawing data one chat's activity turns into; idle draws nothing
/// ([`Badge::on`] returns `None`), so the window needs no badge for most rows
/// in a big home.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Badge {
    /// The mark itself: a meaning, never a glyph, so the window keeps the
    /// drawing and this rule stays UI-agnostic.
    pub mark: BadgeMark,
    /// The colour family the mark draws in.
    pub role: ColourRole,
    /// Whether the title is drawn bold, so a finished reply is seen without
    /// reading a mark; only the unread case asks for it.
    pub bold_title: bool,
}

impl Badge {
    /// The badge for one chat's state; `None` draws nothing (Idle).
    pub fn on(activity: ChatActivity) -> Option<Self> {
        Some(match activity {
            ChatActivity::Idle => return None,
            ChatActivity::Working => Self {
                mark: BadgeMark::Working,
                role: ColourRole::Accent,
                bold_title: false,
            },
            ChatActivity::FinishedUnread => Self {
                mark: BadgeMark::NewReply,
                role: ColourRole::Success,
                bold_title: true,
            },
            ChatActivity::Queued => Self {
                // A prompt waits for its turn, so the mark stays quiet, in
                // the secondary colour.
                mark: BadgeMark::Queued,
                role: ColourRole::Muted,
                bold_title: false,
            },
            ChatActivity::Opening => Self {
                // The chat's child is coming up from the click of it.
                // Progress, not failure, so it draws in the muted colour;
                // still a mark of its own, not the queued one: that is a
                // prompt waiting for its turn, not a child on its way.
                mark: BadgeMark::Opening,
                role: ColourRole::Muted,
                bold_title: false,
            },
            ChatActivity::NeedsAttention => Self {
                mark: BadgeMark::NeedsAttention,
                role: ColourRole::Warning,
                bold_title: false,
            },
            ChatActivity::Ended => Self {
                mark: BadgeMark::Ended,
                role: ColourRole::Error,
                bold_title: false,
            },
        })
    }
}

#[cfg(test)]
mod tests;
