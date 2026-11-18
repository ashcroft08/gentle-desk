// The Profile tab's PIN EDITOR (profiles-editor T6, S2, S3): the pin of THIS
// chat's worktree, one row per layer, each with a picker of the profiles the store
// defines (or "no pin", the explicit clear) and one Save. The editor writes
// NOTHING until Save, an unchanged choice cannot be sent, a layer whose pin the
// store does not report is shown as unknown rather than as "no pin", and every
// control is disabled while a save is in flight. What is not writable keeps the
// read-only drawing.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import type { ChatProfileView_Serialize as ChatProfile } from '../contract/bindings'
import { language, setLanguage } from '../i18n/language.svelte'
import { chatProfile, pinLayer, pins } from '../test-support/chat-profile'
import { handlerArgs, resetTauriMock, setHandler } from '../test-support/tauri-mock'
import ProfilePanel from './ProfilePanel.svelte'
import { profileCopy } from './copy'
import { NO_PIN } from './pin-draft'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The tab's own words in English (D4). */
const copy = profileCopy('en')

afterEach(cleanup)
beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => null)
  setLanguage('en')
})

/** Draws the tab as the shell draws it: the open tab of one chat. */
function draw(over: { view?: ChatProfile | null } = {}) {
  return render(ProfilePanel, {
    props: { chatId: 'foreground_chat', view: chatProfile(), active: true, ...over },
  })
}

/** Every pin save the page asked the shell for, in order. */
function saves(): { chat_id: string; source: string; profile: string | null }[] {
  return handlerArgs('dispatch_intent')
    .map((args) => args.intent as { type: string; data?: Record<string, unknown> })
    .filter((intent) => intent.type === 'SaveChatProfilePin')
    .map((intent) => intent.data as never)
}

/** The pin group's own head, in whichever language is drawn. */
function head(): HTMLElement {
  const copy = profileCopy(language.current)
  return screen.getByRole('button', { name: new RegExp(`^${copy.pinTitle}`) })
}

/** The body the pin group head controls, where the editor is drawn. */
function pinBody(): HTMLElement {
  const body = document.getElementById(head().getAttribute('aria-controls') ?? '')
  if (body === null) throw new Error('the pin group head controls no body')
  return body
}

/** One layer's picker, by the label the tab gives it. */
function picker(layer: string): HTMLSelectElement {
  return within(pinBody()).getByRole('combobox', { name: copy.pinPick(layer) }) as HTMLSelectElement
}

/** One layer's Save. */
function saveButton(layer: string): HTMLButtonElement {
  return within(pinBody()).getByRole('button', { name: copy.pinSaveLayer(layer) }) as HTMLButtonElement
}

/** Opens the pin group, where the editor is drawn. */
async function openPins(): Promise<void> {
  await fireEvent.click(head())
}

