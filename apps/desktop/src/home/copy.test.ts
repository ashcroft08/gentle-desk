// The welcome's own words (welcome-screen T2, S2, S3): the title, the rose's
// accessible name, the line that says the workspace holds no chat yet, the two
// sentences that name a workspace, and the two lines of the unavailable folder.
// One dictionary per language and a builder, as the neighbouring areas do.
//
// The English column is the approved artboard's (design/welcome-screen/
// 01-resume.html, sheets A and A2); the Spanish one is neutral - no voseo, no
// regionalisms - and keeps its accents. The folder's own reason is NOT here: it
// arrives from Rust already worded and is printed verbatim.
import { describe, expect, it } from 'vitest'
import { homeCopy } from './copy'

/** The English words, as sheet A draws them. */
const ENGLISH = {
  title: 'Pick up where you left off',
  roseLabel: 'The gentle-shell rose',
  noChats: 'no chats yet',
  othersTitle: 'Other workspaces:',
  pickAnother: 'Pick another workspace in the sidebar.',
}

/** Neutral Spanish: no voseo, no regionalisms, accents kept. */
const SPANISH: typeof ENGLISH = {
  title: 'Sigue donde lo dejaste',
  roseLabel: 'La rosa de gentle shell',
  noChats: 'sin chats todavía',
  othersTitle: 'Otros espacios:',
  pickAnother: 'Elige otro espacio en la barra lateral.',
}

/** The plain words of one language. */
const WORDS = Object.keys(ENGLISH) as (keyof typeof ENGLISH)[]

/** The words of one language, as a template reads them. */
function words(language: 'en' | 'es'): Record<string, string> {
  const copy = homeCopy(language)
  return Object.fromEntries(WORDS.map((word) => [word, copy[word]]))
}

describe('the welcome copy', () => {
  it('draws the English words sheet A carries', () => {
    expect(words('en')).toEqual(ENGLISH)
  })

  it('draws Spanish for the same shape', () => {
    expect(words('es')).toEqual(SPANISH)
  })

  it('translates every word: nothing of the screen is left in English', () => {
    for (const word of WORDS) expect(SPANISH[word], word).not.toBe(ENGLISH[word])
  })

  it('names the workspace in the one action and in the recents heading', () => {
    expect(homeCopy('en').newSession('Curriculum')).toBe('New session in Curriculum')
    expect(homeCopy('en').recentsTitle('Curriculum')).toBe('Recent chats in Curriculum')
    expect(homeCopy('es').newSession('Curriculum')).toBe('Nueva sesión en Curriculum')
    expect(homeCopy('es').recentsTitle('Curriculum')).toBe('Chats recientes de Curriculum')
  })

  it('words the age the sidebar gave it, in the order each language uses', () => {
    expect(homeCopy('en').lastActivity('21m')).toBe('· last activity 21m ago')
    expect(homeCopy('es').lastActivity('21m')).toBe('· última actividad hace 21m')
  })
})
