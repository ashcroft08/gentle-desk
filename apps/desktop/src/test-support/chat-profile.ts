// The chat profile view fixture (profiles-editor T2c): the whole
// `ChatProfileView` Rust pushes, in the shape `contract/bindings.ts` declares, so
// a test overrides only the field it is about. Every profile here is a small one
// with a real-looking route: the drawing shows models, efforts and tiers, and an
// empty string would test nothing.
import type {
  ChatProfileAgentView_Serialize as Agent,
  ChatProfileChainView_Serialize as Chain,
  ChatProfileDefinitionView_Serialize as Definition,
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
    pins: {
      resolved: true,
      winner: { source: 'repo', profile: 'fast', mark: 'winner' },
      layers: [{ source: 'repo', profile: 'fast', mark: 'winner' }],
      reason: null,
      worktree_unresolved: false,
    },
    stores: { profiles: 'read', fallbacks: 'read', pins: 'read' },
    // Nothing is being saved and nothing failed: the switch and the chain editor
    // are drawn enabled and the tab says nothing about a save (T3-mode, T5).
    saving: false,
    save_failed: false,
    chain_failed: false,
    ...over,
  }
}
