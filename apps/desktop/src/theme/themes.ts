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
