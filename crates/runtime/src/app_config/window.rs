//! Where the window opens: the rectangle and the flags it was closed with, and
//! the decision of whether that still lands on a screen.
//!
//! Two things live here, and neither touches a window or the file system:
//!
//! - [`WindowState`]: the stored rectangle and the three flags (`maximized`,
//!   `fullscreen`, `minimized`), in integer pixels. An ADDITIVE field of version
//!   1, read LENIENTLY like the theme and the chosen home: absent, unknown-shaped
//!   or mistyped is "nothing stored", never a damaged file, so the workspaces
//!   below it survive and the document version does not move.
//! - [`placement`]: the pure decision over the stored state and the [`Screen`]s
//!   that exist right now, so a remembered position that no longer lands on a
//!   screen (a monitor was unplugged) opens centred instead of off-screen.
//!
//! The window layer owns every window call and every monitor query: it builds a
//! [`Screen`] per screen that exists and acts on the [`Placement`] it gets back.

use serde::{Deserialize, Deserializer, Serialize};

/// A rectangle a window can open on, in physical pixels: the usable area of one
/// screen that exists right now.
///
/// The caller builds one per screen (that is where the window API lives) and
/// this module only reads it. A screen left of or above the first has a negative
/// origin, as the window API reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Screen {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// How the window was closed, as the config file stores it: where and how big it
/// was, and in which state.
///
/// The field names are persisted, so they stay as they are.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowState {
    /// Left edge, in physical pixels.
    pub x: i32,
    /// Top edge, in physical pixels.
    pub y: i32,
    /// The width it was closed with, in physical pixels.
    pub width: u32,
    /// The height it was closed with, in physical pixels.
    pub height: u32,
    /// Whether it was maximized. Absent reads as not maximized: a document that
    /// stores a rectangle but no flag still has a rectangle worth restoring.
    #[serde(default)]
    pub maximized: bool,
    #[serde(default)]
    pub fullscreen: bool,
    #[serde(default)]
    pub minimized: bool,
}

/// Where the window opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// Centred on the current screen: nothing was stored, or what was stored no
    /// longer lands on a screen.
    Center,
    /// At the stored rectangle, on a screen that still exists.
    Restore(WindowState),
}

/// Where the window opens: the stored rectangle when it lands on one of
/// `screens`, and the centre otherwise (nothing stored, or nothing that lands).
///
/// `screens` is what exists right now, so an empty slice - or a monitor the user
/// has since unplugged - is no screen to land on and the window opens centred.
pub fn placement(stored: Option<WindowState>, screens: &[Screen]) -> Placement {
    let Some(stored) = stored else {
        return Placement::Center;
    };
    if screens.iter().any(|screen| lands_on(&stored, screen)) {
        Placement::Restore(stored)
    } else {
        Placement::Center
    }
}

/// Whether `window` lands on `screen` by a MEANINGFUL area.
///
/// The rule: what the two rectangles share must be at least a quarter of the
/// smaller of them. So a window that covers a quarter of itself (or, when it is
/// bigger than the screen, a quarter of that screen) is one the user can still
/// see and grab, however it was moved; one pixel of overlap, or a sliver of a
/// corner, is not - that is a window left off-screen by a monitor that was
/// unplugged, and it opens centred instead. A rectangle with no area shares
/// nothing and never lands.
fn lands_on(window: &WindowState, screen: &Screen) -> bool {
    let shared = overlap_area(window, screen);
    if shared == 0 {
        return false;
    }
    let window_area = area(window.width, window.height);
    let screen_area = area(screen.width, screen.height);
    shared * 4 >= window_area.min(screen_area)
}

/// The area, in square pixels, that the two rectangles share: nothing when they
/// do not meet. `i128` because both edges are `i32` wide and their product would
/// overflow anything narrower, however far off-screen a stored rectangle is.
fn overlap_area(window: &WindowState, screen: &Screen) -> i128 {
    let left = i64::from(window.x).max(i64::from(screen.x));
    let top = i64::from(window.y).max(i64::from(screen.y));
    let right = (i64::from(window.x) + i64::from(window.width))
        .min(i64::from(screen.x) + i64::from(screen.width));
    let bottom = (i64::from(window.y) + i64::from(window.height))
        .min(i64::from(screen.y) + i64::from(screen.height));
    let width = i128::from(right - left).max(0);
    let height = i128::from(bottom - top).max(0);
    width * height
}

