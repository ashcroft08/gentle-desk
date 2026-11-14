// The words of the footer home setting (shell-choice S5).
//
// 0.2.0 (S3, D4): written in BOTH languages, one dictionary per language and a
// builder that answers the whole object, so the control redraws them in the same
// tick it is told the new language (S2). The English column is what the setting
// has drawn since S5, unchanged; the Spanish one is neutral - no voseo, no
// regionalisms - and keeps its accents. `Home` stays `Home`: it is the shell's own
// idea, and the name the Settings section already gives it in Spanish.
import type { LanguageView } from '../contract/bindings'

/** The home setting's own words for one language. */
export interface HomeCopy {
  /** The two answers' group name. */
  group: string
  /** The user's own pi. */
  link: string
  /** A space of its own, beside anything else. */
  separate: string
  /** What a choice still waiting for the next run says. */
  restartNote: string
  restartNow: string
}

/** English: the words the setting already drew. */
const ENGLISH: HomeCopy = {
  group: 'Home',
  link: 'My pi',
  separate: 'Separate',
  restartNote: 'Restart to use this home.',
  restartNow: 'Restart now',
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH: typeof ENGLISH = {
  group: 'Home',
  link: 'Mi pi',
  separate: 'Separado',
  restartNote: 'Reinicia para usar este home.',
  restartNow: 'Reiniciar ahora',
}

/** The home setting's own words in `language`, the whole dictionary at once (D4). */
export function homeCopy(language: LanguageView): typeof ENGLISH {
  return language === 'es' ? SPANISH : ENGLISH
}
