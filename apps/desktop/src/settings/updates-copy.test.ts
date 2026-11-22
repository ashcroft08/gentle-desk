// The Updates pane's own words (0.2.0, S3 and D4; 0.8.1 settings-polish D10-D14):
// the states, the four actions, the card's three tags, the three piece names, the
// sentences of a failed read, the last-checked line and the chats-closed gate live
// in ONE language-keyed dictionary. This file pins the two columns and the lines
// that carry a number or a count; the words a DRAWN pane shows are pinned in
// `updates-copy.flip.test.ts` and `UpdatesPane.test.ts`.
//
// The English column is quoted literally, so a change of copy is a change here. The
// versions are the bridge's and travel through these words unchanged (D7); `pi` is
// the name the terminal gives that piece and `Gentle Shell` is the product's (D10),
// so neither is respelled per language.
import { describe, expect, it } from 'vitest'
import { updatesCopy } from './updates-copy'

/** The plain words of one language: everything that is not a sentence builder. */
const ENGLISH = {
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
  updateInTerminal:
    'Update and Update Gentle Shell open a terminal window; read what it says there.',
  extensionsNote:
    'There is no way to tell which ones are behind with this Gentle Shell. Update extensions updates all of them, so this button is always offered and safe to use any time.',
  appUpdatesItself: 'This app updates itself: use the update banner at the top.',
  checkedNever: 'Last checked: never',
  packagesUpToDate: 'The extensions are up to date.',
  packagesUnreadable: 'Could not check the extensions: pi did not answer.',
  packagesNeedsSession: 'You can check it by opening a session.',
}

/** The Spanish words: neutral Spanish, accents allowed. */
const SPANISH = {
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
  piLabel: 'pi',
  shellLabel: 'Gentle Shell',
  unknownVersion: 'desconocida',
  updateInTerminal:
    'Actualizar y Actualizar Gentle Shell abren una ventana de terminal; lee lo que dice allí.',
  extensionsNote:
    'No se puede saber cuáles están desactualizadas con este Gentle Shell. Actualizar extensiones las actualiza todas, así que este botón se ofrece siempre y es seguro usarlo en cualquier momento.',
  appUpdatesItself: 'Esta app se actualiza sola: usa el aviso de actualización de la parte superior.',
  checkedNever: 'Última comprobación: nunca',
  packagesUpToDate: 'Las extensiones están al día.',
  packagesUnreadable: 'No se pudo revisar las extensiones: pi no respondió.',
  packagesNeedsSession: 'Se puede revisar al abrir una sesión.',
}

const WORDS = Object.keys(ENGLISH) as (keyof typeof ENGLISH)[]

/**
 * `pi` is the name the terminal gives that piece, `Gentle Shell` is the product's
 * (D10), and the two versions are facts: none of the three is translated.
 */
const SAME_IN_BOTH = new Set<keyof typeof ENGLISH>(['piLabel', 'shellLabel'])

/** The words of one language, as a template reads them. */
function words(language: 'en' | 'es'): Record<string, string> {
  const copy = updatesCopy(language)
  return Object.fromEntries(WORDS.map((word) => [word, copy[word]]))
}

