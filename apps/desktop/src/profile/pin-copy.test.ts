// The pin editor's own words (profiles-editor T6, S6): one dictionary per language
// and a builder that answers the whole object (D4), spread into the tab's
// `ProfileCopy` so a component names one object and a language change redraws the
// whole tab in one tick. The English column is short so Spanish fits; the Spanish
// one is the app's own neutral Spanish - no voseo, no regionalisms - and keeps its
// accents.
//
// What does NOT live here: the layer names (the store's own keys, labelled in
// `copy.ts`), the profile names and the marks.
import { describe, expect, it } from 'vitest'
import { PIN_ENGLISH, PIN_SPANISH, pinCopy, pinProblemText } from './pin-copy'

/** Every line of one language, read the way the editor reads it. */
function words(language: 'en' | 'es') {
  const copy = pinCopy(language)
  return {
    pinPick: copy.pinPick('Repo'),
    pinSaveLayer: copy.pinSaveLayer('Repo'),
    pinNoPin: copy.pinNoPin,
    pinUnreported: copy.pinUnreported,
    pinNotSaved: copy.pinNotSaved,
    pinNote: copy.pinNote,
    pinUnknownProfile: copy.pinUnknownProfile('gone'),
  }
}

describe('the pin editor copy', () => {
  it('draws the English words the editor draws', () => {
    expect(words('en')).toEqual({
      pinPick: 'Repo pin',
      pinSaveLayer: 'Save Repo',
      pinNoPin: 'no pin',
      pinUnreported: 'current pin, not shown',
      pinNotSaved: 'The pin could not be saved. Nothing was written.',
      pinNote:
        'Saved in this worktree: its chats use the pinned profile. The repo layer is committable; the local one is shared by the whole clone.',
      pinUnknownProfile: 'The profile gone is not defined here.',
    })
  })

  it('translates every word and every line: nothing is left in English', () => {
    const english = words('en')
    const spanish = words('es')

    expect(Object.keys(spanish).sort()).toEqual(Object.keys(english).sort())
    for (const key of Object.keys(english) as (keyof typeof english)[]) {
      expect(spanish[key], key).not.toBe(english[key])
    }
    // The same shape in both dictionaries, so a language can never lose a word.
    expect(Object.keys(PIN_SPANISH).sort()).toEqual(Object.keys(PIN_ENGLISH).sort())
  })

  it('says the pin could not be saved without claiming anything was written', () => {
    for (const language of ['en', 'es'] as const) {
      const text = pinCopy(language).pinNotSaved.toLowerCase()
      expect(text).toMatch(/nothing was written|no se escribió nada/)
    }
  })

  it('names the profile a choice is about, in one line', () => {
    expect(pinProblemText(pinCopy('en'), { kind: 'unknown-profile', profile: 'gone' })).toBe(
      'The profile gone is not defined here.',
    )
    expect(pinProblemText(pinCopy('es'), { kind: 'unknown-profile', profile: 'gone' })).toBe(
      'El perfil gone no está definido aquí.',
    )
  })
})
