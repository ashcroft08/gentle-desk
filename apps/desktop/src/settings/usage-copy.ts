// The Usage pane's words (settings-0.1.12 S6's deferred half). Plain, and no file
// name of anyone's: the user is told what the numbers are, where they come from, and
// what the one button does.
//
// 0.2.0 (S3, D4): those words are written in BOTH languages here - one dictionary
// per language and a builder that answers the whole object, so the two lines that
// carry the shell's own reason and plan stay functions and no string-keyed lookup is
// introduced. The English column is what the pane has drawn since 0.1.12, unchanged;
// the Spanish one is neutral - no voseo, no regionalisms - and keeps its accents.
//
// The rule the pane repeats wherever it can: a provider without credentials is
// drawn as "no data", never as a used-up plan.
//
// What does NOT move: the provider names, the plan names, the window labels, the
// shares and the countdowns, which are the shell's own readings (D7).
import type { LanguageView } from '../contract/bindings'

/** The Usage pane's own words for one language. */
export interface UsageCopy {
  /** Before the first answer, and while the read runs (the shell's own sources). */
  reading: string
  /**
   * The one reason line the pane draws, for both the read that could not happen
   * (something was unreadable) and the answer it could not use (a run that failed,
   * a document of another shape): the shell's own words follow the colon, so the
   * sentence never repeats them.
   */
  cannotRead: (reason: string) => string
  /** Nothing asked the sources anything yet (the section was never opened). */
  notRead: string
  /** The package is not there: one line, the install button and the retry. */
  missing: string
  missingWhy: string
  install: string
  installNote: string
  /** The two actions every state offers, when they apply. */
  checkAgain: string
  retrySafe: string
  /** One provider's own words: its plan, a no-data answer, and a reached limit. */
  plan: (plan: string) => string
  noData: string
  noDataWhy: string
  limitReached: string
  providerFailed: string
  /** The sources answered, and named no provider at all. */
  noProviders: string
  /**
   * One provider answered, and it reported no window at all (L7, D3): its own
   * sentence, never the line above, which is about the whole read.
   */
  noWindows: string
  /** The quiet line under the cards: what the numbers are and are not. */
  note: string
}

/** English: the words the pane already drew. */
const ENGLISH: UsageCopy = {
  reading: 'Reading the usage...',
  cannotRead: (reason) => `Could not read the usage: ${reason}`,
  notRead: 'The usage has not been read yet.',
  missing: 'The usage sources are not installed.',
  missingWhy:
    "They are the shell's own sources: the same ones its subscription panel reads. This app never installs anything silently, so the button opens a terminal window with the install command and you decide there.",
  install: 'Install in terminal',
  installNote:
    "Install opens a terminal window and runs the shell's own installer; read what it says there.",
  checkAgain: 'Check again',
  retrySafe: 'Checking again is safe.',
  plan: (plan) => `Plan: ${plan}`,
  noData: 'No data',
  noDataWhy: 'A provider without credentials reports no data, never 0%.',
  limitReached: 'Limit reached',
  providerFailed: 'Could not read this provider',
  noProviders: 'The usage sources are not reporting any provider right now.',
  noWindows: 'No windows',
  note: "These numbers come from the shell's own sources, the same ones its subscription panel reads: one row per window, with the share used and when it resets.",
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH: typeof ENGLISH = {
  reading: 'Leyendo el uso...',
  cannotRead: (reason) => `No se pudo leer el uso: ${reason}`,
  notRead: 'Todavía no se ha leído el uso.',
  missing: 'Las fuentes de uso no están instaladas.',
  missingWhy:
    'Son las propias fuentes de la shell: las mismas que lee su panel de suscripción. Esta app nunca instala nada en silencio, así que el botón abre una ventana de terminal con el comando de instalación y tú decides allí.',
  install: 'Instalar en la terminal',
  installNote:
    'Instalar abre una ventana de terminal y ejecuta el instalador de la shell; lee lo que dice allí.',
  checkAgain: 'Comprobar de nuevo',
  retrySafe: 'Volver a comprobar es seguro.',
  // `Plan` is the same word in Spanish, so the line reads the same: the plan it
  // carries is the shell's and is never touched.
  plan: (plan) => `Plan: ${plan}`,
  noData: 'Sin datos',
  noDataWhy: 'Un proveedor sin credenciales informa "sin datos", nunca 0%.',
  limitReached: 'Límite alcanzado',
  providerFailed: 'No se pudo leer este proveedor',
  noProviders: 'Las fuentes de uso no informan de ningún proveedor ahora mismo.',
  noWindows: 'Sin ventanas',
  note: 'Estos números vienen de las propias fuentes de la shell, las mismas que lee su panel de suscripción: una fila por ventana, con la parte usada y cuándo se reinicia.',
}

/** The pane's own words in `language`, the whole dictionary at once (D4). */
export function usageCopy(language: LanguageView): typeof ENGLISH {
  return language === 'es' ? SPANISH : ENGLISH
}
