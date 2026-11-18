// The Profile tab's FALLBACK-CHAIN EDITOR (profiles-editor T5, L30-3): the chains
// the fallback store records for the profile THIS chat uses - an agent's own, and
// the profile-wide `*` - drawn as an ordered list of steps the user can edit, add
// to, remove from and reorder, and saved through the shell one chain at a time
// (`SaveChatProfileChain`). The editor writes NOTHING until Save: typing is a
// draft, Cancel throws it away, a chain the owner would refuse never leaves, and
// an unchanged chain is never sent. Every other profile keeps the read-only
// drawing, because only the used profile is edited here.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import type { ChatProfileView_Serialize as ChatProfile } from '../contract/bindings'
import { setLanguage } from '../i18n/language.svelte'
import { chain, chatProfile, definition, FAST } from '../test-support/chat-profile'
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

/** A used profile with one agent chain and the profile-wide one, so a test can
 * tell the two apart. */
const USED = definition('team', {
  agents: [
    { agent: 'orchestrator', model: 'anthropic/claude-sonnet-4', thinking: 'high', tier: null },
    { agent: 'worker', model: 'openai/gpt-5-codex', thinking: 'high', tier: null },
  ],
  chains: [
    chain('*', [{ model: 'anthropic/claude-haiku-4-5', thinking: 'low' }]),
    chain('worker', [
      { model: 'openai/gpt-fallback', thinking: 'low' },
      { model: 'openai/gpt-last', thinking: null },
    ]),
  ],
})

/** Draws the tab as the shell draws it: the open tab of one chat. */
function draw(over: { chatId?: string | null; view?: ChatProfile | null; active?: boolean } = {}) {
  return render(ProfilePanel, {
    props: { chatId: 'foreground_chat', view: chatProfile({ profiles: [USED, FAST] }), active: true, ...over },
  })
}

/** Every chain save the page asked the shell for, in order. */
function saves(): {
  chat_id: string
  profile: string
  agent: string
  steps: { model: string; thinking: string | null }[]
}[] {
  return handlerArgs('dispatch_intent')
    .map((args) => args.intent as { type: string; data?: Record<string, unknown> })
    .filter((intent) => intent.type === 'SaveChatProfileChain')
    .map((intent) => intent.data as never)
}

/** The editor of one chain, by the name the tab gives it. */
function editor(name: string | RegExp): HTMLElement {
  return screen.getByRole('group', { name })
}

/** Every model input of one chain's editor, in order. */
function models(name: string | RegExp): HTMLInputElement[] {
  return within(editor(name)).getAllByRole('textbox', { name: /^model/i }) as HTMLInputElement[]
}

describe('the used profile chains, as an editor', () => {
  it('draws every chain the store records for the used profile, with its steps', () => {
    draw()

    // The profile-wide chain keeps our own label; the agent chain keeps its key.
    expect(within(editor(copy.profileWide)).getAllByRole('textbox')).toHaveLength(2)
    expect(models(copy.profileWide)[0].value).toBe('anthropic/claude-haiku-4-5')
    expect(models('worker').map((input) => input.value)).toEqual([
      'openai/gpt-fallback',
      'openai/gpt-last',
    ])
    // The effort is its own field, drawn as the store keeps it.
    const efforts = within(editor('worker')).getAllByRole('textbox', { name: /^thinking/i })
    expect((efforts[0] as HTMLInputElement).value).toBe('low')
    expect((efforts[1] as HTMLInputElement).value).toBe('')
  })

  it('keeps the read-only drawing for a profile this chat does not use', async () => {
    draw()

    await fireEvent.click(screen.getByRole('button', { name: /^fast/ }))

    const fast = document.getElementById(
      screen.getByRole('button', { name: /^fast/ }).getAttribute('aria-controls') ?? '',
    ) as HTMLElement
    expect(within(fast).queryAllByRole('textbox')).toHaveLength(0)
    expect(within(fast).queryByRole('button', { name: copy.chainSave })).toBeNull()
  })

  it('writes nothing until Save, and then sends the whole chain', async () => {
    draw()

    const [first] = models('worker')
    await fireEvent.input(first, { target: { value: 'openai/gpt-edited' } })

    expect(saves()).toEqual([])

    await fireEvent.click(within(editor('worker')).getByRole('button', { name: copy.chainSave }))

    expect(saves()).toEqual([
      {
        chat_id: 'foreground_chat',
        profile: 'team',
        agent: 'worker',
        steps: [
          { model: 'openai/gpt-edited', thinking: 'low' },
          { model: 'openai/gpt-last', thinking: null },
        ],
      },
    ])
  })

  it('sends the profile-wide chain under its own key', async () => {
    draw()

    await fireEvent.input(models(copy.profileWide)[0], { target: { value: 'openai/gpt-shared' } })
    await fireEvent.click(
      within(editor(copy.profileWide)).getByRole('button', { name: copy.chainSave }),
    )

    expect(saves()[0].agent).toBe('*')
    expect(saves()[0].steps).toEqual([{ model: 'openai/gpt-shared', thinking: 'low' }])
  })

  it('throws the draft away on Cancel, and sends nothing', async () => {
    draw()

    await fireEvent.input(models('worker')[0], { target: { value: 'openai/gpt-edited' } })
    await fireEvent.click(within(editor('worker')).getByRole('button', { name: copy.chainCancel }))

    expect(models('worker')[0].value).toBe('openai/gpt-fallback')
    expect(saves()).toEqual([])
  })

  it('never sends an unchanged chain', async () => {
    draw()

    const save = within(editor('worker')).getByRole('button', {
      name: copy.chainSave,
    }) as HTMLButtonElement
    expect(save.disabled).toBe(true)
    await fireEvent.click(save)

    expect(saves()).toEqual([])
  })

  it('disables every field and action while a save is in flight', async () => {
    draw({ view: chatProfile({ profiles: [USED, FAST], saving: true }) })

    for (const field of within(editor('worker')).getAllByRole('textbox')) {
      expect((field as HTMLInputElement).disabled).toBe(true)
    }
    for (const name of [copy.chainSave, copy.chainCancel, copy.chainAddStep]) {
      expect(
        (within(editor('worker')).getByRole('button', { name }) as HTMLButtonElement).disabled,
      ).toBe(true)
    }
  })

  it('says the chain could not be saved, once for the whole group', () => {
    draw({ view: chatProfile({ profiles: [USED, FAST], chain_failed: true }) })

    expect(screen.getAllByText(copy.chainNotSaved)).toHaveLength(1)
  })

  it('says nothing about a save when none failed', () => {
    draw()

    expect(screen.queryByText(copy.chainNotSaved)).toBeNull()
  })

  it('says the chains are saved in the profile, never that this chat changes', () => {
    draw()

    expect(screen.getByText(copy.chainNote('team'))).toBeTruthy()
    expect(copy.chainNote('team')).toContain('team')
    expect(copy.chainNote('team').toLowerCase()).not.toContain('this chat changes')
  })

  it('draws the store missing instead of an editor when the chains could not be read', () => {
    draw({
      view: chatProfile({
        profiles: [USED],
        stores: { profiles: 'read', fallbacks: 'unavailable', pins: 'read' },
      }),
    })

    expect(screen.getByText(copy.storeMissing('fallbacks'))).toBeTruthy()
    expect(screen.queryByRole('textbox')).toBeNull()
  })
})
