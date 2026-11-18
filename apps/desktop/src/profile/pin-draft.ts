// The Profile tab's PIN EDITOR rules (profiles-editor T6, S2, S3): what one
// layer's picker holds, whether that is a change, and whether a change may be
// sent. They are plain data, so a component draws a decision and a test can pin
// the decision itself (the `rules.ts` and `chain-draft.ts` pattern).
//
// Three rules come from the channel and are checked here so the app never sends
// what the owner would refuse (`desktop-rpc.md:322-372`):
//
// - a pin names a profile the profiles store DEFINES, or `null` to clear the
//   layer: an unknown name is `invalid_data`, so it is caught before Save;
// - only a layer that offers a write target may be written at all: outside a
//   resolvable Git worktree `pins` offers none, and a write would be
//   `config_unavailable`;
// - a layer that already holds this pin is not written again - and a layer whose
//   VALID pin another layer outranks is NOT reported by the store, so its picker
//   starts on a choice nobody can send and the tab never claims to know it.
import type { ChatProfilePinLayerView_Serialize as PinLayer } from '../contract/bindings'

/**
 * The picker's own value for `null`: the layer holds no pin. The channel spells
 * that clear as `profile: null`, never as an empty name.
 */
export const NO_PIN = ''

/**
 * What one layer's picker holds: the profile the layer should hold, [`NO_PIN`]
 * for "no pin", or `null` for a layer whose pin the store does not report (a
 * valid pin another layer outranks, `desktop-rpc.md:211-234`) - a state the user
 * must choose out of, because the tab cannot say what it is.
 */
export type PinChoice = string | null

/** The choice a layer's picker starts on, as far as the view says. */
export function choiceOf(layer: PinLayer): PinChoice {
  if (layer.profile !== null) return layer.profile
  return layer.outranked ? null : NO_PIN
}

/** The layer's pin from the picker's own choice: the store's `null` is `NO_PIN`. */
export function profileOf(choice: PinChoice): string | null {
  return choice === null || choice === NO_PIN ? null : choice
}

/** Whether this choice would write something the layer does not already hold. */
export function dirty(layer: PinLayer, choice: PinChoice): boolean {
  return choiceOf(layer) !== choice
}

/** Why this choice is not a pin the owner would keep, or `null` when it is. */
export type PinProblem = { kind: 'unknown-profile'; profile: string }

/**
 * The first rule a choice breaks: the channel writes a name the profiles store
 * defines, or the explicit clear. `null` - the unreported starting state - is not
 * a choice to send at all, so it breaks no rule here and [`canSavePin`] refuses it
 * on its own.
 */
export function pinProblem(choice: PinChoice, defined: string[]): PinProblem | null {
  if (choice === null || choice === NO_PIN) return null
  return defined.includes(choice) ? null : { kind: 'unknown-profile', profile: choice }
}

/**
 * The names one layer's picker offers: every profile the store defines, plus the
 * pin the layer holds when the store no longer defines it - a stale pin is drawn
 * as the layer's own state instead of being hidden behind the empty first option.
 */
export function pinOptions(layer: PinLayer, defined: string[]): string[] {
  const held = layer.profile
  if (held === null || defined.includes(held)) return defined
  return [...defined, held]
}

/**
 * Whether this layer's one Save may be used: the layer is writable here, nothing
 * is being written, the choice is a change, and it is a pin the owner would keep.
 */
export function canSavePin(
  layer: PinLayer,
  choice: PinChoice,
  defined: string[],
  saving: boolean,
): boolean {
  // `null` is a state to move OUT of, never a pin to send.
  if (choice === null) return false
  return layer.writable && !saving && dirty(layer, choice) && pinProblem(choice, defined) === null
}
