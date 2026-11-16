//! The language value: the two tags it stores and the one rule that picks the
//! starting language from a locale (S1: any Spanish locale starts in Spanish).

use std::collections::HashSet;

use gentle_core::i18n::Language;

#[test]
fn the_stored_tags_are_exactly_en_and_es() {
    assert_eq!(Language::En.as_str(), "en");
    assert_eq!(Language::Es.as_str(), "es");
}

#[test]
fn all_holds_both_languages_in_a_stable_order() {
    assert_eq!(Language::ALL, [Language::En, Language::Es]);
    assert_eq!(
        Language::ALL.map(Language::as_str).to_vec(),
        vec!["en", "es"]
    );
}

#[test]
fn parse_reads_the_canonical_tag_back_and_forth() {
    for language in Language::ALL {
        assert_eq!(Language::parse(language.as_str()), Some(language));
    }
}

#[test]
fn parse_trims_and_ignores_case_but_stays_on_the_two_canonical_tags() {
    assert_eq!(Language::parse("ES"), Some(Language::Es));
    assert_eq!(Language::parse("EN"), Some(Language::En));
    assert_eq!(Language::parse("  es  "), Some(Language::Es));
    assert_eq!(Language::parse(" En\t"), Some(Language::En));
    // A full locale is not a stored value: only `detect_from_locale` guesses.
    for guessed in ["es-ES", "es_MX", "en-US", "espanic"] {
        assert_eq!(Language::parse(guessed), None, "{guessed}");
    }
    for junk in ["", "  ", "xx", "e", "s", "english", "es;q=0.9"] {
        assert_eq!(Language::parse(junk), None, "{junk}");
    }
}

#[test]
fn a_spanish_locale_starts_in_spanish() {
    // "es-" keeps `es` as its primary subtag, so the rule answers Spanish.
    for locale in [
        "es", "es-ES", "es-419", "es-MX", "es-AR", "es_AR", "ES-ar", "es-",
    ] {
        assert_eq!(
            Language::detect_from_locale(locale),
            Language::Es,
            "{locale}"
        );
    }
}

#[test]
fn every_other_locale_starts_in_english() {
    for locale in ["en-US", "de", "fr-FR", "pt-BR", "en", "en-GB"] {
        assert_eq!(
            Language::detect_from_locale(locale),
            Language::En,
            "{locale}"
        );
    }
}

#[test]
fn empty_whitespace_and_one_letter_junk_start_in_english_without_panicking() {
    for locale in ["", " ", "   \t\n", "e", "-", "_", "-es", "e-s"] {
        assert_eq!(
            Language::detect_from_locale(locale),
            Language::En,
            "{locale:?}"
        );
    }
}

#[test]
fn the_language_serializes_as_its_tag() {
    assert_eq!(serde_json::to_string(&Language::En).unwrap(), r#""en""#);
    assert_eq!(serde_json::to_string(&Language::Es).unwrap(), r#""es""#);
    assert_eq!(
        serde_json::to_string(&[Language::En, Language::Es]).unwrap(),
        r#"["en","es"]"#
    );
}

#[test]
fn the_language_deserializes_from_its_tag() {
    assert_eq!(
        serde_json::from_str::<Language>(r#""en""#).unwrap(),
        Language::En
    );
    assert_eq!(
        serde_json::from_str::<Language>(r#""es""#).unwrap(),
        Language::Es
    );
    assert_eq!(
        serde_json::to_string(&serde_json::from_str::<Language>(r#""es""#).unwrap()).unwrap(),
        r#""es""#
    );
}

#[test]
fn an_unknown_stored_tag_fails_here_because_the_config_layer_owns_the_fallback() {
    for stored in [r#""xx""#, r#""es-ES""#, r#"""#, "null"] {
        assert!(
            serde_json::from_str::<Language>(stored).is_err(),
            "{stored}"
        );
    }
}

#[test]
fn the_language_is_a_cheap_value_and_a_hash_key() {
    // Copy: both bindings stay usable without cloning.
    let language = Language::Es;
    let copied = language;
    assert_eq!((language, copied), (Language::Es, Language::Es));

    let mut seen: HashSet<Language> = HashSet::new();
    assert!(seen.insert(Language::En));
    assert!(seen.insert(Language::Es));
    assert!(!seen.insert(Language::En));
    assert_eq!(seen.len(), 2);
    assert!(format!("{language:?}").contains("Es"));
}
