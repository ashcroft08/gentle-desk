// The chat header's words (0.2.0, S3, D5): the tablist's accessible name, the two
// tab words and the working badge the Helpers tab carries. 01b draws them, and
// `./tabs` keeps the ORDER, the ids and the walk - this module keeps the words, so
// the area that draws them owns them and no string-keyed lookup is introduced.
//
// The shape is one dictionary per language and a builder that answers the whole
// object (D4), so the parametrized badge (one helper singular, every other count
// plural) stays a function and the tablist redraws in the same tick the language
// changes (S2). The English column is what 01b draws, unchanged; the Spanish one is
// neutral - no voseo, no regionalisms - and keeps its accents.
//
// What does NOT move: the chat's TITLE, which is the session's own name and is
// drawn exactly as it is stored (D7), and the count itself, which is a number.
import type { LanguageView } from '../contract/bindings'
import type { ViewTab } from './tabs'

/** The chat header's own words for one language. */
export interface ChatCopy {
  /** The tablist's accessible name. */
  tabsLabel: string
  /** Each tab's own word, in the order the tablist draws them. */
  tabs: Record<ViewTab, string>
  /**
   * The working badge's accessible name: one helper is singular, every other
   * count plural. The badge is drawn only while something runs, so this is never
   * asked for zero.
   */
  workingBadge: (running: number) => string
}

/** English: the words 01b already drew. */
const ENGLISH: ChatCopy = {
  tabsLabel: 'View',
  tabs: { chat: 'Chat', helpers: 'Helpers' },
  workingBadge: (running) => (running === 1 ? '1 helper working' : `${running} helpers working`),
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH: typeof ENGLISH = {
  tabsLabel: 'Vista',
  // "Chat" is the same word in Spanish, so it is shared on purpose.
  tabs: { chat: 'Chat', helpers: 'Ayudantes' },
  workingBadge: (running) =>
    running === 1 ? '1 ayudante trabajando' : `${running} ayudantes trabajando`,
}

/** The chat header's own words in `language`, the whole dictionary at once (D4). */
export function chatCopy(language: LanguageView): typeof ENGLISH {
  return language === 'es' ? SPANISH : ENGLISH
}
