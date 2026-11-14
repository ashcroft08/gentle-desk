//! Boundary cases of S1's rule: exactly `es` is Spanish (not every word that
//! starts with it), and a padded locale is read after trimming.

use gentle_core::i18n::Language;

#[test]
fn only_the_exact_es_subtag_is_spanish_not_every_word_that_starts_with_it() {
    for locale in ["espanic", "estonian", "esXL", "es.es", "es x"] {
        assert_eq!(
            Language::detect_from_locale(locale),
            Language::En,
            "{locale}"
        );
    }
}

#[test]
fn a_locale_with_surrounding_spaces_is_read_after_trimming() {
    for locale in [" es ", " es-ES ", "\tes_AR\n", "  ES-ar"] {
        assert_eq!(
            Language::detect_from_locale(locale),
            Language::Es,
            "{locale:?}"
        );
    }
    for locale in [" en-US ", "\tde\n"] {
        assert_eq!(
            Language::detect_from_locale(locale),
            Language::En,
            "{locale:?}"
        );
    }
}
