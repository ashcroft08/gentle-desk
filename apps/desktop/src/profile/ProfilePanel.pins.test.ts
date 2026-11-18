// The Profile tab's pins, read-only (profiles-editor T2c, state 5 of
// 06-profile-tab.html): the layers the `pins` value names with the mark each one
// carries, and the two quiet cases - a chat with no resolvable worktree, and a
// workspace that wrote no pin at all. Neither of those is an error, so neither
// draws an alert.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import type { ChatProfileView_Serialize as ChatProfile } from '../contract/bindings'
import { setLanguage } from '../i18n/language.svelte'
import { chatProfile, pinLayer, pins } from '../test-support/chat-profile'
import { resetTauriMock, setHandler } from '../test-support/tauri-mock'
import ProfilePanel from './ProfilePanel.svelte'
import { profileCopy } from './copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

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

/** The pin group's head. */
function group(name: RegExp): HTMLElement {
  return screen.getByRole('button', { name })
}

describe('the pins of this chat', () => {
  it('draws each layer with its mark, the winner included', async () => {
    draw()

    await fireEvent.click(group(/^Pin/))

    const pin = bodyOf(group(/^Pin/))
    expect(within(pin).getByText(copy.pinRepo)).toBeTruthy()
    expect(within(pin).getByText(copy.pinWinner)).toBeTruthy()
    // The layer a write may replace is an EDITOR (T6): its picker holds the pin
    // the store reports, so the profile is drawn inside a control, not as text.
    const repo = within(pin).getByRole('combobox', {
      name: copy.pinPick(copy.pinRepo),
    }) as HTMLSelectElement
    expect(repo.value).toBe('fast')
    expect(repo.disabled).toBe(false)
  })

  it('marks the layers that name a gone profile or are not a pin at all', async () => {
    draw({
      view: chatProfile({
        pins: {
          resolved: false,
          winner: null,
          layers: [
            pinLayer({ source: 'repo', profile: 'gone', mark: 'stale' }),
            pinLayer({ source: 'local', status: 'invalid', mark: 'invalid' }),
          ],
          reason: 'stale',
          worktree_unresolved: false,
        },
      }),
    })

    await fireEvent.click(group(/^Pin/))

    const pin = bodyOf(group(/^Pin/))
    expect(within(pin).getByText(copy.pinStale)).toBeTruthy()
    expect(within(pin).getByText(copy.pinInvalid)).toBeTruthy()
    // The local layer is not a pin at all, so its picker holds no pin - the
    // editor's own word for the clear, drawn from the option it starts on.
    const local = within(pin).getByRole('combobox', {
      name: copy.pinPick(copy.pinLocal),
    }) as HTMLSelectElement
    expect(local.value).toBe('')
    expect(within(local).getByText(copy.pinNoPin)).toBeTruthy()
  })

  it('keeps the read-only drawing for a layer this worktree offers no target for', async () => {
    draw({
      view: chatProfile({
        pins: pins('fast', {
          winner: pinLayer({ source: 'repo', profile: 'fast', mark: 'winner', writable: false }),
          layers: [pinLayer({ source: 'repo', profile: 'fast', mark: 'winner', writable: false })],
        }),
      }),
    })

    await fireEvent.click(group(/^Pin/))

    const pin = bodyOf(group(/^Pin/))
    expect(within(pin).getByText('fast')).toBeTruthy()
    expect(within(pin).queryAllByRole('combobox')).toHaveLength(0)
    expect(within(pin).queryByRole('button', { name: /^Save/ })).toBeNull()
  })

  it('draws a chat with no worktree as pins need a Git workspace, with no pin and no error', async () => {
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

    await fireEvent.click(group(/^Pin/))

    const pin = bodyOf(group(/^Pin/))
    expect(within(pin).getByText(copy.noWorktree)).toBeTruthy()
    expect(within(pin).getByText(copy.noWorktreeWhy)).toBeTruthy()
    // No pin is drawn and nothing is an error.
    expect(within(pin).queryByText('fast')).toBeNull()
    expect(screen.queryByRole('alert')).toBeNull()
  })

  it('draws a workspace that wrote no pin as it is, never as a failure', async () => {
    draw({
      view: chatProfile({
        pins: {
          resolved: false,
          winner: null,
          layers: [],
          reason: null,
          worktree_unresolved: false,
        },
      }),
    })

    await fireEvent.click(group(/^Pin/))

    expect(within(bodyOf(group(/^Pin/))).getByText(copy.pinsNone)).toBeTruthy()
  })
})
