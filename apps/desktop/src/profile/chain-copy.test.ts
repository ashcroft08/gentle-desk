// The fallback editor's own words (profiles-editor T5, S6): one dictionary per
// language, the whole group at once (D4), and every parametrized line taking its
// argument. The English column is kept short so Spanish fits, and the Spanish one
// is the app's own neutral Spanish - no voseo, no regionalisms, nothing left in
// English. The model names, the agent keys and the efforts are NOT here: they are
// the store's data and are drawn exactly as they arrive (D7).
import { describe, expect, it } from 'vitest'
import { CHAIN_ENGLISH, CHAIN_SPANISH, chainCopy } from './chain-copy'
import { profileCopy } from './copy'

/** Every word of one language, read the way the editor reads it. */
function words(language: 'en' | 'es'): Record<string, string> {
  const copy = chainCopy(language)
  return {
    chainSave: copy.chainSave,
    chainCancel: copy.chainCancel,
    chainAddStep: copy.chainAddStep,
    chainRemoveStep: copy.chainRemoveStep(2),
    chainMoveUp: copy.chainMoveUp(2),
    chainMoveDown: copy.chainMoveDown(2),
    chainModel: copy.chainModel(2),
    chainThinking: copy.chainThinking(2),
    chainThinkingHint: copy.chainThinkingHint,
    chainNotSaved: copy.chainNotSaved,
    chainNote: copy.chainNote('team'),
    chainEmpty: copy.chainEmpty,
    chainTooLong: copy.chainTooLong,
    chainMissingModel: copy.chainMissingModel(2),
    chainRepeatedModel: copy.chainRepeatedModel('openai/gpt-5'),
  }
}

/** The English words, exactly as the editor draws them. */
const ENGLISH = {
  chainSave: 'Save',
  chainCancel: 'Cancel',
  chainAddStep: 'Add step',
  chainRemoveStep: 'Remove step 2',
  chainMoveUp: 'Move step 2 up',
  chainMoveDown: 'Move step 2 down',
  chainModel: 'Model 2',
  chainThinking: 'Thinking 2',
  chainThinkingHint: 'none',
  chainNotSaved: 'The chain could not be saved. Nothing was written.',
  chainNote: 'Saved in the profile "team", the chains every chat with that profile uses.',
  chainEmpty: 'A chain needs at least one step.',
  chainTooLong: 'A chain holds at most 8 steps.',
  chainMissingModel: 'Step 2 needs a model.',
  chainRepeatedModel: 'The model openai/gpt-5 appears twice in this chain.',
}

/** The same words in the app's neutral Spanish. */
const SPANISH = {
  chainSave: 'Guardar',
  chainCancel: 'Cancelar',
  chainAddStep: 'Añadir paso',
  chainRemoveStep: 'Quitar el paso 2',
  chainMoveUp: 'Subir el paso 2',
  chainMoveDown: 'Bajar el paso 2',
  chainModel: 'Modelo 2',
  chainThinking: 'Razonamiento 2',
  chainThinkingHint: 'ninguno',
  chainNotSaved: 'No se pudo guardar la cadena. No se escribió nada.',
  chainNote: 'Se guarda en el perfil "team", las cadenas que usan todos los chats con ese perfil.',
  chainEmpty: 'Una cadena necesita al menos un paso.',
  chainTooLong: 'Una cadena tiene como máximo 8 pasos.',
  chainMissingModel: 'El paso 2 necesita un modelo.',
  chainRepeatedModel: 'El modelo openai/gpt-5 aparece dos veces en esta cadena.',
}

describe('the chain editor copy', () => {
  it('draws the English words the editor draws', () => {
    expect(words('en')).toEqual(ENGLISH)
  })

  it('draws Spanish for the same shape', () => {
    expect(words('es')).toEqual(SPANISH)
  })

  it('translates every word: nothing is left in English', () => {
    const english = words('en')
    const spanish = words('es')

    expect(Object.keys(spanish).sort()).toEqual(Object.keys(english).sort())
    for (const key of Object.keys(english)) {
      expect(spanish[key], key).not.toBe(english[key])
    }
  })

  it('is the same group the tab spreads into its own dictionary', () => {
    // One object for the tab (D4): the spread must carry these exact words.
    expect(profileCopy('en').chainSave).toBe(CHAIN_ENGLISH.chainSave)
    expect(profileCopy('es').chainCancel).toBe(CHAIN_SPANISH.chainCancel)
    expect(profileCopy('es').chainNote('team')).toBe(SPANISH.chainNote)
  })
})
