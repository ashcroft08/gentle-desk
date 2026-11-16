// The what's-new modal's language tabs (design/whats-new-modal, artboard D): the
// order the row draws, the ids that wire each tab to the one panel it opens, and
// the key walk - the same walk the chat header and the settings rail use, so the
// three tablists of the app move alike.
//
// The tab NAMES are here and not in the copy dictionary: a language names itself,
// so `English` and `Español` are never translated (the 0.2.0 LanguagePane
// convention). What the modal says AROUND them stays in `copy.ts`.
import type { LanguageView } from '../contract/bindings'

/** One tab of the row: the language it reads the notes in, and its own name. */
export interface LanguageTab {
  id: LanguageView
  name: string
}

/** The tabs in the order the row draws them. */
export const LANGUAGES: LanguageTab[] = [
  { id: 'en', name: 'English' },
  { id: 'es', name: 'Español' },
]

/** The id of each tab, so the panel can name it with `aria-labelledby`. */
export const TAB_IDS: Record<LanguageView, string> = {
  en: 'whatsnew-tab-en',
  es: 'whatsnew-tab-es',
}

/** The id of the one panel the tabs open, so each tab points at it with `aria-controls`. */
export const PANEL_ID = 'whatsnew-notes'

/** The order the keys walk. */
const ORDER: LanguageView[] = LANGUAGES.map((tab) => tab.id)

/**
 * The tab a horizontal or vertical move asks for: the arrow keys walk the row and
 * wrap, Home and End jump to the ends. `null` for every other key, which keeps
 * its own meaning.
 */
export function tabAfter(key: string, from: LanguageView): LanguageView | null {
  const index = ORDER.indexOf(from)
  switch (key) {
    case 'ArrowRight':
    case 'ArrowDown':
      return ORDER[(index + 1) % ORDER.length]
    case 'ArrowLeft':
    case 'ArrowUp':
      return ORDER[(index - 1 + ORDER.length) % ORDER.length]
    case 'Home':
      return ORDER[0]
    case 'End':
      return ORDER[ORDER.length - 1]
    default:
      return null
  }
}
