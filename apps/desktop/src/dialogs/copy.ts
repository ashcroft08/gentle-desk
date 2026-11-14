// The dialog words, taken from the iced shell the migration replaces
// (`crates/ui/src/view/dialog.rs`): the same five button labels it drew. The
// extension supplies the title, the message, the options and the hint, so no
// other text is invented here, and nothing of that text is translated (D7).
//
// 0.2.0 (S3, D4): the five labels are written in BOTH languages here - one
// dictionary per language and a builder that answers the whole object - so a
// pending dialog redraws its own buttons in the same tick it is told the new
// language (S2). The English column is what iced drew, unchanged; the Spanish
// one is neutral - no voseo, no regionalisms - and keeps its accents.
import type { LanguageView } from '../contract/bindings'

/** The dialog buttons' own words for one language. */
export interface DialogsCopy {
  /** The confirm dialog's two answers, as iced drew them. */
  yes: string
  no: string
  /** The way out of every dialog. */
  cancel: string
  /** What an input or editor dialog sends its text with. */
  submit: string
}

/** English: the words iced already drew. */
const ENGLISH: DialogsCopy = { yes: 'Yes', no: 'No', cancel: 'Cancel', submit: 'Submit' }

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH: typeof ENGLISH = {
  yes: 'Sí',
  // "No" is the same word in Spanish, so it is shared on purpose.
  no: 'No',
  cancel: 'Cancelar',
  submit: 'Enviar',
}

/** The dialog buttons' words in `language`, the whole dictionary at once (D4). */
export function dialogsCopy(language: LanguageView): typeof ENGLISH {
  return language === 'es' ? SPANISH : ENGLISH
}
