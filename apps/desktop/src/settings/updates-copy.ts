// The Updates pane's words (0.1.12, S2, S3, S4, S7): one plain sentence per state,
// so the user never has to know which command runs underneath. The versions are the
// only numbers, and they are drawn quietly under the headline.
//
// 0.2.0 (S3, D4): those words are written in BOTH languages here - one dictionary
// per language and a builder that answers the whole object, so the lines that carry
// a name, a version or a count stay functions and no string-keyed lookup is
// introduced. The English column is what the pane has drawn since 0.1.12,
// unchanged; the Spanish one is neutral - no voseo, no regionalisms - and keeps its
// accents.
//
// What does NOT move: the versions the check reported (D7), the names `pi` and
// `shell` (the terminal's own) and the middle dot the house's small notes use.
import type { LanguageView } from '../contract/bindings'

/** The Updates pane's own words for one language. */
export interface UpdatesCopy {
  /** The three states that are not a card. */
  checking: string
  notChecked: string
  /** The one check button, and the two actions a terminal window runs. */
  checkAgain: string
  update: string
  updateExtensions: string
  /**
   * The two halves of "skip this version" (0.8.0 settings-polish S1, S2): the
   * action offered beside a newer version, and the way back once that version is
   * the one the user skipped. Plain words, no version: the version itself is the
   * check's (D7) and the two buttons sit beside the headline that names it.
   */
  skipVersion: string
  stopSkipping: string
  /** The card's two tags. */
  updateAvailable: string
  upToDate: string
  /** How each piece of this machine is named: no file, no product's file name. */
  appLabel: string
  piLabel: string
  shellLabel: string
  /** What a version the check could not read is drawn as, never a guess. */
  unknownVersion: string
  /** The quiet row's separator, the middle dot the house's small notes use. */
  separator: string
  /** The headline of one row that is behind: what it runs, and what is published. */
  headline: (label: string, current: string | null, latest: string | null) => string
  /** The quiet row: the version each of the three pieces runs. */
  versions: (lines: string[]) => string
  /** What Update does: it is the terminal's, so the pane says so. */
  updateInTerminal: string
  /**
   * What Update extensions does, and why it is always offered: the shell's own
   * updater with pi's `--extensions` flag passed through, touching only the
   * packages the shell has installed. Nothing can say which of them are behind, so
   * there is no check to wait for and the button is safe to press any time (running
   * it again changes nothing). No version, no promise.
   */
  updateExtensionsInTerminal: string
  /** The app's own release is not a terminal's job: the banner installs it. */
  appUpdatesItself: string
  /**
   * Why the update buttons are dead while a chat is open (S4): the rule, the count
   * that blocks them, and the way out. 0.1.14 added Close chat to the chat row's
   * own menu, so the hint names it: before that the desk had no way to close a chat
   * and the gate could not be satisfied without quitting the app.
   */
  gateHint: (openChats: number) => string
}

/** English: the words the pane already drew. */
const ENGLISH: UpdatesCopy = {
  checking: 'Checking...',
  notChecked: 'Not checked yet.',
  checkAgain: 'Check again',
  update: 'Update',
  updateExtensions: 'Update extensions',
  skipVersion: 'Skip this version',
  stopSkipping: 'Stop skipping',
  updateAvailable: 'Update available',
  upToDate: 'Everything is up to date',
  appLabel: 'This app',
  piLabel: 'pi',
  shellLabel: 'shell',
  unknownVersion: 'unknown',
  separator: ' \u00b7 ',
  headline: (label, current, latest) =>
    `${label} ${current ?? ENGLISH.unknownVersion} \u2192 ${latest ?? ENGLISH.unknownVersion}`,
  versions: (lines) => lines.join(ENGLISH.separator),
  updateInTerminal:
    "Update opens a terminal window and runs the shell's own update; read what it says there.",
  updateExtensionsInTerminal:
    'Update extensions opens a terminal window and updates the packages your shell has installed. There is no way to tell which ones are behind, so this button is always offered and safe to use any time.',
  appUpdatesItself: 'This app updates itself: use the update banner at the top.',
  gateHint: (openChats) => {
    const chats = openChats === 1 ? '1 chat is' : `${openChats} chats are`
    const their = openChats === 1 ? 'its' : 'their'
    return `Updates need every chat closed. ${chats} open - Close chat in ${their} row menu, and the update buttons come alive.`
  },
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH: typeof ENGLISH = {
  checking: 'Comprobando...',
  notChecked: 'Todavía sin comprobar.',
  checkAgain: 'Comprobar de nuevo',
  update: 'Actualizar',
  updateExtensions: 'Actualizar extensiones',
  skipVersion: 'Omitir esta versión',
  stopSkipping: 'Dejar de omitir',
  updateAvailable: 'Actualización disponible',
  upToDate: 'Todo está al día',
  appLabel: 'Esta app',
  // `pi` and `shell` are the names the terminal gives those two pieces.
  piLabel: 'pi',
  shellLabel: 'shell',
  unknownVersion: 'desconocida',
  separator: ' \u00b7 ',
  headline: (label, current, latest) =>
    `${label} ${current ?? SPANISH.unknownVersion} \u2192 ${latest ?? SPANISH.unknownVersion}`,
  versions: (lines) => lines.join(SPANISH.separator),
  updateInTerminal:
    'Actualizar abre una ventana de terminal y ejecuta la actualización de la propia shell; lee lo que dice allí.',
  updateExtensionsInTerminal:
    'Actualizar extensiones abre una ventana de terminal y actualiza los paquetes que tienes instalados en la shell. No hay forma de saber cuáles están atrasados, así que este botón se ofrece siempre y es seguro usarlo en cualquier momento.',
  appUpdatesItself: 'Esta app se actualiza sola: usa el aviso de actualización de la parte superior.',
  gateHint: (openChats) => {
    const chats = openChats === 1 ? '1 chat está' : `${openChats} chats están`
    const open = openChats === 1 ? 'abierto' : 'abiertos'
    return `Las actualizaciones necesitan todos los chats cerrados. ${chats} ${open} - Cerrar chat en el menú de su fila y los botones de actualización se activan.`
  },
}

/** The pane's own words in `language`, the whole dictionary at once (D4). */
export function updatesCopy(language: LanguageView): typeof ENGLISH {
  return language === 'es' ? SPANISH : ENGLISH
}
