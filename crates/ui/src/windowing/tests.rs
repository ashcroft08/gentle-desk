use super::*;

const ROW: f32 = 50.0;

fn rows(w: &Window) -> (usize, usize) {
    (w.rows.start, w.rows.end)
}

/// The window plus its spacers always add up to the full list height.
fn assert_covers(w: &Window, total: usize) {
    let drawn = w.rows.len() as f32 * ROW;
    let sum = w.space_above + drawn + w.space_below;
    let expected = total as f32 * ROW;
    // f32 has about 7 digits: allow a relative error on very tall lists.
    assert!(
        (sum - expected).abs() <= 0.01 + expected * 1e-6,
        "{w:?} covers {sum}, expected {}",
        total as f32 * ROW
    );
}

#[test]
fn an_empty_list_has_an_empty_window() {
    let w = window(0, ROW, 0.0, 400.0, 3);
    assert_eq!(rows(&w), (0, 0));
    assert_eq!((w.space_above, w.space_below), (0.0, 0.0));
}

#[test]
fn at_the_top_it_draws_the_visible_rows_plus_the_overscan_below() {
    // 400 px / 50 px = 8 visible rows, plus 2 spare.
    let w = window(1000, ROW, 0.0, 400.0, 2);
    assert_eq!(rows(&w), (0, 10));
    assert_eq!(w.space_above, 0.0);
    assert_covers(&w, 1000);
}

#[test]
fn in_the_middle_it_draws_around_the_offset() {
    // Offset 1000 px = row 20 is the first visible; 8 visible rows -> 20..28.
    let w = window(1000, ROW, 1000.0, 400.0, 2);
    assert_eq!(rows(&w), (18, 30));
    assert_eq!(w.space_above, 18.0 * ROW);
    assert_eq!(w.space_below, 970.0 * ROW);
    assert_covers(&w, 1000);
}

#[test]
fn a_row_partly_visible_counts_as_visible() {
    // Offset 1025 px: row 20 is half hidden, row 28 is half shown -> 20..29.
    let w = window(1000, ROW, 1025.0, 400.0, 0);
    assert_eq!(rows(&w), (20, 29));
}

#[test]
fn at_the_bottom_it_stops_at_the_last_row() {
    // The content is 50 000 px; the furthest offset is 49 600.
    let w = window(1000, ROW, 49_600.0, 400.0, 2);
    assert_eq!(rows(&w), (990, 1000));
    assert_eq!(w.space_below, 0.0);
    assert_covers(&w, 1000);
}

#[test]
fn an_offset_beyond_the_content_is_clamped_to_the_end() {
    // The list shrank (a search narrowed it) while the old offset was kept.
    let w = window(30, ROW, 50_000.0, 400.0, 2);
    assert_eq!(rows(&w), (20, 30));
    assert_covers(&w, 30);
}

#[test]
fn a_list_shorter_than_the_viewport_is_drawn_whole() {
    let w = window(3, ROW, 0.0, 400.0, 2);
    assert_eq!(rows(&w), (0, 3));
    assert_eq!((w.space_above, w.space_below), (0.0, 0.0));
}

#[test]
fn a_negative_or_nan_offset_means_the_top() {
    for offset in [-300.0, f32::NAN] {
        let w = window(1000, ROW, offset, 400.0, 2);
        assert_eq!(rows(&w), (0, 10), "offset {offset}");
        assert_covers(&w, 1000);
    }
}

#[test]
fn an_unknown_viewport_still_draws_a_first_page_without_panicking() {
    // Before the first layout the height is 0 (or garbage): show the overscan only.
    for viewport in [0.0, -10.0, f32::NAN] {
        let w = window(1000, ROW, 0.0, viewport, 4);
        assert!(w.rows.len() <= 4, "viewport {viewport}: {w:?}");
        assert_eq!(w.rows.start, 0);
        assert_covers(&w, 1000);
    }
}

#[test]
fn a_bad_row_height_draws_nothing_instead_of_dividing_by_zero() {
    for row_height in [0.0, -5.0, f32::NAN, f32::INFINITY] {
        let w = window(1000, row_height, 0.0, 400.0, 2);
        assert_eq!(rows(&w), (0, 0));
    }
}

#[test]
fn the_window_always_covers_the_list_and_stays_in_bounds() {
    for total in [0usize, 1, 7, 8, 9, 100, 5000] {
        for offset in [0.0, 1.0, 49.0, 50.0, 333.0, 10_000.0, 1e9] {
            for overscan in [0usize, 1, 5] {
                let w = window(total, ROW, offset, 400.0, overscan);
                assert!(w.rows.start <= w.rows.end && w.rows.end <= total, "{w:?}");
                assert_covers(&w, total);
            }
        }
    }
}

#[test]
fn a_huge_list_draws_only_a_page_of_rows() {
    let w = window(1_000_000, ROW, 25_000_000.0, 700.0, 5);
    assert!(w.rows.len() <= 14 + 1 + 10, "{} rows", w.rows.len());
    assert_covers(&w, 1_000_000);
}
