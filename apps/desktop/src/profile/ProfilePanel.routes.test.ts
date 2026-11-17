// What the profile this chat RUNS is set to (profiles-editor T2c-2, S9, L19-L22):
// the Orchestrator pair and the Fallback pair, exactly the two Settings > Profiles
// draws for its active profile, and the profile's own orchestration mode. The mode
// is the PROFILE's - `workflow.<profile>.orchestratorMode`, absent = organic - so
// every chat using that profile carries it, and no switch is drawn here: the writing
// switch is T3. One test per rule, EN and ES.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen, within } from '@testing-library/svelte'
import { tick } from 'svelte'
import type { ChatProfileView_Serialize as ChatProfile } from '../contract/bindings'
import { setLanguage } from '../i18n/language.svelte'
import { agent, chain, chatProfile, definition, FAST, TEAM } from '../test-support/chat-profile'
import { resetTauriMock, setHandler } from '../test-support/tauri-mock'
import ProfilePanel from './ProfilePanel.svelte'
import { profileCopy } from './copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The tab's own words in English, read through the dictionary (D4). */
const copy = profileCopy('en')

afterEach(cleanup)
beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => null)
  setLanguage('en')
})

/** Draws the tab as the shell draws it: the open tab of one chat. */
function draw(over: { chatId?: string | null; view?: ChatProfile | null; active?: boolean } = {}) {
  return render(ProfilePanel, {
    props: { chatId: 'foreground_chat', view: chatProfile(), active: true, ...over },
  })
}

describe("the used profile's routes and its own mode", () => {
  /** A profile that names both pairs, and a mode of its own. */
  const BOTH = definition('both', {
    agents: [agent('orchestrator', 'prov/big', 'medium')],
    chains: [
      chain('*', [
        { model: 'prov/fb-one', thinking: 'low' },
        { model: 'prov/fb-two', thinking: 'low' },
      ]),
    ],
    mode: 'full-delegation',
  })

  /** The block the two pairs are drawn in, so a value drawn twice is unambiguous. */
  function routes(): HTMLElement {
    return screen.getByRole('region', { name: copy.routesLabel })
  }

  it('draws the Orchestrator and Fallback pairs of the profile this chat uses', () => {
    draw({ view: chatProfile({ profiles: [BOTH], default_profile: 'both' }) })

    const block = routes()
    expect(within(block).getByText(copy.orchestratorLabel)).toBeTruthy()
    expect(within(block).getByText('prov/big')).toBeTruthy()
    expect(within(block).getByText('medium')).toBeTruthy()
    // The fallback pair is the chain Settings draws: models in order, one effort.
    expect(within(block).getByText(copy.fallbackLabel)).toBeTruthy()
    expect(within(block).getByText('prov/fb-one \u2192 prov/fb-two')).toBeTruthy()
    expect(within(block).getByText('low')).toBeTruthy()
  })

  it('draws the mode the profile stores, labelled as the profile\u2019s own', () => {
    draw({ view: chatProfile({ profiles: [BOTH], default_profile: 'both' }) })

    const block = routes()
    expect(within(block).getByText(copy.modeLabel)).toBeTruthy()
    expect(within(block).getByText(copy.modeFullDelegation)).toBeTruthy()
    // The scope line: every chat using this profile, not this chat alone (L20).
    expect(within(block).getByText(copy.modeScope)).toBeTruthy()
    // Read-only: the switch is T3 and is not drawn here, not even disabled.
    expect(within(block).queryByRole('radiogroup')).toBeNull()
  })

  it('draws organic for a profile whose workflow names no mode at all', () => {
    draw({ view: chatProfile({ profiles: [TEAM], default_profile: 'team' }) })

    expect(within(routes()).getByText(copy.modeOrganic)).toBeTruthy()
  })

  it("draws the pairs of the profile this chat BOUND, never the default's", () => {
    draw({
      view: chatProfile({
        profiles: [TEAM, BOTH],
        default_profile: 'team',
        bound: 'both',
        bound_from_session_file: true,
      }),
    })

    const block = routes()
    expect(within(block).getByText('prov/big')).toBeTruthy()
    expect(within(block).getByText(copy.modeFullDelegation)).toBeTruthy()
    // The default profile's own route is nowhere: the tab describes the used one.
    expect(screen.queryByText('anthropic/claude-haiku-4-5')).toBeNull()
  })

  it('draws no Orchestrator row for a profile that names no orchestrator agent', () => {
    draw({ view: chatProfile({ profiles: [FAST], default_profile: 'fast' }) })

    const block = routes()
    expect(within(block).queryByText(copy.orchestratorLabel)).toBeNull()
    expect(within(block).queryByText(copy.fallbackLabel)).toBeNull()
    // The mode needs neither of them: it is the profile's, and it is drawn.
    expect(within(block).getByText(copy.modeLabel)).toBeTruthy()
  })

  it('draws no Fallback row for a profile that records no chain', () => {
    const solo = definition('solo', {
      agents: [agent('orchestrator', 'prov/solo', 'high')],
      chains: [],
    })
    draw({ view: chatProfile({ profiles: [solo], default_profile: 'solo' }) })

    const block = routes()
    expect(within(block).queryByText(copy.orchestratorLabel)).toBeTruthy()
    expect(within(block).queryByText(copy.fallbackLabel)).toBeNull()
  })

  it('draws no block at all when the store defines no profile to describe', () => {
    draw({ view: chatProfile({ profiles: [], default_profile: null, bound: 'gone' }) })

    expect(screen.queryByRole('region')).toBeNull()
  })

  it('draws every word in Spanish, the two pairs included', async () => {
    const spanish = profileCopy('es')
    draw({ view: chatProfile({ profiles: [BOTH], default_profile: 'both' }) })

    setLanguage('es')
    await tick()

    const block = screen.getByRole('region', { name: spanish.routesLabel })
    expect(within(block).getByText(spanish.orchestratorLabel)).toBeTruthy()
    // `Fallback` stays English in Spanish, as the Settings pane draws it.
    expect(spanish.fallbackLabel).toBe('Fallback')
    expect(within(block).getByText(spanish.fallbackLabel)).toBeTruthy()
    expect(within(block).getByText(spanish.modeLabel)).toBeTruthy()
    expect(within(block).getByText(spanish.modeFullDelegation)).toBeTruthy()
    expect(within(block).getByText(spanish.modeScope)).toBeTruthy()
    expect(within(block).queryByText(copy.modeLabel)).toBeNull()
  })
})
