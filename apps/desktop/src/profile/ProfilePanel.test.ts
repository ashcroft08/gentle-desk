// The Profile tab's READ-ONLY states (profiles-editor T2c, artboard
// 06-profile-tab.html): a chat that follows the default (1), a chat bound to a
// profile of its own (2), the pins with their marks and the no-worktree case (5),
// the calm unavailable state (10), the partial read, and the tab drawing no
// control at all - editing is T3+. One test per rule.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import { tick } from 'svelte'
import type { ChatProfileView_Serialize as ChatProfile } from '../contract/bindings'
import { setLanguage } from '../i18n/language.svelte'
import { chatProfile, definition } from '../test-support/chat-profile'
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

/** The body a group head controls. */
function bodyOf(head: HTMLElement): HTMLElement {
  const id = head.getAttribute('aria-controls') ?? ''
  const body = document.getElementById(id)
  if (body === null) throw new Error('the group head controls no body')
  return body
}

/** One accordion group's head, by the name it opens with. */
function group(name: RegExp): HTMLElement {
  return screen.getByRole('button', { name })
}

describe('the profile this chat uses', () => {
  it('draws the default profile it follows, and says the answer is approximate', () => {
    draw()

    expect(screen.getByRole('heading', { level: 2, name: /team/ })).toBeTruthy()
    expect(screen.getByText('default')).toBeTruthy()
    expect(screen.getByText(copy.lineDefault('team'))).toBeTruthy()
    // L9: the binding is READ FROM THE SESSION FILE, so the tab says so.
    expect(screen.getByText(copy.approximation)).toBeTruthy()
    // The profile the chat runs opens first, and its head says what it holds.
    const team = group(/^team/)
    expect(team.getAttribute('aria-expanded')).toBe('true')
    expect(
      within(team).getByText(`${copy.agentCount(3)} \u00b7 ${copy.chainCount(1)}`),
    ).toBeTruthy()
    expect(within(bodyOf(team)).getByText('anthropic/claude-sonnet-4')).toBeTruthy()
  })

  it('draws the neutral line for a chat whose default is not named', () => {
    draw({ view: chatProfile({ default_profile: null }) })

    expect(screen.getByText(copy.lineDefaultNone)).toBeTruthy()
    // No name and no tag: nothing is invented for a profile that has no name.
    expect(screen.queryByRole('heading', { level: 2 })).toBeNull()
    expect(screen.getByText(copy.approximation)).toBeTruthy()
  })

  it('draws a profile this chat bound to itself with the this-chat-only tag', () => {
    draw({ view: chatProfile({ bound: 'fast', bound_from_session_file: true, bound_origin: 'user' }) })

    expect(screen.getByRole('heading', { level: 2, name: /fast/ })).toBeTruthy()
    expect(screen.getByText(copy.tagSession)).toBeTruthy()
    expect(screen.getByText(copy.lineSession('fast'))).toBeTruthy()
    expect(screen.getByText(copy.approximation)).toBeTruthy()
    // The global default is still named in the list, never mixed with the bound one.
    expect(group(/^team/)).toBeTruthy()
  })

  it('draws a bound profile the store no longer defines instead of hiding the binding', () => {
    draw({ view: chatProfile({ bound: 'gone', bound_from_session_file: true }) })

    expect(screen.getByRole('heading', { level: 2, name: /gone/ })).toBeTruthy()
    expect(screen.getByText(copy.tagSession)).toBeTruthy()
  })
})

