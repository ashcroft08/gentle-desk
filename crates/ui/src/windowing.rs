//! Which rows of a long fixed-height list are really drawn.
//!
//! A scrollable list of N rows is represented by the rows that intersect the
//! viewport (plus a few spare ones) between two blank spacers, so the content
//! keeps its full height and the scrollbar stays honest while only a handful of
//! widgets exist. Pure arithmetic, no drawing: the view feeds it the scroll
//! offset and the viewport height and draws what it returns.

use std::ops::Range;

/// The slice of a list to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct Window {
    /// Indices of the rows to build.
    pub rows: Range<usize>,
    /// Blank height standing in for the rows above the window.
    pub space_above: f32,
    /// Blank height standing in for the rows below the window.
    pub space_below: f32,
}

/// The window for `total` rows of `row_height` pixels, scrolled `offset` pixels
/// down, in a viewport `viewport` pixels tall, with `overscan` extra rows on
/// each side so a fast scroll never shows a gap before the next redraw.
pub fn window(
    total: usize,
    row_height: f32,
    offset: f32,
    viewport: f32,
    overscan: usize,
) -> Window {
    if total == 0 || !(row_height.is_finite() && row_height > 0.0) {
        return Window {
            rows: 0..0,
            space_above: 0.0,
            space_below: 0.0,
        };
    }
    // An unknown viewport (before the first layout) counts as empty: only the
    // spare rows are built, which is enough for a first frame.
    let viewport = if viewport.is_finite() && viewport > 0.0 {
        viewport
    } else {
        0.0
    };
    let content = total as f32 * row_height;
    let furthest = (content - viewport).max(0.0);
    let offset = if offset.is_nan() {
        0.0
    } else {
        offset.clamp(0.0, furthest)
    };

    let first_visible = ((offset / row_height).floor() as usize).min(total);
    let last_visible = (((offset + viewport) / row_height).ceil() as usize).min(total);
    let first = first_visible.saturating_sub(overscan);
    let last = last_visible.saturating_add(overscan).min(total);
    Window {
        rows: first..last,
        space_above: first as f32 * row_height,
        space_below: (total - last) as f32 * row_height,
    }
}

#[cfg(test)]
mod tests;
