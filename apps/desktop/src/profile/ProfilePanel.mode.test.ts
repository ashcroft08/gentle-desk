// The Profile tab's orchestrator-mode SWITCH (profiles-editor T3-mode, L23, L24):
// the same Organic / Full delegation pair Settings > Profiles draws, for the
// profile THIS chat uses, and it SAVES the profile's own setting through the
// shell (`SaveChatProfileMode`). The mode is the profile's, so the copy says it is
// saved in the profile and never that this chat changes live (the probe of L24
// did not establish that). Nothing here writes twice, and nothing writes when the
// chosen mode is already the one stored. One test per rule, EN and ES.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import { tick } from 'svelte'
import type { ChatProfileView_Serialize as ChatProfile } from '../contract/bindings'
import { setLanguage } from '../i18n/language.svelte'
import { chatProfile, definition, TEAM } from '../test-support/chat-profile'
import { handlerArgs, resetTauriMock, setHandler } from '../test-support/tauri-mock'
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

/** Every save the page asked the shell for, in order. */
function saves(): { chat_id: string; profile: string; mode: string }[] {
  return handlerArgs('dispatch_intent')
    .map((args) => args.intent as { type: string; data?: { chat_id: string; profile: string; mode: string } })
    .filter((intent) => intent.type === 'SaveChatProfileMode')
    .map((intent) => intent.data as { chat_id: string; profile: string; mode: string })
}

/** The switch itself: the one radiogroup of the tab. */
function group(): HTMLElement {
  return screen.getByRole('radiogroup', { name: copy.modeLabel })
}

describe("the used profile's mode, as a switch", () => {
  it("draws the mode the used profile stores, as the checked option", () => {
    draw({ view: chatProfile({ profiles: [TEAM], default_profile: 'team' }) })

    const organic = within(group()).getByRole('radio', { name: copy.modeOrganic })
    const full = within(group()).getByRole('radio', { name: copy.modeFullDelegation })
    expect(organic.getAttribute('aria-checked')).toBe('true')
    expect(full.getAttribute('aria-checked')).toBe('false')
  })

  it('draws full delegation as the checked option for a profile that stores it', () => {
    const delegated = definition('team', { mode: 'full-delegation' })
    draw({ view: chatProfile({ profiles: [delegated], default_profile: 'team' }) })

    expect(
      within(group()).getByRole('radio', { name: copy.modeFullDelegation }).getAttribute('aria-checked'),
    ).toBe('true')
  })

  it('sends the intent with the chat, the profile and the chosen mode', async () => {
    draw({ view: chatProfile({ profiles: [TEAM], default_profile: 'team' }) })

    await fireEvent.click(within(group()).getByRole('radio', { name: copy.modeFullDelegation }))

    expect(saves()).toEqual([
      { chat_id: 'foreground_chat', profile: 'team', mode: 'full-delegation' },
    ])
  })

  it('sends organic for the profile this chat BOUND, never the default one', async () => {
    const busy = definition('busy', { mode: 'full-delegation' })
    draw({
      view: chatProfile({
        profiles: [TEAM, busy],
        default_profile: 'team',
        bound: 'busy',
        bound_from_session_file: true,
      }),
    })

    await fireEvent.click(within(group()).getByRole('radio', { name: copy.modeOrganic }))

    expect(saves()).toEqual([{ chat_id: 'foreground_chat', profile: 'busy', mode: 'organic' }])
  })

  it('never writes when the chosen mode is already the one stored', async () => {
    draw({ view: chatProfile({ profiles: [TEAM], default_profile: 'team' }) })

    await fireEvent.click(within(group()).getByRole('radio', { name: copy.modeOrganic }))

    expect(saves()).toEqual([])
  })

  it('disables both options while a save is in flight', async () => {
    draw({ view: chatProfile({ profiles: [TEAM], saving: true }) })

    const organic = within(group()).getByRole('radio', { name: copy.modeOrganic }) as HTMLButtonElement
    const full = within(group()).getByRole('radio', { name: copy.modeFullDelegation }) as HTMLButtonElement
    expect(organic.disabled).toBe(true)
    expect(full.disabled).toBe(true)

    await fireEvent.click(full)
    expect(saves()).toEqual([])
  })

  it('says the mode could not be saved, in one short line', () => {
    draw({ view: chatProfile({ profiles: [TEAM], save_failed: true }) })

    expect(screen.getByText(copy.modeNotSaved)).toBeTruthy()
  })

  it('says nothing about a save when none failed', () => {
    draw({ view: chatProfile({ profiles: [TEAM] }) })

    expect(screen.queryByText(copy.modeNotSaved)).toBeNull()
  })

  it('says it is saved in the profile, never that this chat changes', () => {
    draw({ view: chatProfile({ profiles: [TEAM], default_profile: 'team' }) })

    expect(within(group().parentElement as HTMLElement).getByText(copy.modeNote('team'))).toBeTruthy()
    // The note is about the profile's setting: no line promises a live switch.
    expect(copy.modeNote('team')).toContain('team')
    expect(copy.modeNote('team').toLowerCase()).not.toContain('this chat changes')
  })

  it('draws every word in Spanish', async () => {
    const spanish = profileCopy('es')
    draw({ view: chatProfile({ profiles: [TEAM], default_profile: 'team' }) })

    setLanguage('es')
    await tick()

    const block = screen.getByRole('radiogroup', { name: spanish.modeLabel })
    expect(within(block).getByRole('radio', { name: spanish.modeOrganic })).toBeTruthy()
    expect(within(block).getByRole('radio', { name: spanish.modeFullDelegation })).toBeTruthy()
    expect(screen.getByText(spanish.modeNote('team'))).toBeTruthy()
    expect(screen.queryByText(copy.modeNotSaved)).toBeNull()
  })

  it('draws the failure notice in Spanish too', async () => {
    const spanish = profileCopy('es')
    draw({ view: chatProfile({ profiles: [TEAM], save_failed: true }) })

    setLanguage('es')
    await tick()

    expect(screen.getByText(spanish.modeNotSaved)).toBeTruthy()
  })
})
