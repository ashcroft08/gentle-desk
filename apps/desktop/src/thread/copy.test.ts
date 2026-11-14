// The thread's own words (0.2.0, S3 and D4): the scroll region's name, the two
// author lines a screen reader hears, Copy, the live status, the model-error
// title and the two activity lines live in ONE language-keyed dictionary. This
// file pins the two columns; the words a DRAWN thread shows are pinned in
// `copy.flip.test.ts`.
//
// The English column is quoted literally - it is what the thread has drawn since
// the iced view - so a change of copy is a change here. A message's text and the
// provider's error text are not words of this module: they travel through it
// exactly as they arrived (D7).
import { describe, expect, it } from 'vitest'
import { threadCopy } from './copy'

/** The English words, exactly as the thread drew them before 0.2.0. */
const ENGLISH = {
  threadLabel: 'Chat thread',
  youSaid: 'You said',
  assistantSaid: 'Assistant said',
  copy: 'Copy',
  replying: 'Assistant is replying',
  modelErrorTitle: 'Model error',
  opening: 'Opening chat',
  working: 'Working',
}

/** The Spanish words: neutral Spanish, no voseo, no regionalisms, accents kept. */
const SPANISH = {
  threadLabel: 'Hilo del chat',
  youSaid: 'Tú dijiste',
  assistantSaid: 'El asistente dijo',
  copy: 'Copiar',
  replying: 'El asistente está respondiendo',
  modelErrorTitle: 'Error del modelo',
  opening: 'Abriendo el chat',
  working: 'Trabajando',
}

/** The plain words of one language. */
const WORDS = Object.keys(ENGLISH) as (keyof typeof ENGLISH)[]

/** Nothing in the thread is spelled the same in both languages. */
const SAME_IN_BOTH = new Set<keyof typeof ENGLISH>([])

/** The words of one language, as a template reads them. */
function words(language: 'en' | 'es'): Record<string, string> {
  const copy = threadCopy(language)
  return Object.fromEntries(WORDS.map((word) => [word, copy[word]]))
}

describe('the thread copy', () => {
  it('draws the English words the thread already had', () => {
    expect(words('en')).toEqual(ENGLISH)
  })

  it('draws Spanish for the same shape', () => {
    expect(words('es')).toEqual(SPANISH)
  })

  it('translates every word: nothing is left in English', () => {
    const english = threadCopy('en')
    const spanish = threadCopy('es')

    expect(Object.keys(spanish).sort()).toEqual(Object.keys(english).sort())
    for (const word of WORDS) {
      if (SAME_IN_BOTH.has(word)) {
        expect(spanish[word], word).toBe(english[word])
        continue
      }
      expect(spanish[word], word).not.toBe(english[word])
    }
  })
})
