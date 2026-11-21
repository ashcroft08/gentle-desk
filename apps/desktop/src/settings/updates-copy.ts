// The Updates pane's words (0.1.12, S2, S3, S4, S7; 0.8.1 settings-polish S8):
// one plain sentence per state, so the user never has to know which command runs
// underneath. The versions are the only numbers, and each row draws its own.
//
// 0.2.0 (S3, D4): those words are written in BOTH languages here - one dictionary
// per language and a builder that answers the whole object, so the lines that carry
// a name, a version or a count stay functions and no string-keyed lookup is
// introduced.
//
// What does NOT move: the versions the check reported (D7), the name `pi` (the
// terminal's own) and the announced version of the one row whose product name the
// pane says in full, `Gentle Shell` (0.8.1, D10: it is a product name, so it reads
// the same in both languages).
//
// 0.8.1 (D12, D13, D14) replaces three lines: a failed read is said as a SHORT
// clause and never as the client's message, the "not checked yet" line is gone
// because the last-checked line says `never`, and the extensions note is the one
// sentence that explains why that button has no state.
import type { LanguageView, UpdateReasonView } from '../contract/bindings'

/** Which piece a row is, for the one sentence that names it. */
export type ReasonRow = 'app' | 'pi' | 'shell'

/** The Updates pane's own words for one language. */
export interface UpdatesCopy {
  /** The one state that is not a card. */
  checking: string
  /** The one check button, and the three actions a terminal window runs. */
  checkAgain: string
  update: string
  updateShell: string
  updateExtensions: string
  /** The two halves of "skip this version" (S1, S2). */
  skipVersion: string
  stopSkipping: string
  /** The card's three tags: available, everything read and level, or not read. */
  updateAvailable: string
  upToDate: string
  notCheckedTag: string
  /** How each piece of this machine is named: no file, no product's file name. */
  appLabel: string
  piLabel: string
  shellLabel: string
  /** What a version the check could not read is drawn as, never a guess. */
  unknownVersion: string
  /**
   * Why one row has no answer (S8, D12): which read failed - that row's lookup,
   * or the installed-version probe both rows share - said as a short clause. The
   * client's own message names addresses and is never drawn.
   */
  rowReason: (row: ReasonRow, reason: UpdateReasonView) => string
  /** What a skip means, on the row that carries it. */
  skippedNote: (version: string) => string
  /** What the two terminal actions do, once there is one on screen. */
  updateInTerminal: string
  /** Why the extensions button has no state to wait for (D14). */
  extensionsNote: string
  /** The app's own release is not a terminal's job: the banner installs it. */
  appUpdatesItself: string
  /**
   * When the rows were last read (S8, D11): the house's own age (`hace 3m`) or the
   * one word it answers for something just read, and `never` before any check.
   */
  checkedAt: (age: string, justNow: boolean) => string
  checkedNever: string
  /**
   * Why the update buttons are dead while a chat is open (S4): the rule, the count
   * that blocks them, and the way out.
   */
  gateHint: (openChats: number) => string
}

/** The one sentence a failed read is worded as, in English (S8, D12). */
function rowReason(row: ReasonRow, reason: UpdateReasonView): string {
  if (reason === 'InstalledUnreadable') return 'Could not read the installed version.'
  const clause =
    reason === 'Offline'
      ? 'offline'
      : row === 'app'
        ? 'the release manifest did not answer'
        : 'the registry did not answer'
  const subject =
    row === 'app'
      ? 'the latest version of this app'
      : row === 'pi'
        ? 'the latest pi version on npm'
        : 'the latest gentle-shell version on npm'
  return `Could not check ${subject}: ${clause}.`
}

/** English: the words the pane draws. */
const ENGLISH: UpdatesCopy = {
  checking: 'Checking...',
  checkAgain: 'Check again',
  update: 'Update',
  updateShell: 'Update Gentle Shell',
  updateExtensions: 'Update extensions',
  skipVersion: 'Skip this version',
  stopSkipping: 'Stop skipping',
  updateAvailable: 'Update available',
  upToDate: 'Everything is up to date',
  notCheckedTag: 'Not checked',
  appLabel: 'This app',
  piLabel: 'pi',
  shellLabel: 'Gentle Shell',
  unknownVersion: 'unknown',
  rowReason,
  skippedNote: (version) => `You skipped ${version}: it is not offered again when the app opens.`,
  updateInTerminal:
    'Update and Update Gentle Shell open a terminal window; read what it says there.',
  extensionsNote:
    'There is no way to tell which ones are behind with this Gentle Shell. Update extensions updates all of them, so this button is always offered and safe to use any time.',
  appUpdatesItself: 'This app updates itself: use the update banner at the top.',
  checkedAt: (age, justNow) => (justNow ? 'Last checked: just now' : `Last checked: ${age} ago`),
  checkedNever: 'Last checked: never',
  gateHint: (openChats) => {
    const chats = openChats === 1 ? '1 chat is' : `${openChats} chats are`
    const their = openChats === 1 ? 'its' : 'their'
    return `Updates need every chat closed. ${chats} open - Close chat in ${their} row menu, and the update buttons come alive.`
  },
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH: typeof ENGLISH = {
  checking: 'Comprobando...',
  checkAgain: 'Comprobar de nuevo',
  update: 'Actualizar',
  updateShell: 'Actualizar Gentle Shell',
  updateExtensions: 'Actualizar extensiones',
  skipVersion: 'Omitir esta versión',
  stopSkipping: 'Dejar de omitir',
  updateAvailable: 'Actualización disponible',
  upToDate: 'Todo está al día',
  notCheckedTag: 'Sin comprobar',
  appLabel: 'Esta app',
  // `pi` is the name the terminal gives that piece; `Gentle Shell` is the
  // product's, and a product's name is not translated (D10).
  piLabel: 'pi',
  shellLabel: 'Gentle Shell',
  unknownVersion: 'desconocida',
  rowReason: (row, reason) => {
    if (reason === 'InstalledUnreadable') return 'No se pudo leer la versión instalada.'
    const clause =
      reason === 'Offline'
        ? 'sin conexión'
        : row === 'app'
          ? 'el manifiesto de versiones no respondió'
          : 'el registro no respondió'
    const subject =
      row === 'app'
        ? 'la última versión de esta app'
        : row === 'pi'
          ? 'la última versión de pi en npm'
          : 'la última versión de gentle-shell en npm'
    return `No se pudo comprobar ${subject}: ${clause}.`
  },
  skippedNote: (version) => `Omitiste la ${version}: no se vuelve a ofrecer al abrir la app.`,
  updateInTerminal:
    'Actualizar y Actualizar Gentle Shell abren una ventana de terminal; lee lo que dice allí.',
  extensionsNote:
    'No se puede saber cuáles están desactualizadas con este Gentle Shell. Actualizar extensiones las actualiza todas, así que este botón se ofrece siempre y es seguro usarlo en cualquier momento.',
  appUpdatesItself: 'Esta app se actualiza sola: usa el aviso de actualización de la parte superior.',
  checkedAt: (age, justNow) =>
    justNow ? 'Última comprobación: ahora mismo' : `Última comprobación: hace ${age}`,
  checkedNever: 'Última comprobación: nunca',
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
