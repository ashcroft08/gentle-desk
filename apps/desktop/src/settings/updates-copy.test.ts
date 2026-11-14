// The Updates pane's own words (0.2.0, S3 and D4): the three states, the three
// actions, the card's two tags, the three piece names, the headlines, the quiet
// versions row and the chats-closed gate live in ONE language-keyed dictionary.
// This file pins the two columns and the lines that carry a number or a count; the
// words a DRAWN pane shows are pinned in `updates-copy.flip.test.ts`.
//
// The English column is quoted literally: it is what the pane has drawn since
// 0.1.12, so a change of copy is a change here. The versions are the bridge's and
// travel through these words unchanged (D7); `pi` and `shell` are the names the
// terminal gives those two pieces, so they are never respelled either.
import { describe, expect, it } from 'vitest'
import { updatesCopy } from './updates-copy'

/** The English words, exactly as the pane drew them before 0.2.0. */
const ENGLISH = {
  checking: 'Checking...',
  notChecked: 'Not checked yet.',
  checkAgain: 'Check again',
  update: 'Update',
  updateExtensions: 'Update extensions',
  updateAvailable: 'Update available',
  upToDate: 'Everything is up to date',
  appLabel: 'This app',
  piLabel: 'pi',
  shellLabel: 'shell',
  unknownVersion: 'unknown',
  separator: ' \u00b7 ',
  updateInTerminal:
    "Update opens a terminal window and runs the shell's own update; read what it says there.",
  updateExtensionsInTerminal:
    'Update extensions opens a terminal window and updates the packages your shell has installed. There is no way to tell which ones are behind, so this button is always offered and safe to use any time.',
  appUpdatesItself: 'This app updates itself: use the update banner at the top.',
}

/** The Spanish words: neutral Spanish, accents allowed. */
const SPANISH = {
  checking: 'Comprobando...',
  notChecked: 'Todavía sin comprobar.',
  checkAgain: 'Comprobar de nuevo',
  update: 'Actualizar',
  updateExtensions: 'Actualizar extensiones',
  updateAvailable: 'Actualización disponible',
  upToDate: 'Todo está al día',
  appLabel: 'Esta app',
  piLabel: 'pi',
  shellLabel: 'shell',
  unknownVersion: 'desconocida',
  separator: ' \u00b7 ',
  updateInTerminal:
    'Actualizar abre una ventana de terminal y ejecuta la actualización de la propia shell; lee lo que dice allí.',
  updateExtensionsInTerminal:
    'Actualizar extensiones abre una ventana de terminal y actualiza los paquetes que tienes instalados en la shell. No hay forma de saber cuáles están atrasados, así que este botón se ofrece siempre y es seguro usarlo en cualquier momento.',
  appUpdatesItself: 'Esta app se actualiza sola: usa el aviso de actualización de la parte superior.',
}

/** The plain words of one language: the three lines below carry a number. */
const WORDS = Object.keys(ENGLISH) as (keyof typeof ENGLISH)[]

/**
 * `pi` and `shell` are the names the terminal gives those two pieces, and the
 * separator is punctuation, not a word: neither is respelled.
 */
const SAME_IN_BOTH = new Set<keyof typeof ENGLISH>(['piLabel', 'shellLabel', 'separator'])

/** The words of one language, as a template reads them. */
function words(language: 'en' | 'es'): Record<string, string> {
  const copy = updatesCopy(language)
  return Object.fromEntries(WORDS.map((word) => [word, copy[word]]))
}

describe('the updates copy', () => {
  it('draws the English words the pane already had', () => {
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

  it('spells a headline with the name it is given and the versions it was sent', () => {
    const english = updatesCopy('en')
    const spanish = updatesCopy('es')

    expect(english.headline('pi', '1.5.2', '2.0.5')).toBe('pi 1.5.2 \u2192 2.0.5')
    expect(spanish.headline('Esta app', '0.1.11', '0.1.12')).toBe('Esta app 0.1.11 \u2192 0.1.12')
    // A version the check could not read is drawn as never a guess (S3).
    expect(english.headline('shell', '4.7.0', null)).toBe('shell 4.7.0 \u2192 unknown')
    expect(spanish.headline('pi', null, null)).toBe('pi desconocida \u2192 desconocida')
  })

  it('joins the quiet versions row with the same middle dot in both languages', () => {
    const lines = ['This app 0.1.12', 'pi 1.5.2', 'shell 4.7.0']

    expect(updatesCopy('en').versions(lines)).toBe('This app 0.1.12 \u00b7 pi 1.5.2 \u00b7 shell 4.7.0')
    expect(updatesCopy('es').versions(lines)).toBe(
      'This app 0.1.12 \u00b7 pi 1.5.2 \u00b7 shell 4.7.0',
    )
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
})
