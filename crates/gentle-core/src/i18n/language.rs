//! The two languages the app speaks, and the locale rule that picks the first
//! one (S1: the system's language, and any Spanish locale starts in Spanish).

use serde::{Deserialize, Serialize};

/// A language the app's own words can be written in.
///
/// The value travels as its two-letter tag: `"en"` or `"es"` in `config.json`,
/// in the view state and in the web's dictionaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// English.
    En,
    /// Neutral Spanish: no voseo and no regionalisms.
    Es,
}

impl Language {
    /// Both languages, in the order the settings offer them.
    pub const ALL: [Language; 2] = [Language::En, Language::Es];

    /// The tag this language is stored and serialized as: `"en"` or `"es"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Es => "es",
        }
    }

    /// Reads a stored tag, case-insensitively and ignoring surrounding spaces.
    ///
    /// Only the two canonical tags are accepted: a full locale such as
    /// `"es-ES"` is not a stored value and answers [`None`], and so does
    /// anything else. Choosing a language from a locale is
    /// [`Language::detect_from_locale`]'s job, not this one's.
    pub fn parse(tag: &str) -> Option<Language> {
        match tag.trim().to_ascii_lowercase().as_str() {
            "en" => Some(Language::En),
            "es" => Some(Language::Es),
            _ => None,
        }
    }

    /// The language a locale tag starts the app in.
    ///
    /// The primary subtag (before the first `-` or `_`), trimmed of
    /// surrounding spaces and lowercased, decides: exactly `es` starts in
    /// Spanish — not every locale that begins with those letters — so `es`,
    /// `es-ES`, `es-419`, `es_MX`, `ES-ar` and `  es  ` all answer
    /// [`Language::Es`]. Every other input — an empty or whitespace-only
    /// string, one-letter junk, another language — answers [`Language::En`],
    /// and no input makes this panic.
    pub fn detect_from_locale(locale: &str) -> Language {
        let primary = locale
            .split(['-', '_'])
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        if primary == "es" {
            Language::Es
        } else {
            Language::En
        }
    }
}
