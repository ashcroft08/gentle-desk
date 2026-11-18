// The chain editor's DRAFT RULES as the page enforces them (profiles-editor T5,
// L30-3): steps are added up to the owner's cap of eight, removed down to one and
// reordered, and a draft that breaks one of the owner's own rules is refused
// BEFORE anything is sent - a repeated model, a step with no model - with the
// reason drawn in the tab's own words. The Spanish column is pinned here too,
// because every line of the editor is a word.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import { tick } from 'svelte'
import type { ChatProfileView_Serialize as ChatProfile } from '../contract/bindings'
import { setLanguage } from '../i18n/language.svelte'
import { chain, chatProfile, definition } from '../test-support/chat-profile'
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

/** The used profile: one agent chain of two steps, and the profile-wide one. */
const USED = definition('team', {
  agents: [{ agent: 'worker', model: 'openai/gpt-5-codex', thinking: 'high', tier: null }],
  chains: [
    chain('*', [{ model: 'anthropic/claude-haiku-4-5', thinking: 'low' }]),
    chain('worker', [
      { model: 'openai/gpt-fallback', thinking: 'low' },
      { model: 'openai/gpt-last', thinking: null },
    ]),
  ],
})

/** Draws the tab as the shell draws it: the open tab of one chat. */
function draw(over: { view?: ChatProfile | null } = {}) {
  return render(ProfilePanel, {
    props: {
      chatId: 'foreground_chat',
      view: chatProfile({ profiles: [USED] }),
      active: true,
      ...over,
    },
  })
}

/** Every chain save the page asked the shell for, in order. */
function saves(): { agent: string; steps: { model: string; thinking: string | null }[] }[] {
  return handlerArgs('dispatch_intent')
    .map((args) => args.intent as { type: string; data?: { agent: string; steps: never } })
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

describe('the chain editor rules', () => {
  it('adds a step, removes one and reorders, and sends the order it drew', async () => {
    draw()

    // Down moves the first step after the second, and the draft is what is sent.
    await fireEvent.click(within(editor('worker')).getByRole('button', { name: copy.chainMoveDown(1) }))
    expect(models('worker').map((input) => input.value)).toEqual([
      'openai/gpt-last',
      'openai/gpt-fallback',
    ])

    // A new step starts empty; the chain is not sendable while a model is blank.
    await fireEvent.click(within(editor('worker')).getByRole('button', { name: copy.chainAddStep }))
    expect(models('worker')).toHaveLength(3)
    expect(within(editor('worker')).getByText(copy.chainMissingModel(3))).toBeTruthy()

    await fireEvent.input(models('worker')[2], { target: { value: 'openai/gpt-new' } })
    await fireEvent.click(within(editor('worker')).getByRole('button', { name: copy.chainSave }))

    expect(saves()[0].steps).toEqual([
      { model: 'openai/gpt-last', thinking: null },
      { model: 'openai/gpt-fallback', thinking: 'low' },
      { model: 'openai/gpt-new', thinking: null },
    ])
  })

  it('removes a step, and never below one', async () => {
    draw()

    await fireEvent.click(within(editor('worker')).getByRole('button', { name: copy.chainRemoveStep(1) }))
    expect(models('worker').map((input) => input.value)).toEqual(['openai/gpt-last'])

    const remove = within(editor('worker')).getByRole('button', {
      name: copy.chainRemoveStep(1),
    }) as HTMLButtonElement
    expect(remove.disabled).toBe(true)
  })

  it('stops adding at eight steps', async () => {
    const eight = Array.from({ length: 8 }, (_, index) => ({
      model: `openai/gpt-step-${index + 1}`,
      thinking: null,
    }))
    draw({
      view: chatProfile({ profiles: [definition('team', { chains: [chain('*', eight)] })] }),
    })

    const add = within(editor(copy.profileWide)).getByRole('button', {
      name: copy.chainAddStep,
    }) as HTMLButtonElement
    expect(add.disabled).toBe(true)
    expect(models(copy.profileWide)).toHaveLength(8)
  })

  it('refuses a chain that repeats a model, and sends nothing', async () => {
    draw()

    await fireEvent.input(models('worker')[0], { target: { value: 'openai/gpt-last' } })

    expect(within(editor('worker')).getByText(copy.chainRepeatedModel('openai/gpt-last'))).toBeTruthy()
    const save = within(editor('worker')).getByRole('button', {
      name: copy.chainSave,
    }) as HTMLButtonElement
    expect(save.disabled).toBe(true)
    await fireEvent.click(save)
    expect(saves()).toEqual([])
  })

  it('refuses a step with no model, and sends nothing', async () => {
    draw()

    await fireEvent.input(models('worker')[0], { target: { value: '   ' } })

    expect(within(editor('worker')).getByText(copy.chainMissingModel(1))).toBeTruthy()
    expect(
      (within(editor('worker')).getByRole('button', { name: copy.chainSave }) as HTMLButtonElement)
        .disabled,
    ).toBe(true)
  })

describe('the chain editor in Spanish', () => {
  it('draws every word in Spanish', async () => {
    const spanish = profileCopy('es')
    draw()

    setLanguage('es')
    await tick()

    expect(within(editor(spanish.profileWide)).getByRole('button', { name: spanish.chainSave })).toBeTruthy()
    expect(within(editor('worker')).getByRole('button', { name: spanish.chainAddStep })).toBeTruthy()
    expect(screen.getByText(spanish.chainNote('team'))).toBeTruthy()
    expect(screen.queryByText(copy.chainSave)).toBeNull()
  })

  it('draws the failure notice and the refusal lines in Spanish', async () => {
    const spanish = profileCopy('es')
    draw({ view: chatProfile({ profiles: [USED], chain_failed: true }) })
    await fireEvent.input(models('worker')[0], { target: { value: 'openai/gpt-last' } })

    setLanguage('es')
    await tick()

    expect(screen.getByText(spanish.chainNotSaved)).toBeTruthy()
    expect(within(editor('worker')).getByText(spanish.chainRepeatedModel('openai/gpt-last'))).toBeTruthy()
  })
})
})
