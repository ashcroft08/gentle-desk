// The pin editor's own words (profiles-editor T6, S6). They live apart from the
// tab's dictionary for the same reason the chain editor's do (`chain-copy.ts`):
// one file per editor, spread into the one `ProfileCopy` the tab draws (D4), so a
// component still names one object and a language change still redraws the whole
// tab in one tick.
//
// What does NOT live here either: the layer names (they are the store's own keys
// and `copy.ts` already labels them), the profile names, and the marks.
import type { LanguageView } from '../contract/bindings'
import type { PinProblem } from './pin-draft'

/** The pin editor's words, in one language. */
export interface PinCopy {
  /** The picker of one layer, by the layer's own label. */
  pinPick: (layer: string) => string
  /** The one action of one layer, by the layer's own label. */
  pinSaveLayer: (layer: string) => string
  /** The picker's own choice for `null`: the layer holds no pin. */
  pinNoPin: string
  /** The choice a layer whose pin the store does not REPORT starts on: it is
   * shown so the picker never claims the layer has no pin, and it is not one the
   * user can choose. */
  pinUnreported: string
  /** The one short line a pin write that did not happen leaves, drawn once. */
  pinNotSaved: string
  /** The line under the editors: the pin lives in the WORKTREE's own files, and
   * which of the two layers is shared by the whole clone. */
  pinNote: string
  /** Why one choice is not a pin the owner would keep: the name is not defined. */
  pinUnknownProfile: (profile: string) => string
}

/** English: short, and the same voice as the rest of the tab. */
export const PIN_ENGLISH: PinCopy = {
  pinPick: (layer) => `${layer} pin`,
  pinSaveLayer: (layer) => `Save ${layer}`,
  pinNoPin: 'no pin',
  pinUnreported: 'current pin, not shown',
  pinNotSaved: 'The pin could not be saved. Nothing was written.',
  pinNote:
    'Saved in this worktree: its chats use the pinned profile. The repo layer is committable; the local one is shared by the whole clone.',
  pinUnknownProfile: (profile) => `The profile ${profile} is not defined here.`,
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
export const PIN_SPANISH: PinCopy = {
  pinPick: (layer) => `Perfil fijado en ${layer}`,
  pinSaveLayer: (layer) => `Guardar ${layer}`,
  pinNoPin: 'sin perfil',
  pinUnreported: 'perfil actual, no se muestra',
  pinNotSaved: 'No se pudo guardar el perfil fijado. No se escribió nada.',
  pinNote:
    'Se guarda en este espacio de trabajo: sus chats usan el perfil fijado. La capa del repositorio se puede confirmar; la local la comparte todo el clon.',
  pinUnknownProfile: (profile) => `El perfil ${profile} no está definido aquí.`,
}

/**
 * What the editor says about a choice that breaks a rule, in the tab's own
 * words: one line, and the profile it is about.
 */
export function pinProblemText(copy: PinCopy, of: PinProblem): string {
  switch (of.kind) {
    case 'unknown-profile':
      return copy.pinUnknownProfile(of.profile)
  }
}

/** The editor's words in `language`, the whole group at once (D4). */
export function pinCopy(language: LanguageView): PinCopy {
  return language === 'es' ? PIN_SPANISH : PIN_ENGLISH
}