describe('the pins of this chat, as an editor', () => {
  it('draws one picker per writable layer, holding the pin the store reports', async () => {
    draw()
    await openPins()

    const repo = picker(copy.pinRepo)
    expect(repo.value).toBe('fast')
    expect(picker(copy.pinLocal).value).toBe(NO_PIN)

    // Every profile the store defines is offered, and so is the explicit clear.
    const options = Array.from(repo.querySelectorAll('option')).map((option) => option.value)
    expect(options).toEqual([NO_PIN, 'team', 'fast'])
    expect(within(repo).getByText(copy.pinNoPin)).toBeTruthy()
  })

  it('writes nothing until Save, and then sends the layer and the profile', async () => {
    draw()
    await openPins()

    expect(saveButton(copy.pinRepo).disabled).toBe(true)
    await fireEvent.change(picker(copy.pinRepo), { target: { value: 'team' } })
    expect(saves()).toEqual([])

    await fireEvent.click(saveButton(copy.pinRepo))

    expect(saves()).toEqual([{ chat_id: 'foreground_chat', source: 'repo', profile: 'team' }])
  })

  it('sends the explicit clear for the `no pin` choice', async () => {
    draw()
    await openPins()

    await fireEvent.change(picker(copy.pinRepo), { target: { value: NO_PIN } })
    await fireEvent.click(saveButton(copy.pinRepo))

    expect(saves()).toEqual([{ chat_id: 'foreground_chat', source: 'repo', profile: null }])
  })

  it('sends the layer the user picked, and only that one', async () => {
    draw()
    await openPins()

    await fireEvent.change(picker(copy.pinLocal), { target: { value: 'fast' } })
    await fireEvent.click(saveButton(copy.pinLocal))

    expect(saves()).toEqual([{ chat_id: 'foreground_chat', source: 'local', profile: 'fast' }])
  })

  it('never sends an unchanged choice', async () => {
    draw()
    await openPins()

    // Both layers already hold what the store reports: no pin, and `fast`.
    const local = saveButton(copy.pinLocal)
    expect(local.disabled).toBe(true)
    await fireEvent.click(local)
    await fireEvent.click(saveButton(copy.pinRepo))

    expect(saves()).toEqual([])
  })

  it('disables every picker and Save while a save is in flight', async () => {
    draw({ view: chatProfile({ saving: true }) })
    await openPins()

    expect(picker(copy.pinRepo).disabled).toBe(true)
    expect(picker(copy.pinLocal).disabled).toBe(true)
    expect(saveButton(copy.pinRepo).disabled).toBe(true)
    expect(saveButton(copy.pinLocal).disabled).toBe(true)
  })

  it('does not claim to know a pin the store does not report', async () => {
    // Both files hold a profile the store defines and the repo one outranks the
    // local one, which the `pins` value does not name at all (desktop-rpc.md:211).
    draw({
      view: chatProfile({
        pins: pins('team', {
          layers: [
            pinLayer({ source: 'repo', profile: 'team', mark: 'winner' }),
            pinLayer({ source: 'local', outranked: true }),
          ],
        }),
      }),
    })
    await openPins()

    const local = picker(copy.pinLocal)
    expect(
      within(local).getByText(copy.pinUnreported),
      'the unreported pin is a choice of its own, not "no pin"',
    ).toBeTruthy()
    expect(local.value).toBe('')
    expect(saveButton(copy.pinLocal).disabled).toBe(true)

    // Picking a real choice is a change the user may save.
    await fireEvent.change(local, { target: { value: 'fast' } })
    expect(saveButton(copy.pinLocal).disabled).toBe(false)
  })

  it('refuses, in one line, a pin the profiles store does not define', async () => {
    // A stale layer: the file names a profile the store dropped, so the picker
    // draws it and cannot send it.
    draw({
      view: chatProfile({
        pins: pins('team', {
          resolved: false,
          winner: null,
          layers: [pinLayer({ source: 'repo', profile: 'gone', mark: 'stale' })],
        }),
      }),
    })
    await openPins()

    expect(within(pinBody()).getByText(copy.pinUnknownProfile('gone'))).toBeTruthy()
    expect(saveButton(copy.pinRepo).disabled).toBe(true)
  })

  it('says the pin could not be saved once, and nothing when none failed', async () => {
    draw()
    await openPins()
    expect(screen.queryByText(copy.pinNotSaved)).toBeNull()

    // A push that carries the failure: one line for the whole group, not one per
    // layer.
    await fireEvent.click(head())
    cleanup()
    draw({ view: chatProfile({ pin_failed: true }) })
    await openPins()
    expect(screen.getAllByText(copy.pinNotSaved)).toHaveLength(1)
  })

  it('says the pin is saved in the worktree, never that this chat changes', async () => {
    draw()
    await openPins()

    expect(within(pinBody()).getByText(copy.pinNote)).toBeTruthy()
    expect(copy.pinNote.toLowerCase()).not.toContain('this chat changes')
  })

  it('draws no editor and no error for a chat with no Git worktree', async () => {
    draw({
      view: chatProfile({
        pins: {
          resolved: false,
          winner: null,
          layers: [],
          reason: 'worktree_unresolved',
          worktree_unresolved: true,
        },
      }),
    })
    await openPins()

    expect(within(pinBody()).getByText(copy.noWorktree)).toBeTruthy()
    expect(within(pinBody()).queryAllByRole('combobox')).toHaveLength(0)
    expect(screen.queryByRole('alert')).toBeNull()
  })

  it('draws the store missing instead of an editor when the pins could not be read', async () => {
    draw({
      view: chatProfile({
        pins: null,
        stores: { profiles: 'read', fallbacks: 'read', pins: 'unavailable' },
      }),
    })
    await openPins()

    expect(within(pinBody()).getByText(copy.storeMissing('pins'))).toBeTruthy()
    expect(within(pinBody()).queryAllByRole('combobox')).toHaveLength(0)
  })
})

describe('the pin editor in Spanish', () => {
  it('draws its own words, and sends the same layer and profile', async () => {
    setLanguage('es')
    const spanish = profileCopy('es')
    draw()
    await openPins()

    const repo = within(pinBody()).getByRole('combobox', {
      name: spanish.pinPick(spanish.pinRepo),
    }) as HTMLSelectElement
    expect(repo.value).toBe('fast')

    await fireEvent.change(repo, { target: { value: 'team' } })
    await fireEvent.click(
      within(pinBody()).getByRole('button', { name: spanish.pinSaveLayer(spanish.pinRepo) }),
    )

    expect(saves()).toEqual([{ chat_id: 'foreground_chat', source: 'repo', profile: 'team' }])
  })
})
