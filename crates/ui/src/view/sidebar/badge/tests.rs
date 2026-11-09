//! The drawing side of the badges: no rule here, only the map from the
//! runtime's semantic mark to the exact glyph the sidebar has always drawn,
//! and the room a mark costs the title.

use super::*;
use gentle_runtime::badge::BadgeMark;

/// Every mark still draws the glyph it drew when the rule lived here, and
/// every glyph fits in the room the title gives up for it, so the row keeps
/// its height and its single line.
#[test]
fn every_mark_still_draws_todays_glyph_inside_the_title_room() {
    for (mark, expected) in [
        (BadgeMark::Working, "\u{25CF}"),
        (BadgeMark::NewReply, "\u{25CF}"),
        (BadgeMark::Queued, "\u{2026}"),
        (BadgeMark::Opening, "\u{25CC}"),
        (BadgeMark::NeedsAttention, "!"),
        (BadgeMark::Ended, "\u{00D7}"),
    ] {
        let drawn = glyph(mark);
        assert_eq!(drawn, expected, "{mark:?} maps to {drawn:?}");
        assert!(
            drawn.chars().count() <= TITLE_ROOM,
            "{mark:?} marks itself with {drawn:?}, wider than the {TITLE} characters it costs",
            TITLE = TITLE_ROOM,
        );
    }
}
