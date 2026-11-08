//! From the backend's index rows to what the sidebar shows.

use gentle_app::SessionItem;
use gentle_backend::session_index::SessionEntry;

/// Longest title shown, in characters.
const TITLE_CHARS: usize = 60;

pub fn to_item(entry: &SessionEntry) -> SessionItem {
    let summary = &entry.summary;
    SessionItem {
        path: entry.path.to_string_lossy().into_owned(),
        title: title(summary.name.as_deref(), summary.first_message.as_deref()),
        cwd: summary.cwd.clone(),
        modified_ms: entry.modified_ms,
        message_count: summary.message_count,
    }
}

/// The name if the session has one, else the first line of the first message.
fn title(name: Option<&str>, first_message: Option<&str>) -> String {
    let text = name
        .or(first_message)
        .and_then(|text| text.lines().map(str::trim).find(|line| !line.is_empty()))
        .unwrap_or("(no messages)");
    let mut shown: String = text.chars().take(TITLE_CHARS).collect();
    if text.chars().count() > TITLE_CHARS {
        shown.push('…');
    }
    shown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_beats_the_first_message() {
        assert_eq!(title(Some("Mi chat"), Some("hola")), "Mi chat");
    }

    #[test]
    fn the_first_non_blank_line_of_the_first_message_is_used() {
        assert_eq!(
            title(None, Some("\n  \n  hola mundo \nsegunda")),
            "hola mundo"
        );
    }

    #[test]
    fn nothing_to_show_says_so() {
        assert_eq!(title(None, None), "(no messages)");
        assert_eq!(title(None, Some("  \n ")), "(no messages)");
    }

    #[test]
    fn long_titles_are_cut_on_a_character_boundary_with_an_ellipsis() {
        let long = "ñ".repeat(80);
        let shown = title(None, Some(&long));
        assert_eq!(shown.chars().count(), TITLE_CHARS + 1);
        assert!(shown.ends_with('…'));
        assert_eq!(
            title(None, Some(&"a".repeat(TITLE_CHARS))),
            "a".repeat(TITLE_CHARS)
        );
    }
}
