// The update line's own words (0.2.0, S3 and D4): the four states the line can
// speak and its three buttons live in ONE language-keyed dictionary. This file
// pins the two columns; the words a DRAWN line shows are pinned in
// `copy.flip.test.ts`.
//
// The English column is quoted literally - it is what the line has drawn since
// 0.1.9 - so a change of copy is a change here. The VERSION the updater reports
// and its own failure message are not words of this module: both arrive exactly
// as they were written (D7).
import { describe, expect, it } from 'vitest'
import { updateCopy } from './copy'

/** The English words, exactly as the line drew them before 0.2.0. */
const ENGLISH = {
  noNewerVersion: 'no newer version was found',
  install: 'Install update',
  retry: 'Try again',
  dismiss: 'Not now',
}

/** The Spanish words: neutral Spanish, no voseo, no regionalisms, accents kept. */
const SPANISH = {
  noNewerVersion: 'no se encontró una versión más nueva',
  install: 'Instalar actualización',
  retry: 'Reintentar',
  dismiss: 'Ahora no',
}

/** The plain words of one language. */
const WORDS = Object.keys(ENGLISH) as (keyof typeof ENGLISH)[]

/** Nothing in the line is spelled the same in both languages. */
const SAME_IN_BOTH = new Set<keyof typeof ENGLISH>([])

/** The words of one language, as a template reads them. */
function words(language: 'en' | 'es'): Record<string, string> {
  const copy = updateCopy(language)
  return Object.fromEntries(WORDS.map((word) => [word, copy[word]]))
}

describe('the update copy', () => {
  it('draws the English words the line already had', () => {
    expect(words('en')).toEqual(ENGLISH)
  })

  it('draws Spanish for the same shape', () => {
    expect(words('es')).toEqual(SPANISH)
  })

  it('translates every word: nothing is left in English', () => {
    const english = updateCopy('en')
    const spanish = updateCopy('es')

    expect(Object.keys(spanish).sort()).toEqual(Object.keys(english).sort())
    for (const word of WORDS) {
      if (SAME_IN_BOTH.has(word)) {
        expect(spanish[word], word).toBe(english[word])
        continue
      }
      expect(spanish[word], word).not.toBe(english[word])
    }
  })

  it('speaks the four states and wraps the version and the failure untouched', () => {
    const message = 'signature check failed'

    expect(updateCopy('en').found('0.2.0')).toBe(
      'Version 0.2.0 is available. The app will close while it installs.',
    )
    expect(updateCopy('en').downloading('0.2.0', 40)).toBe('Downloading version 0.2.0: 40%.')
    expect(updateCopy('en').downloading('0.2.0', null)).toBe('Downloading version 0.2.0.')
    expect(updateCopy('en').ready('0.2.0')).toBe(
      'Version 0.2.0 is installed. Open the app again to use it.',
    )

    expect(updateCopy('es').found('0.2.0')).toBe(
      'La versión 0.2.0 está disponible. La app se cerrará mientras se instala.',
    )
    expect(updateCopy('es').downloading('0.2.0', 40)).toBe('Descargando la versión 0.2.0: 40%.')
    expect(updateCopy('es').downloading('0.2.0', null)).toBe('Descargando la versión 0.2.0.')
    expect(updateCopy('es').ready('0.2.0')).toBe(
      'La versión 0.2.0 está instalada. Abre la app de nuevo para usarla.',
    )

    // D7: the version and the updater's own message are carried, not translated.
    expect(updateCopy('es').failed('0.2.0', message)).toBe(
      `No se pudo instalar la actualización a la versión 0.2.0: ${message}`,
    )
    expect(updateCopy('es').failed(null, message)).toBe(
      `No se pudo instalar la actualización: ${message}`,
    )
  })
})
