// The six themes, in the order the Appearance section's cards draw them, named
// exactly as the config file stores them, the contract carries them and
// `theme.css` selects them. One list, so a name is never spelled twice in the web
// side; `themes.test.ts` pins it against the stylesheet.
//
// 0.2.0 (S3, D4): the labels the picker DRAWS are written in BOTH languages here,
// one dictionary per language and a builder that answers the whole record, so a
// theme keeps one label per language and no string-keyed lookup is introduced.
// The English column is what the picker has drawn since 01a, unchanged; the
// Spanish one is neutral - no voseo, no regionalisms - and keeps its accents.
//
// Three of the six labels are NOT ours: `Gentle`, `Gentleman-Cute` and
// `Gentleman-Sexy` are the theme names the config stores, the contract carries
// and `theme.css` selects, so they are drawn as themselves in every language
// (S3: what is not this app's own word is shown as it arrives). Only `system`,
// `dark` and `light` are words this app chose to say.
import type { LanguageView, ThemeView } from '../contract/bindings'

/** The six theme names, in the picker's own order: `system` first, `light` last. */
export const THEMES: readonly ThemeView[] = [
  'system',
  'dark',
  'Gentle',
  'Gentleman-Cute',
  'Gentleman-Sexy',
  'light',
]

/**
 * The label the picker draws for each theme, in one language. A theme name is an
 * identifier the config stores, the contract carries and `theme.css` selects, so
 * it is never renamed to read well; the label is only the drawn form of it, kept
 * in the footer and in the button's name. Exhaustive by type, so a theme added to
 * `ThemeView` cannot ship without the word the picker shows for it.
 */
export type ThemeLabelsCopy = Record<ThemeView, string>

/** English: the labels the picker already drew. */
const ENGLISH: ThemeLabelsCopy = {
  system: 'System',
  dark: 'Dark',
  Gentle: 'Gentle',
  'Gentleman-Cute': 'Gentleman-Cute',
  'Gentleman-Sexy': 'Gentleman-Sexy',
  light: 'Light',
}

/**
 * Neutral Spanish: no voseo, no regionalisms, nothing left in English that is
 * ours to translate. The three `Gentle*` identifiers are drawn as they are
 * stored, exactly as in the English column.
 */
const SPANISH: typeof ENGLISH = {
  system: 'Sistema',
  dark: 'Oscuro',
  Gentle: 'Gentle',
  'Gentleman-Cute': 'Gentleman-Cute',
  'Gentleman-Sexy': 'Gentleman-Sexy',
  light: 'Claro',
}

/** The picker's labels in `language`, the whole dictionary at once (D4). */
export function themeLabelsCopy(language: LanguageView): typeof ENGLISH {
  return language === 'es' ? SPANISH : ENGLISH
}
