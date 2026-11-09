//! The glyphs one sidebar row draws for its chat's state (S6).
//!
//! The rule that turns a [`ChatActivity`] into a mark, a colour role and a
//! bold title moved to `gentle_runtime::badge` so any window can use it; this
//! module re-exports that plain data and keeps the only iced-side half of the
//! rule: the exact glyph each mark has drawn so far, shorthanded so the rows
//! never change before the Tauri parity. Working and NewReply share a dot
//! today; the new design's different shape is a decision for the drawing, not
//! for the runtime data.
//!
//! Idle draws nothing: most rows in a big home are idle, and a mark would
//! carry no information.

pub use gentle_runtime::badge::{Badge, BadgeMark, ColourRole};

/// How many characters of the title's line a badge costs, no matter which
/// one: the title truncates this much sooner and the row keeps its height and
/// its single line.
pub const TITLE_ROOM: usize = 2;

/// The glyph the row draws for a mark, exactly as they have been drawn so
/// far: at most [`TITLE_ROOM`] characters wide.
pub fn glyph(mark: BadgeMark) -> &'static str {
    match mark {
        BadgeMark::Working => "\u{25CF}",  // filled circle: a working turn
        BadgeMark::NewReply => "\u{25CF}", // same dot today: the shared shape
        BadgeMark::Queued => "\u{2026}",   // clock-like ellipsis: on its way
        BadgeMark::Opening => "\u{25CC}",  // empty circle: the child coming up
        BadgeMark::NeedsAttention => "!",  // a dialog wants an answer
        BadgeMark::Ended => "\u{00D7}",    // a cross: the child is gone
    }
}

#[cfg(test)]
mod tests;
