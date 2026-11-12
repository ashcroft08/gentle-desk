// The six themes, in the order the footer picker draws them (01a's
// `#theme-list`), named exactly as the config file stores them, the contract
// carries them and `theme.css` selects them. One list, so a name is never
// spelled twice in the web side; `themes.test.ts` pins it against the
// stylesheet.
import type { ThemeView } from '../contract/bindings'

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
 * The label the picker draws for each theme. A theme name is an identifier the
 * config stores, the contract carries and `theme.css` selects, so it is never
 * renamed to read well; the label is only the capitalised form of it, drawn in
 * the footer and in the button's name. Exhaustive by type, so a theme added to
 * `ThemeView` cannot ship without the word the picker shows for it.
 */
export const THEME_LABELS: Record<ThemeView, string> = {
  system: 'System',
  dark: 'Dark',
  Gentle: 'Gentle',
  'Gentleman-Cute': 'Gentleman-Cute',
  'Gentleman-Sexy': 'Gentleman-Sexy',
  light: 'Light',
}

/**
 * The small dot the picker draws beside each theme's name: the theme's own
 * accent, copied from `theme.css` so the list reads at a glance. `system` has no
 * accent of its own, so its dot is the two themes it follows, dark over light.
 * Exhaustive by type, like the labels above.
 */
export const THEME_SWATCHES: Record<ThemeView, string> = {
  system: 'linear-gradient(135deg, #0e1116 0 49%, #f2f4f7 50% 100%)',
  dark: '#7aa2f7',
  Gentle: '#7fb4ca',
  'Gentleman-Cute': '#f095c8',
  'Gentleman-Sexy': '#f43888',
  light: '#2e5bd7',
}

/** The label one theme is drawn with: its identifier, capitalised. */
export function themeLabel(theme: ThemeView): string {
  return THEME_LABELS[theme]
}

/** The fill of one theme's dot: its accent, or the split for `system`. */
export function themeSwatch(theme: ThemeView): string {
  return THEME_SWATCHES[theme]
}
