// The fallback-chain editor's own words (profiles-editor T5, S6, L30-3). They
// live apart from the tab's dictionary only because the tab's file reached the
// house lint's 200 lines; `copy.ts` spreads them into the one `ProfileCopy` the
// tab draws (D4, the same whole-object builder), so a component still names one
// object and a language change still redraws the whole tab in one tick.
//
// What does NOT live here either: the model names, the agent keys and the efforts.
// They are the store's data and are drawn exactly as they arrive (D7).
import type { LanguageView } from '../contract/bindings'
import type { ChainProblem } from './chain-draft'

/** One chain editor's words, in one language. */
export interface ChainCopy {
  /** The two actions of one chain, and the one that adds a step. */
  chainSave: string
  chainCancel: string
  chainAddStep: string
  /** The actions of one step, by its 1-based number. */
  chainRemoveStep: (index: number) => string
  chainMoveUp: (index: number) => string
  chainMoveDown: (index: number) => string
  /** The two fields of one step, by its 1-based number. */
  chainModel: (index: number) => string
  chainThinking: (index: number) => string
  /** What an empty effort input means: no effort at all. */
  chainThinkingHint: string
  /** The one short line a chain write that did not happen leaves, drawn once. */
  chainNotSaved: string
  /** The line under the group: the chains live in the PROFILE, never in a chat. */
  chainNote: (name: string) => string
  /** The owner's own rules, in the tab's words, for a draft that breaks one. */
  chainEmpty: string
  chainTooLong: string
  chainMissingModel: (index: number) => string
  chainRepeatedModel: (model: string) => string
}

/** English: short, and the same words the rest of the tab uses for a save. */
export const CHAIN_ENGLISH: ChainCopy = {
  chainSave: 'Save',
  chainCancel: 'Cancel',
  chainAddStep: 'Add step',
  chainRemoveStep: (index) => `Remove step ${index}`,
  chainMoveUp: (index) => `Move step ${index} up`,
  chainMoveDown: (index) => `Move step ${index} down`,
  chainModel: (index) => `Model ${index}`,
  chainThinking: (index) => `Thinking ${index}`,
  chainThinkingHint: 'none',
  chainNotSaved: 'The chain could not be saved. Nothing was written.',
  chainNote: (name) =>
    `Saved in the profile "${name}", the chains every chat with that profile uses.`,
  chainEmpty: 'A chain needs at least one step.',
  chainTooLong: 'A chain holds at most 8 steps.',
  chainMissingModel: (index) => `Step ${index} needs a model.`,
  chainRepeatedModel: (model) => `The model ${model} appears twice in this chain.`,
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
export const CHAIN_SPANISH: ChainCopy = {
  chainSave: 'Guardar',
  chainCancel: 'Cancelar',
  chainAddStep: 'Añadir paso',
  chainRemoveStep: (index) => `Quitar el paso ${index}`,
  chainMoveUp: (index) => `Subir el paso ${index}`,
  chainMoveDown: (index) => `Bajar el paso ${index}`,
  chainModel: (index) => `Modelo ${index}`,
  chainThinking: (index) => `Razonamiento ${index}`,
  chainThinkingHint: 'ninguno',
  chainNotSaved: 'No se pudo guardar la cadena. No se escribió nada.',
  chainNote: (name) =>
    `Se guarda en el perfil "${name}", las cadenas que usan todos los chats con ese perfil.`,
  chainEmpty: 'Una cadena necesita al menos un paso.',
  chainTooLong: 'Una cadena tiene como máximo 8 pasos.',
  chainMissingModel: (index) => `El paso ${index} necesita un modelo.`,
  chainRepeatedModel: (model) => `El modelo ${model} aparece dos veces en esta cadena.`,
}

/**
 * What the editor says about a draft that breaks one of the owner's rules, in the
 * tab's own words: one line, and the number or the model it is about.
 */
export function problemText(copy: ChainCopy, of: ChainProblem): string {
  switch (of.kind) {
    case 'empty':
      return copy.chainEmpty
    case 'too-long':
      return copy.chainTooLong
    case 'missing-model':
      return copy.chainMissingModel(of.index)
    case 'repeated-model':
      return copy.chainRepeatedModel(of.model)
  }
}

/** The editor's words in `language`, the whole group at once (D4). */
export function chainCopy(language: LanguageView): ChainCopy {
  return language === 'es' ? CHAIN_SPANISH : CHAIN_ENGLISH
}
