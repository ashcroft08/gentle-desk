// The Profile tab's two pure rules (profiles-editor T2c): which of the four
// states the page draws, and which profile the chat runs - the bound one, else the
// global default, else nothing at all. They are plain data so the components only
// draw a decision (the `chat/tabs.ts` pattern).
import { describe, expect, it } from 'vitest'
import { agent, chain, chatProfile, definition, FAST, TEAM } from '../test-support/chat-profile'
import {
  fallbackChainOf,
  fallbackPair,
  missingStores,
  orchestratorOf,
  panelState,
  usedName,
  usedProfile,
} from './rules'

describe('panelState', () => {
  it('is reading until the first answer lands, and nothing before that is a failure', () => {
    expect(panelState(null, false)).toBe('reading')
  })

  it("is unavailable once a read answered nothing, or when no store could be read", () => {
    expect(panelState(null, true)).toBe('unavailable')
    expect(
      panelState(chatProfile({ stores: { profiles: 'unavailable', fallbacks: 'skipped', pins: 'skipped' } }), true),
    ).toBe('unavailable')
  })

  it('is partial when one store was read and another was not', () => {
    const view = chatProfile({ stores: { profiles: 'read', fallbacks: 'read', pins: 'skipped' } })

    expect(panelState(view, true)).toBe('partial')
    expect(missingStores(view)).toEqual(['pins'])
  })

  it('is ready when all three stores answered', () => {
    expect(panelState(chatProfile(), true)).toBe('ready')
    expect(missingStores(chatProfile())).toEqual([])
  })
})

describe('the profile this chat uses', () => {
  it('is the default profile for a chat that bound nothing', () => {
    const view = chatProfile()

    expect(usedName(view)).toBe('team')
    expect(usedProfile(view)?.name).toBe('team')
  })

  it('is the bound profile, whatever the default is', () => {
    const view = chatProfile({ bound: 'fast', bound_from_session_file: true })

    expect(usedName(view)).toBe('fast')
    expect(usedProfile(view)?.name).toBe('fast')
  })

  it('is nothing at all when neither a binding nor a default is named', () => {
    const view = chatProfile({ bound: null, default_profile: null })

    expect(usedName(view)).toBeNull()
    expect(usedProfile(view)).toBeNull()
  })

  it('keeps the bound name even when the store no longer defines it', () => {
    const view = chatProfile({ bound: 'gone', bound_from_session_file: true })

    expect(usedName(view)).toBe('gone')
    expect(usedProfile(view)).toBeNull()
  })
})

describe("the used profile's Orchestrator and Fallback pairs", () => {
  it('takes the orchestrator from the reserved `orchestrator` agent, and nothing when absent', () => {
    expect(orchestratorOf(TEAM)).toEqual(agent('orchestrator', 'anthropic/claude-sonnet-4', 'high'))
    // `fast` names a worker only: no orchestrator row is invented for it.
    expect(orchestratorOf(FAST)).toBeNull()
    expect(orchestratorOf(definition('bare', { agents: [] }))).toBeNull()
  })

  it("takes the fallback from the profile's `*` chain when the orchestrator has none", () => {
    expect(fallbackChainOf(TEAM)).toEqual(TEAM.chains[0])
    expect(fallbackChainOf(TEAM)?.agent).toBe('*')
  })

  it("prefers the orchestrator's own chain, the rule `Profile::chain_for` applies", () => {
    const profile = definition('both', {
      chains: [
        chain('*', [{ model: 'wide/one' }]),
        chain('orchestrator', [{ model: 'orx/backup' }]),
      ],
    })

    expect(fallbackChainOf(profile)?.agent).toBe('orchestrator')
  })

  it('is nothing at all for a profile that records no chain', () => {
    expect(fallbackChainOf(FAST)).toBeNull()
    expect(fallbackChainOf(definition('bare', { chains: [] }))).toBeNull()
  })

  it('joins the chain models with an arrow and keeps the effort only when it is uniform', () => {
    expect(
      fallbackPair(
        chain('*', [
          { model: 'prov/fb-one', thinking: 'low' },
          { model: 'prov/fb-two', thinking: 'low' },
        ]),
      ),
    ).toEqual({ model: 'prov/fb-one \u2192 prov/fb-two', thinking: 'low' })

    expect(
      fallbackPair(
        chain('*', [
          { model: 'prov/fb-one', thinking: 'low' },
          { model: 'prov/fb-two', thinking: 'high' },
        ]),
      ),
    ).toEqual({ model: 'prov/fb-one \u2192 prov/fb-two', thinking: null })

    // A step that names no effort keeps the pair from speaking for the chain.
    expect(fallbackPair(chain('*', [{ model: 'prov/only' }]))).toEqual({
      model: 'prov/only',
      thinking: null,
    })
  })
})