/// The area of a rectangle of that size, in square pixels.
fn area(width: u32, height: u32) -> i128 {
    i128::from(width) * i128::from(height)
}

/// The stored state, or `None` for an absent, unknown-shaped or mistyped value,
/// so the rest of the document (the theme, the workspaces) survives what this
/// field holds. Only a value this version can read as a rectangle (its three
/// flags defaulting to false) is one worth restoring; anything else - another
/// type entirely, or a rectangle with a missing or impossible number - reads as
/// not stored.
pub(crate) fn or_none<'de, D>(deserializer: D) -> Result<Option<WindowState>, D::Error>
where
    D: Deserializer<'de>,
{
    let stored = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(stored.and_then(|value| serde_json::from_value(value).ok()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_config::{AppConfig, Theme, CONFIG_VERSION};

    /// The only screen in most tests: a 1920x1080 primary at the origin.
    const SCREEN: Screen = Screen {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
    };

    /// A 1280x800 window, not maximized, fullscreen or minimized.
    fn stored_at(x: i32, y: i32) -> WindowState {
        WindowState {
            x,
            y,
            width: 1280,
            height: 800,
            maximized: false,
            fullscreen: false,
            minimized: false,
        }
    }

    #[test]
    fn a_stored_rectangle_on_a_single_screen_restores() {
        let stored = stored_at(100, 80);
        assert_eq!(
            placement(Some(stored), &[SCREEN]),
            Placement::Restore(stored)
        );
    }

    #[test]
    fn a_stored_rectangle_on_a_second_screen_restores() {
        // A monitor to the left of the primary reports a negative origin.
        let left = Screen {
            x: -1440,
            y: 0,
            width: 1440,
            height: 900,
        };
        let stored = stored_at(-1440, 40);
        assert_eq!(
            placement(Some(stored), &[SCREEN, left]),
            Placement::Restore(stored)
        );
    }

    #[test]
    fn an_off_screen_rectangle_centres() {
        // The rectangle a monitor that was unplugged used to hold.
        let stored = stored_at(3000, 400);
        assert_eq!(placement(Some(stored), &[SCREEN]), Placement::Center);
        assert_eq!(
            placement(Some(stored), &[]),
            Placement::Center,
            "no screen at all is no screen to land on"
        );
    }

    #[test]
    fn a_sliver_on_screen_centres() {
        // One pixel of an 800x600 window on the screen's right edge: it touches
        // the screen, but not by an area anyone could grab.
        let stored = WindowState {
            x: 1919,
            y: 200,
            width: 800,
            height: 600,
            maximized: false,
            fullscreen: false,
            minimized: false,
        };
        assert_eq!(placement(Some(stored), &[SCREEN]), Placement::Center);
    }

    #[test]
    fn a_last_corner_on_screen_centres() {
        // The bottom-right corner of a window whose rest is past the screen's
        // bottom-right corner: a 20x20 sliver, and nothing to grab.
        let stored = WindowState {
            x: 1900,
            y: 1060,
            width: 800,
            height: 600,
            maximized: false,
            fullscreen: false,
            minimized: false,
        };
        assert_eq!(placement(Some(stored), &[SCREEN]), Placement::Center);
    }

    #[test]
    fn nothing_stored_centres() {
        assert_eq!(placement(None, &[SCREEN]), Placement::Center);
    }

    #[test]
    fn a_window_larger_than_the_screen_restores_when_it_covers_it() {
        // 2560x1440 on the only 1920x1080 screen: bigger than the screen, so
        // "sensibly" is measured against the screen - it covers all of it - and
        // the window opens as it was closed.
        let stored = WindowState {
            x: 0,
            y: 0,
            width: 2560,
            height: 1440,
            maximized: false,
            fullscreen: false,
            minimized: false,
        };
        assert_eq!(
            placement(Some(stored), &[SCREEN]),
            Placement::Restore(stored)
        );
    }

    #[test]
    fn a_window_larger_than_the_screen_centres_when_only_a_corner_is_on_it() {
        let stored = WindowState {
            x: 1900,
            y: 1060,
            width: 2560,
            height: 1440,
            maximized: false,
            fullscreen: false,
            minimized: false,
        };
        assert_eq!(placement(Some(stored), &[SCREEN]), Placement::Center);
    }

    #[test]
    fn the_quarter_rule_is_the_boundary() {
        // A 1280x800 window whose last quarter (320px) is on screen restores; a
        // pixel less centres. The rule is stated on `lands_on`.
        let quarter = stored_at(-960, 100);
        assert_eq!(
            placement(Some(quarter), &[SCREEN]),
            Placement::Restore(quarter)
        );
        let less = stored_at(-961, 100);
        assert_eq!(placement(Some(less), &[SCREEN]), Placement::Center);
    }

    #[test]
    fn a_zero_sized_rectangle_centres() {
        let stored = WindowState {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            maximized: false,
            fullscreen: false,
            minimized: false,
        };
        assert_eq!(placement(Some(stored), &[SCREEN]), Placement::Center);
    }

    #[test]
    fn a_default_config_stores_no_window_state() {
        assert_eq!(AppConfig::default().window, None);
    }

    #[test]
    fn a_config_without_the_window_field_loads_with_nothing_stored() {
        let data =
            br#"{"version":1,"theme":"dark","workspaces":[{"path":"C:/dev/p","collapsed":false}]}"#;
        let config: AppConfig = serde_json::from_slice(data).expect("a v1 document loads");
        assert_eq!(config.window, None);
        assert_eq!(config.version, CONFIG_VERSION);
        assert_eq!(config.version, 1);
        assert_eq!(config.theme, Theme::Dark);
        assert_eq!(config.workspaces[0].path, "C:/dev/p");
    }

    #[test]
    fn a_stored_window_round_trips_through_the_document() {
        let config = AppConfig {
            window: Some(WindowState {
                x: -1440,
                y: 40,
                width: 1280,
                height: 800,
                maximized: true,
                fullscreen: false,
                minimized: true,
            }),
            ..AppConfig::default()
        };
        let bytes = serde_json::to_vec(&config).expect("the document writes");
        let back: AppConfig = serde_json::from_slice(&bytes).expect("and reads back");
        assert_eq!(back.window, config.window);
    }

    #[test]
    fn a_window_without_the_flags_reads_as_not_maximized_fullscreen_or_minimized() {
        // A stored rectangle with no flag is still a rectangle worth restoring,
        // so the three flags default to false instead of the whole value being
        // dropped.
        let data =
            br#"{"version":1,"window":{"x":0,"y":0,"width":800,"height":600},"workspaces":[]}"#;
        let config: AppConfig = serde_json::from_slice(data).expect("a v1 document loads");
        assert_eq!(
            config.window,
            Some(WindowState {
                x: 0,
                y: 0,
                width: 800,
                height: 600,
                maximized: false,
                fullscreen: false,
                minimized: false,
            })
        );
    }

    #[test]
    fn an_unknown_shaped_window_loads_with_nothing_stored_and_keeps_the_rest() {
        let values: [&[u8]; 6] = [
            br#""maximized""#,
            br#"42"#,
            br#"null"#,
            br#"{"x":10}"#,
            br#"{"x":"10","y":0,"width":800,"height":600}"#,
            br#"{"x":0,"y":0,"width":-800,"height":600}"#,
        ];
        for value in values {
            let data = format!(
                r#"{{"version":1,"theme":"Gentle","workspaces":[{{"path":"C:/dev/p","collapsed":true}}],"window":{}}}"#,
                std::str::from_utf8(value).expect("test input is text")
            );
            let config: AppConfig = serde_json::from_str(&data)
                .unwrap_or_else(|error| panic!("{data} is not a damaged file: {error}"));
            assert_eq!(config.window, None, "{data} stores nothing");
            assert_eq!(config.theme, Theme::Gentle, "the theme survives {data}");
            assert!(
                config.workspaces[0].collapsed,
                "the workspaces survive {data}"
            );
        }
    }
}
