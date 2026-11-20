//! The window's size limits, read from `tauri.conf.json`.
//!
//! The app has a MINIMUM and no maximum. Without a minimum the window could be
//! dragged down to a few pixels, where the sidebar, the chat header and the
//! composer overlap and the welcome view has nothing sensible to do (the layout
//! rules are `@media` queries, so they only work down to a floor the window
//! enforces). A maximum would be the opposite mistake: it would stop the window
//! from being maximized on a large monitor. The welcome view already caps its own
//! column, so a wide window does not stretch it.

/// The smallest window the app is drawn for, in logical pixels.
const MIN_WIDTH: u64 = 800;
const MIN_HEIGHT: u64 = 560;

fn window() -> serde_json::Value {
    let config: serde_json::Value =
        serde_json::from_str(include_str!("../tauri.conf.json")).expect("tauri.conf.json is JSON");
    config["app"]["windows"][0].clone()
}

#[test]
fn the_window_cannot_be_dragged_below_the_size_the_app_is_drawn_for() {
    let window = window();

    assert_eq!(window["minWidth"].as_u64(), Some(MIN_WIDTH));
    assert_eq!(window["minHeight"].as_u64(), Some(MIN_HEIGHT));
}

#[test]
fn the_window_has_no_maximum_so_it_can_still_be_maximized() {
    let window = window();

    assert!(
        window.get("maxWidth").is_none(),
        "a maxWidth blocks maximizing"
    );
    assert!(
        window.get("maxHeight").is_none(),
        "a maxHeight blocks maximizing"
    );
}

#[test]
fn the_default_size_is_not_below_the_minimum() {
    let window = window();

    assert!(window["width"].as_u64().unwrap() >= MIN_WIDTH);
    assert!(window["height"].as_u64().unwrap() >= MIN_HEIGHT);
}