describe('the profiles listed read-only', () => {
  it('draws every profile with its agents, its chains and the profile-wide chain labelled', async () => {
    draw()

    expect(group(/^team/)).toBeTruthy()
    expect(group(/^fast/)).toBeTruthy()

    const team = bodyOf(group(/^team/))
    for (const heading of [copy.agentColumn, copy.modelColumn, copy.thinkingColumn, copy.tierColumn]) {
      expect(within(team).getByText(heading)).toBeTruthy()
    }
    // The route the store holds: model, thinking and tier, as stored.
    expect(within(team).getByText('anthropic/claude-sonnet-4')).toBeTruthy()
    // The orchestrator and the worker both run at `high`, as the store says.
    expect(within(team).getAllByText('high')).toHaveLength(2)
    expect(within(team).getByText('standard')).toBeTruthy()
    // The `*` chain is the profile-wide one, and it says so.
    expect(within(team).getByText(copy.profileWide)).toBeTruthy()
    // The worker's model, and the same model as the chain's second step.
    expect(within(team).getAllByText('openai/gpt-5-codex')).toHaveLength(2)

    // One group open at a time: opening `fast` closes `team`.
    await fireEvent.click(group(/^fast/))

    expect(group(/^team/).getAttribute('aria-expanded')).toBe('false')
    expect(bodyOf(group(/^team/)).hidden).toBe(true)
    expect(group(/^fast/).getAttribute('aria-expanded')).toBe('true')
    // The fast profile's own route: its worker's model, and the same model as
    // its worker chain's only step.
    expect(within(bodyOf(group(/^fast/))).getAllByText('openai/gpt-5-mini')).toHaveLength(2)
  })

  it('opens the group of the profile this chat runs, not the first one in the store', () => {
    draw({ view: chatProfile({ bound: 'fast', bound_from_session_file: true }) })

    expect(group(/^fast/).getAttribute('aria-expanded')).toBe('true')
    expect(group(/^team/).getAttribute('aria-expanded')).toBe('false')
    expect(bodyOf(group(/^fast/)).hidden).toBe(false)
  })

  it('draws no edit control at all: this tab only reads', () => {
    draw()

    for (const name of [/^Edit$/, /^Save$/, /^Use in this chat$/, /^Make default$/, /^Clear$/]) {
      expect(screen.queryByRole('button', { name }), String(name)).toBeNull()
    }
    expect(screen.queryByRole('combobox')).toBeNull()
    expect(screen.queryByRole('textbox')).toBeNull()
    expect(screen.getByText(copy.readOnly)).toBeTruthy()
  })

  it('says so when the store defines no profile at all', () => {
    draw({ view: chatProfile({ profiles: [], default_profile: null, bound: null }) })

    expect(screen.getByText(copy.noProfiles)).toBeTruthy()
    expect(screen.queryByText(copy.readOnly)).toBeTruthy()
  })
})

describe('a chat whose profile cannot be read', () => {
  it('draws the calm unavailable state when no store answered', () => {
    draw({
      view: chatProfile({
        bound: null,
        profiles: [],
        default_profile: null,
        pins: null,
        stores: { profiles: 'unavailable', fallbacks: 'skipped', pins: 'skipped' },
      }),
    })

    expect(screen.getByText(copy.unavailable)).toBeTruthy()
    expect(screen.getByText(copy.unavailableWhy)).toBeTruthy()
    expect(screen.getByRole('button', { name: copy.tryAgain })).toBeTruthy()
    // Nothing read is drawn, and nothing is an error.
    expect(screen.queryByText('team')).toBeNull()
    expect(screen.queryByRole('alert')).toBeNull()
  })

  it('draws what it read and marks the store that did not answer', async () => {
    draw({
      view: chatProfile({ stores: { profiles: 'read', fallbacks: 'skipped', pins: 'unavailable' } }),
    })

    // The profiles it read are drawn, one group per profile.
    expect(group(/^team/)).toBeTruthy()
    // The store that did not answer is marked where its content would be.
    expect(within(bodyOf(group(/^team/))).getByText(copy.storeMissing('fallbacks'))).toBeTruthy()
    await fireEvent.click(group(/^Pin/))
    expect(within(bodyOf(group(/^Pin/))).getByText(copy.storeMissing('pins'))).toBeTruthy()
  })

  it('marks a profile store it could not read instead of drawing an empty list', () => {
    draw({
      view: chatProfile({
        profiles: [],
        default_profile: null,
        bound: 'fast',
        bound_from_session_file: true,
        stores: { profiles: 'unavailable', fallbacks: 'unavailable', pins: 'read' },
      }),
    })

    expect(screen.getByText(copy.storeMissing('profiles'))).toBeTruthy()
    // The bound name still reaches the page: it came from the session file.
    expect(screen.getByRole('heading', { level: 2, name: /fast/ })).toBeTruthy()
  })
})

describe('the tab in Spanish', () => {
  it('flips every word in the same tick, with no push and no restart', async () => {
    const spanish = profileCopy('es')
    draw({ view: chatProfile({ profiles: [definition('team')], default_profile: 'team' }) })
    expect(screen.getByText(copy.lineDefault('team'))).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(screen.getByText(spanish.lineDefault('team'))).toBeTruthy()
    expect(screen.getByText(spanish.approximation)).toBeTruthy()
    expect(screen.getByText(spanish.readOnly)).toBeTruthy()
    expect(screen.getByText(spanish.tagDefault)).toBeTruthy()
    expect(screen.queryByText(copy.lineDefault('team'))).toBeNull()
  })
})
