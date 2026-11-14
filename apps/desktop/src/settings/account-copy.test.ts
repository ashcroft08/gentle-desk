// The Account pane's own words (0.2.0, S3 and D4): the pane's states, its two
// buttons and one row's five check answers live in ONE language-keyed dictionary.
// This file pins the two columns; the words a DRAWN pane shows are pinned in
// `account-copy.flip.test.ts`.
//
// The English column is quoted literally: it is what the pane has drawn since
// 0.1.12, so a change of copy is a change here. Provider NAMES, the check's own
// reason and the app's name are not words of this module: the first two travel
// through it exactly as the shell wrote them (D7).
import { describe, expect, it } from 'vitest'
import { accountCopy } from './account-copy'

/** The English words, exactly as the pane drew them before 0.2.0. */
const ENGLISH = {
  reading: 'Reading the sign-ins...',
  retrySafe: 'Checking again is safe.',
  noProviders: 'No providers are signed in.',
  checkAgain: 'Check again',
  openShell: 'Open shell to sign in or out',
  note: 'The shell opens in a terminal window: type /login to sign in or /logout to sign out, then press Check again here.',
  connected: 'Connected',
  unverified: 'Signed in (not checkable)',
  notReady: 'Not ready',
  invalid: 'Sign-in is not valid',
  error: 'Could not check',
}

/** The Spanish words: neutral Spanish, accents allowed. */
const SPANISH = {
  reading: 'Leyendo los inicios de sesión...',
  retrySafe: 'Volver a comprobar es seguro.',
  noProviders: 'No hay proveedores con sesión iniciada.',
  checkAgain: 'Comprobar de nuevo',
  openShell: 'Abrir la shell para iniciar o cerrar sesión',
  note: 'La shell se abre en una ventana de terminal: escribe /login para iniciar sesión o /logout para cerrarla, y luego pulsa Comprobar de nuevo aquí.',
  connected: 'Conectado',
  unverified: 'Sesión iniciada (no verificable)',
  notReady: 'No está listo',
  invalid: 'La sesión no es válida',
  error: 'No se pudo comprobar',
}

/** The plain words of one language. */
const WORDS = Object.keys(ENGLISH) as (keyof typeof ENGLISH)[]

/** Nothing in the pane is spelled the same in both languages. */
const SAME_IN_BOTH = new Set<keyof typeof ENGLISH>([])

/** The words of one language, as a template reads them. */
function words(language: 'en' | 'es'): Record<string, string> {
  const copy = accountCopy(language)
  return Object.fromEntries(WORDS.map((word) => [word, copy[word]]))
}

describe('the account copy', () => {
  it('draws the English words the pane already had', () => {
    expect(words('en')).toEqual(ENGLISH)
  })

  it('draws Spanish for the same shape', () => {
    expect(words('es')).toEqual(SPANISH)
  })

  it('translates every word: nothing is left in English', () => {
    const english = accountCopy('en')
    const spanish = accountCopy('es')

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

    expect(accountCopy('en').failed(reason)).toBe(`Could not read the sign-ins: ${reason}`)
    expect(accountCopy('es').failed(reason)).toBe(
      `No se pudieron leer los inicios de sesión: ${reason}`,
    )
    // D7: the reason is the shell's, so both columns carry it untouched.
    expect(accountCopy('en').failed(reason)).toContain(reason)
    expect(accountCopy('es').failed(reason)).toContain(reason)
  })
})
