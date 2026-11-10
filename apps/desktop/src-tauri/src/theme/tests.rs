//! The window chrome's one rule: which theme the window itself asks the
//! operating system for. A windowless test, because the rule is a mapping and
//! the window it is applied to is behind the seam (`src/theme.rs`).

use super::*;

#[test]
fn the_six_themes_ask_the_window_for_their_own_chrome() {
    assert_eq!(
        native_theme(Theme::System),
        None,
        "the reader's own setting"
    );
    assert_eq!(native_theme(Theme::Light), Some(tauri::Theme::Light));
    for dark in [
        Theme::Dark,
        Theme::Gentle,
        Theme::GentlemanCute,
        Theme::GentlemanSexy,
    ] {
        assert_eq!(
            native_theme(dark),
            Some(tauri::Theme::Dark),
            "{dark:?} has a dark palette"
        );
    }
}
