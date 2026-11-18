// The chat profile view fixture (profiles-editor T2c): the whole
// `ChatProfileView` Rust pushes, in the shape `contract/bindings.ts` declares, so
// a test overrides only the field it is about. Every profile here is a small one
// with a real-looking route: the drawing shows models, efforts and tiers, and an
// empty string would test nothing.
import type {
  ChatProfileAgentView_Serialize as Agent,
  ChatProfileChainView_Serialize as Chain,
  ChatProfileDefinitionView_Serialize as Definition,
  ChatProfilePinLayerView_Serialize as PinLayer,
  ChatProfilePinsView_Serialize as Pins,
  ChatProfileStepView_Serialize as Step,
  ChatProfileView_Serialize as ChatProfile,
} from '../contract/bindings'

/** One agent row of a profile: the store's own key and the route it stores. */
export function agent(
  name: string,
  model: string | null,
  thinking: string | null = null,
  tier: string | null = null,
): Agent {
  return { agent: name, model, thinking, tier }
}

/** One fallback chain: the agent it is for (`*` is the profile-wide one). */
export function chain(
  name: string,
  steps: { model: string; thinking?: string | null }[],
): Chain {
  const rows: Step[] = steps.map((step) => ({ model: step.model, thinking: step.thinking ?? null }))
  return { agent: name, steps: rows }
}

/**
 * One pin layer of a worktree (profiles-editor T6): the file, the state of that
 * file, the pin the read names for it (`outranked` for a valid one another layer
 * hides) and whether it may be written. Every default is the case a test is not
 * about: a readable file, no mark, a write target offered.
 */
export function pinLayer(over: Partial<PinLayer> & { source: string }): PinLayer {
  return { status: 'ok', profile: null, mark: null, outranked: false, writable: true, ...over }
}

/** The pin group of a worktree: the repo layer holds `profile`, the local one is
 * empty, and both may be written. */
export function pins(profile: string, over: Partial<Pins> = {}): Pins {
  const winner = pinLayer({ source: 'repo', profile, mark: 'winner' })
  return {
    resolved: true,
    winner,
    layers: [winner, pinLayer({ source: 'local', status: 'missing' })],
    reason: null,
    worktree_unresolved: false,
    ...over,
  }
}

/** One profile definition, with a minimal route a test can replace. */
export function definition(name: string, over: Partial<Definition> = {}): Definition {
  return {
    name,
    agents: [agent('orchestrator', 'anthropic/claude-sonnet-4', 'high')],
    chains: [chain('*', [{ model: 'anthropic/claude-haiku-4-5', thinking: 'low' }])],
    // The store's absent `workflow` entry reads as gentle-pi's own default.
    mode: 'organic',
    ...over,
  }
}

/** The profile a chat follows when it binds nothing. */
export const TEAM: Definition = definition('team', {
  agents: [
    agent('orchestrator', 'anthropic/claude-sonnet-4', 'high'),
    agent('explore', 'anthropic/claude-haiku-4-5', 'low'),
    agent('worker', 'openai/gpt-5-codex', 'high', 'standard'),
  ],
  chains: [
    chain('*', [
      { model: 'anthropic/claude-haiku-4-5', thinking: 'low' },
      { model: 'openai/gpt-5-codex', thinking: 'medium' },
    ]),
  ],
})

/** The second profile, the one a chat can bind to itself. */
export const FAST: Definition = definition('fast', {
  agents: [agent('worker', 'openai/gpt-5-mini', 'low')],
  chains: [chain('worker', [{ model: 'openai/gpt-5-mini', thinking: 'low' }])],
})

/** A view of one chat as T2b-2 pushes it, with every store read and no binding. */
export function chatProfile(over: Partial<ChatProfile> = {}): ChatProfile {
  return {
    chat_id: 'foreground_chat',
    bound: null,
    bound_from_session_file: false,
    bound_origin: null,
    default_profile: 'team',
    profiles: [TEAM, FAST],
    // This worktree pins `fast` in its repo layer and leaves the local one empty;
    // both layers offer a write target, so both are drawn as editors (T6).
    pins: pins('fast'),
    stores: { profiles: 'read', fallbacks: 'read', pins: 'read' },
    // Nothing is being saved and nothing failed: the switch and the chain editor
    // are drawn enabled and the tab says nothing about a save (T3-mode, T5).
    saving: false,
    save_failed: false,
    chain_failed: false,
    pin_failed: false,
    // Nor is anything being applied, nothing failed to apply, no answer of the
    // child's is on screen, and no dialog is waiting for the human
    // (profiles-editor T3b).
    applying: false,
    apply_notice: null,
    apply_answer: null,
    apply_confirmation: null,
    ...over,
  }
}
