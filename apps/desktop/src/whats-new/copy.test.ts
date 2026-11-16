// The what's-new modal's own words (0.2.0, S3 and D4): its heading, its one
// answer, the keyboard hint beside it and the "since X" header live in ONE
// language-keyed dictionary. This file pins the two columns; the words a DRAWN
// modal shows are pinned in `copy.flip.test.ts`.
//
// The English column is quoted literally - it is what the modal has drawn since
// T2 - so a change of copy is a change here. The release BODY is not part of this
// dictionary: it travels in latest.json and its words are the release's, drawn
// as they arrive (D7). The version numbers are the same in both languages.
import { describe, expect, it } from 'vitest'
import { sinceLabel, whatsNewCopy } from './copy'

/** The English words, exactly as the modal drew them before 0.2.0. */
const ENGLISH = {
  heading: "What's new",
  dismiss: 'Got it',
  escHint: 'Esc to close',
}

/** The Spanish words: neutral Spanish, no voseo, no regionalisms, accents kept. */
const SPANISH = {
  heading: 'Novedades',
  dismiss: 'Entendido',
  escHint: 'Esc para cerrar',
}

/** The plain words of one language. */
const WORDS = Object.keys(ENGLISH) as (keyof typeof ENGLISH)[]

/** Nothing in the modal is spelled the same in both languages. */
const SAME_IN_BOTH = new Set<keyof typeof ENGLISH>([])

/** The words of one language, as a template reads them. */
function words(language: 'en' | 'es'): Record<string, string> {
  const copy = whatsNewCopy(language)
  return Object.fromEntries(WORDS.map((word) => [word, copy[word]]))
}

describe("the what's-new copy", () => {
  it('draws the English words the modal already had', () => {
    expect(words('en')).toEqual(ENGLISH)
  })

  it('draws Spanish for the same shape', () => {
    expect(words('es')).toEqual(SPANISH)
  })

  it('translates every word: nothing is left in English', () => {
    const english = whatsNewCopy('en')
    const spanish = whatsNewCopy('es')

    expect(Object.keys(spanish).sort()).toEqual(Object.keys(english).sort())
    for (const word of WORDS) {
      if (SAME_IN_BOTH.has(word)) {
        expect(spanish[word], word).toBe(english[word])
        continue
      }
      expect(spanish[word], word).not.toBe(english[word])
    }
  })

  it('says "since X" in the language of the modal, and keeps the version untouched', () => {
    expect(sinceLabel('0.1.11', '0.1.9', whatsNewCopy('en'))).toBe('since 0.1.9')
    expect(sinceLabel('0.1.11', '0.1.9', whatsNewCopy('es'))).toBe('desde 0.1.9')
    // The rule itself does not depend on the language: only its word does.
    expect(sinceLabel('0.1.11', '0.1.11', whatsNewCopy('es'))).toBeNull()
    expect(sinceLabel('0.1.9', '0.1.11', whatsNewCopy('es'))).toBeNull()
  })
})
