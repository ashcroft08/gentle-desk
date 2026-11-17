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
  ChatProfileAvailabilityView,
  ChatProfileDefinitionView_Serialize as ProfileDefinition,
  ChatProfileView_Serialize as ChatProfile,
} from '../contract/bindings'

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
