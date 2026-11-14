// The notice's words, quoted from the states board
// (design/gentle-desk-v2/00-states-board.html): the cards draw icon buttons, and
// the accessible names below are the ones the board gives them. The level's own
// word lives here too, beside the card's other words, because every level says
// itself in words and not only in colour.
//
// 0.2.0 (S3, D4): those words are written in BOTH languages here - one dictionary
// per language and a builder that answers the whole object, so no string-keyed
// lookup is introduced and a card redraws them in the same tick it is told the
// new language (S2). The English column is what the card has drawn since the
// states board, unchanged; the Spanish one is neutral - no voseo, no
// regionalisms - and keeps its accents.
//
// What does NOT move: the notice's own text, which came from the child (D7).
import type { LanguageView, NoticeLevel } from '../contract/bindings'

/** The notice's own words for one language. */
export interface NoticesCopy {
  /** The action's name: the user restarts that chat, not the message. */
  restart: string
  /** The dismiss button's name, as the board's warning card draws it. */
  dismiss: string
  /** Each level's own word, which screen readers hear before the text. */
  levels: Record<NoticeLevel, string>
}

/** English: the words the card already drew. */
const ENGLISH: NoticesCopy = {
  restart: 'Restart chat',
  dismiss: 'Dismiss',
  levels: { Error: 'Error', Warning: 'Warning', Info: 'Info' },
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH: typeof ENGLISH = {
  restart: 'Reiniciar chat',
  dismiss: 'Descartar',
  // "Error" is the same word in Spanish, so it is shared on purpose.
  levels: { Error: 'Error', Warning: 'Advertencia', Info: 'Información' },
}

/** The notice's own words in `language`, the whole dictionary at once (D4). */
export function noticesCopy(language: LanguageView): typeof ENGLISH {
  return language === 'es' ? SPANISH : ENGLISH
}
