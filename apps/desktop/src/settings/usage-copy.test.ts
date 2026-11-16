// The Usage pane's own words (0.2.0, S3 and D4): the pane's states, its three
// actions, one provider's plan and no-data answer, the one reason line and the
// quiet note under the cards live in ONE language-keyed dictionary. This file pins
// the two columns; the words a DRAWN pane shows are pinned in
// `usage-copy.flip.test.ts`.
//
// The English column is quoted literally: it is what the pane has drawn since
// 0.1.12, so a change of copy is a change here. Provider names, plan names, window
// labels, shares and countdowns are the shell's own readings and travel through
// these words unchanged (D7).
import { describe, expect, it } from 'vitest'
import { usageCopy } from './usage-copy'

/** The English words, exactly as the pane drew them before 0.2.0. */
const ENGLISH = {
  reading: 'Reading the usage...',
  notRead: 'The usage has not been read yet.',
  missing: 'The usage sources are not installed.',
  missingWhy:
    "They are the shell's own sources: the same ones its subscription panel reads. This app never installs anything silently, so the button opens a terminal window with the install command and you decide there.",
  install: 'Install in terminal',
  installNote:
    "Install opens a terminal window and runs the shell's own installer; read what it says there.",
  checkAgain: 'Check again',
  retrySafe: 'Checking again is safe.',
  noData: 'No data',
  noDataWhy: 'A provider without credentials reports no data, never 0%.',
  limitReached: 'Limit reached',
  providerFailed: 'Could not read this provider',
  noProviders: 'The usage sources are not reporting any provider right now.',
  note: "These numbers come from the shell's own sources, the same ones its subscription panel reads: one row per window, with the share used and when it resets.",
}

/** The Spanish words: neutral Spanish, accents allowed. */
const SPANISH = {
  reading: 'Leyendo el uso...',
  notRead: 'Todavía no se ha leído el uso.',
  missing: 'Las fuentes de uso no están instaladas.',
  missingWhy:
    'Son las propias fuentes de la shell: las mismas que lee su panel de suscripción. Esta app nunca instala nada en silencio, así que el botón abre una ventana de terminal con el comando de instalación y tú decides allí.',
  install: 'Instalar en la terminal',
  installNote:
    'Instalar abre una ventana de terminal y ejecuta el instalador de la shell; lee lo que dice allí.',
  checkAgain: 'Comprobar de nuevo',
  retrySafe: 'Volver a comprobar es seguro.',
  noData: 'Sin datos',
  noDataWhy: 'Un proveedor sin credenciales informa "sin datos", nunca 0%.',
  limitReached: 'Límite alcanzado',
  providerFailed: 'No se pudo leer este proveedor',
  noProviders: 'Las fuentes de uso no informan de ningún proveedor ahora mismo.',
  note: 'Estos números vienen de las propias fuentes de la shell, las mismas que lee su panel de suscripción: una fila por ventana, con la parte usada y cuándo se reinicia.',
}

/** The plain words of one language: the two lines below carry a name. */
const WORDS = Object.keys(ENGLISH) as (keyof typeof ENGLISH)[]

/** Nothing in the pane is spelled the same in both languages. */
const SAME_IN_BOTH = new Set<keyof typeof ENGLISH>([])

/** The words of one language, as a template reads them. */
function words(language: 'en' | 'es'): Record<string, string> {
  const copy = usageCopy(language)
  return Object.fromEntries(WORDS.map((word) => [word, copy[word]]))
}

describe('the usage copy', () => {
  it('draws the English words the pane already had', () => {
    expect(words('en')).toEqual(ENGLISH)
  })

  it('draws Spanish for the same shape', () => {
    expect(words('es')).toEqual(SPANISH)
  })

  it('translates every word: nothing is left in English', () => {
    const english = usageCopy('en')
    const spanish = usageCopy('es')

    expect(Object.keys(spanish).sort()).toEqual(Object.keys(english).sort())
    for (const word of WORDS) {
      if (SAME_IN_BOTH.has(word)) {
        expect(spanish[word], word).toBe(english[word])
        continue
      }
      expect(spanish[word], word).not.toBe(english[word])
    }
  })

  it('wraps the shell own reason after the colon, word for word in both languages', () => {
    const reason = 'permission denied'

    expect(usageCopy('en').cannotRead(reason)).toBe(`Could not read the usage: ${reason}`)
    expect(usageCopy('es').cannotRead(reason)).toBe(`No se pudo leer el uso: ${reason}`)
    expect(usageCopy('en').cannotRead(reason)).toContain(reason)
    expect(usageCopy('es').cannotRead(reason)).toContain(reason)
  })

  it('names a plan with the shell own plan name, word for word in both languages', () => {
    // `Plan` is the same word in Spanish, so this line reads the same twice: the
    // plan it carries is the shell's and is never touched (D7).
    expect(usageCopy('en').plan('individual goat')).toBe('Plan: individual goat')
    expect(usageCopy('es').plan('individual goat')).toBe('Plan: individual goat')
  })
})
