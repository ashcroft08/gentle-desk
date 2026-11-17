// The Profile tab's pure rules (profiles-editor T2c): which of the four states the
// page draws, which profile this chat runs, and which store the view could not
// read. They are plain data, so a component only draws a decision and a test can
// pin the decision itself (the `chat/tabs.ts` pattern).
//
// The state is the artboard's own (06-profile-tab.html): state 1 and 2 are the
// name and the tag of the profile in use, state 5 is the pins, state 10 is a chat
// whose child did not answer - `stores` says so without one channel error, and
// "some of it was read" is a state of its own, never a failure of the whole tab.
import type {
  ChatProfileAgentView_Serialize as ProfileAgent,
  ChatProfileAvailabilityView,
  ChatProfileChainView_Serialize as ProfileChain,
  ChatProfileDefinitionView_Serialize as ProfileDefinition,
  ChatProfileView_Serialize as ChatProfile,
} from '../contract/bindings'

/**
 * The reserved agent key gentle-pi routes the orchestrator with, and the `*` key of
 * the profile-wide chain: the two names this view INTERPRETS
 * (`crates/runtime/src/profiles.rs`).
 */
export const ORCHESTRATOR_ROLE = 'orchestrator'
export const DEFAULT_CHAIN = '*'

/** The three stores a chat's profile is read from, in the order Rust asks. */
export type StoreName = 'profiles' | 'fallbacks' | 'pins'

/** The three, in the order the profile data depends on them. */
export const STORES: StoreName[] = ['profiles', 'fallbacks', 'pins']

/** What the tab draws: the four shapes the read-only slice has. */
export type PanelState = 'reading' | 'unavailable' | 'partial' | 'ready'

/** How one store answered, or `null` while there is no view at all. */
export function storeOf(
  view: ChatProfile | null,
  store: StoreName,
): ChatProfileAvailabilityView | null {
  return view === null ? null : view.stores[store]
}

/** Whether one store's content is in this view. */
export function storeRead(view: ChatProfile | null, store: StoreName): boolean {
  return storeOf(view, store) === 'read'
}

/** Every store this view could not use, in the order Rust asks them. */
export function missingStores(view: ChatProfile | null): StoreName[] {
  return STORES.filter((store) => !storeRead(view, store))
}

/**
 * The shape the tab draws: reading until a read answers, unavailable when it
 * answered nothing or no store could be read, partial when one store was read and
 * another was not, ready when all three answered. The stores decide, never the
 * count of profiles: an empty store that answered is a real answer.
 */
export function panelState(view: ChatProfile | null, asked: boolean): PanelState {
  if (view === null) return asked ? 'unavailable' : 'reading'
  const read = STORES.filter((store) => storeRead(view, store)).length
  if (read === 0) return 'unavailable'
  return read === STORES.length ? 'ready' : 'partial'
}

/**
 * The name of the profile this chat runs: the one its session file bound, else the
 * global default, else none - the artboard's states 1 and 2 in one line. A bound
 * name is drawn even when the store no longer defines it: the binding is the
 * truth about this chat, and hiding it would invent a profile.
 */
export function usedName(view: ChatProfile | null): string | null {
  if (view === null) return null
  return view.bound ?? view.default_profile
}

/** That profile's definition, or `null` when the store does not define it. */
export function usedProfile(view: ChatProfile | null): ProfileDefinition | null {
  const name = usedName(view)
  if (name === null) return null
  return view?.profiles.find((profile) => profile.name === name) ?? null
}

/**
 * The used profile's Orchestrator pair: the row stored under the reserved
 * `orchestrator` agent, or `null` for a profile that names none - the row
 * `Profile::orchestrator()` selects in `crates/runtime/src/profiles.rs`, which
 * the pane Settings used to draw as its summary line (removed in profiles-editor
 * T2d).
 */
export function orchestratorOf(profile: ProfileDefinition): ProfileAgent | null {
  return profile.agents.find((agent) => agent.agent === ORCHESTRATOR_ROLE) ?? null
}

/**
 * The chain the used profile's Fallback pair draws: the orchestrator's own, else
 * the profile-wide `*` - `Profile::chain_for(ORCHESTRATOR_ROLE)` in
 * `crates/runtime/src/profiles.rs`, the rule `Profile::fallback()` names.
 * `null` for a profile that records no chain at all: no fallback is ever invented
 * to fill the row.
 */
export function fallbackChainOf(profile: ProfileDefinition): ProfileChain | null {
  return (
    profile.chains.find((chain) => chain.agent === ORCHESTRATOR_ROLE) ??
    profile.chains.find((chain) => chain.agent === DEFAULT_CHAIN) ??
    null
  )
}

/**
 * The Fallback pair of one chain, in the shape the pair has: the models in the
 * order gentle-pi would move through them, joined with an arrow, and the effort
 * only when EVERY step asks for the same one - one effort cell cannot speak for a
 * chain that changes effort per step. It is the rule the removed Settings pane
 * drew for the same pair (L19), kept here now that the tab is the only view.
 */
export function fallbackPair(chain: ProfileChain): { model: string; thinking: string | null } | null {
  if (chain.steps.length === 0) return null
  const model = chain.steps.map((step) => step.model).join(' \u2192 ')
  const first = chain.steps[0].thinking ?? null
  const thinking = chain.steps.every((step) => (step.thinking ?? null) === first) ? first : null
  return { model, thinking }
}