describe('the updates copy', () => {
  it('draws the English words the pane draws', () => {
    expect(words('en')).toEqual(ENGLISH)
  })

  it('draws Spanish for the same shape', () => {
    expect(words('es')).toEqual(SPANISH)
  })

  it('translates every word: nothing is left in English', () => {
    const english = updatesCopy('en')
    const spanish = updatesCopy('es')

    expect(Object.keys(spanish).sort()).toEqual(Object.keys(english).sort())
    for (const word of WORDS) {
      if (SAME_IN_BOTH.has(word)) {
        expect(spanish[word], word).toBe(english[word])
        continue
      }
      expect(spanish[word], word).not.toBe(english[word])
    }
  })

  it('says why a lookup failed with the short clause, and which piece it was', () => {
    const english = updatesCopy('en')
    const spanish = updatesCopy('es')

    // The artboard's three drawn sentences, character for character.
    expect(english.rowReason('shell', 'Offline')).toBe(
      'Could not check the latest gentle-shell version on npm: offline.',
    )
    expect(english.rowReason('pi', 'Offline')).toBe(
      'Could not check the latest pi version on npm: offline.',
    )
    expect(english.rowReason('app', 'NoAnswer')).toBe(
      'Could not check the latest version of this app: the release manifest did not answer.',
    )
    expect(spanish.rowReason('shell', 'Offline')).toBe(
      'No se pudo comprobar la última versión de gentle-shell en npm: sin conexión.',
    )
    expect(spanish.rowReason('pi', 'Offline')).toBe(
      'No se pudo comprobar la última versión de pi en npm: sin conexión.',
    )
    expect(spanish.rowReason('app', 'NoAnswer')).toBe(
      'No se pudo comprobar la última versión de esta app: el manifiesto de versiones no respondió.',
    )
    // D12: the second clause, the one a read that answered nothing gets.
    expect(english.rowReason('pi', 'NoAnswer')).toBe(
      'Could not check the latest pi version on npm: the registry did not answer.',
    )
    // The installed-version probe is a sentence of its own, with no clause: the row
    // draws the unknown word beside it.
    expect(english.rowReason('shell', 'InstalledUnreadable')).toBe(
      'Could not read the installed version.',
    )
    expect(spanish.rowReason('pi', 'InstalledUnreadable')).toBe(
      'No se pudo leer la versión instalada.',
    )
  })

  it('never lets the client message through: the sentence is built from the clause', () => {
    // Nothing of an address, a status code or an English client error can appear:
    // the builder only ever sees the enum.
    for (const reason of ['Offline', 'NoAnswer', 'InstalledUnreadable'] as const) {
      for (const row of ['app', 'pi', 'shell'] as const) {
        const sentence = updatesCopy('es').rowReason(row, reason)
        expect(sentence).not.toMatch(/https?:|answered \d|error|npmjs/)
        expect(sentence.endsWith('.')).toBe(true)
      }
    }
  })

  it('says what a skip means, with the version it was given', () => {
    expect(updatesCopy('en').skippedNote('4.9.2')).toBe(
      'You skipped 4.9.2: it is not offered again when the app opens.',
    )
    expect(updatesCopy('es').skippedNote('4.9.2')).toBe(
      'Omitiste la 4.9.2: no se vuelve a ofrecer al abrir la app.',
    )
  })

  it('words the last check with the house age, and the one word for something just read', () => {
    const english = updatesCopy('en')
    const spanish = updatesCopy('es')

    // D11: the age is `ago`'s own (`3m`), never a second formatter's `3 min`.
    expect(english.checkedAt('3m', false)).toBe('Last checked: 3m ago')
    expect(spanish.checkedAt('3m', false)).toBe('Última comprobación: hace 3m')
    expect(english.checkedAt('now', true)).toBe('Last checked: just now')
    expect(spanish.checkedAt('ahora', true)).toBe('Última comprobación: ahora mismo')
    // Before the first check the same line says it, once (D13).
    expect(english.checkedNever).toBe('Last checked: never')
    expect(spanish.checkedNever).toBe('Última comprobación: nunca')
  })

  it('counts the open chats in the gate line, one chat or many', () => {
    const english = updatesCopy('en')
    const spanish = updatesCopy('es')

    expect(english.gateHint(1)).toBe(
      'Updates need every chat closed. 1 chat is open - Close chat in its row menu, and the update buttons come alive.',
    )
    expect(english.gateHint(3)).toContain('3 chats are open - Close chat in their row menu')
    expect(spanish.gateHint(1)).toBe(
      'Las actualizaciones necesitan todos los chats cerrados. 1 chat está abierto - Cerrar chat en el menú de su fila y los botones de actualización se activan.',
    )
    expect(spanish.gateHint(3)).toContain('3 chats están abiertos')
  })

  it('counts the extensions that are behind, one or many (packages-status S1)', () => {
    const english = updatesCopy('en')
    const spanish = updatesCopy('es')

    // The plural-safe line: the number is the count, and both languages carry
    // the singular of it.
    expect(english.packagesBehind(1)).toBe('1 extension has an update available')
    expect(english.packagesBehind(4)).toBe('4 extensions have an update available')
    expect(spanish.packagesBehind(1)).toBe('1 extensión con actualización disponible')
    expect(spanish.packagesBehind(4)).toBe('4 extensiones con actualización disponible')
  })

  it('never claims a read that did not run: the clause is the app\'s own', () => {
    // The packages read carries no reason of its own, so the block says the short
    // generic clause - never the client's message, never an address (S1, S4).
    for (const language of ['en', 'es'] as const) {
      const sentence = updatesCopy(language).packagesUnreadable
      expect(sentence).not.toMatch(/https?:|\d{3}/)
      expect(sentence.endsWith('.')).toBe(true)
    }
  })
})
