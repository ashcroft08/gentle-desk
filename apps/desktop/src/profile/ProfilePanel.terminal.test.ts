// The Profile tab's one way out: the terminal (profiles-editor T2d, L21-L22).
// Settings > Profiles is gone, so the profiles screen the terminal already edits
// is opened from the chat's own tab. The action writes nothing and reads nothing
// by itself: it opens a window, it is drawn in every state, and a refusal is the
// one line under the button instead of a button that did nothing.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import { tick } from 'svelte'
import { setLanguage } from '../i18n/language.svelte'
import { chatProfile } from '../test-support/chat-profile'
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
  setHandler('open_terminal', () => null)
  setLanguage('en')
})

/** Draws the tab as the shell draws it: the open tab of one chat. */
function draw(over: { view?: ReturnType<typeof chatProfile> | null } = {}) {
  return render(ProfilePanel, {
    props: { chatId: 'foreground_chat', view: chatProfile(), active: true, ...over },
  })
}

describe('the way out: the terminal', () => {
  it('draws the terminal action and its note in the footer, beside the line saying what is saved', () => {
    draw()

    // The footer's own line, once T3-mode made the tab write the used profile's
    // orchestration mode: it is no longer a read-only tab, so it no longer says so.
    expect(screen.getByText(copy.footer)).toBeTruthy()
    expect(screen.getByRole('button', { name: copy.editInTerminal })).toBeTruthy()
    expect(screen.getByText(copy.editNote)).toBeTruthy()
  })

  it('offers the terminal even when this chat could not be read', () => {
    // The way out must not depend on the child answering: a chat whose read
    // failed is exactly when its profile may need the terminal.
    draw({
      view: chatProfile({
        bound: null,
        profiles: [],
        default_profile: null,
        pins: null,
        stores: { profiles: 'unavailable', fallbacks: 'skipped', pins: 'skipped' },
      }),
    })

    expect(screen.getByRole('button', { name: copy.editInTerminal })).toBeTruthy()
  })

  it('opens the shell profiles screen in a terminal, and sends nothing else', async () => {
    draw()
    const asked = handlerArgs('dispatch_intent').map((args) => args.intent)

    await fireEvent.click(screen.getByRole('button', { name: copy.editInTerminal }))

    expect(handlerArgs('open_terminal')).toEqual([{ action: 'Profiles' }])
    // It opens a window: the tab asks the shell for nothing else.
    expect(handlerArgs('dispatch_intent').map((args) => args.intent)).toEqual(asked)
  })

  it('says the terminal could not open, instead of a button that does nothing', async () => {
    setHandler('open_terminal', () => Promise.reject('gentle-shell could not be found'))
    draw()

    await fireEvent.click(screen.getByRole('button', { name: copy.editInTerminal }))

    await vi.waitFor(() =>
      expect(screen.getByRole('status').textContent).toContain('gentle-shell could not be found'),
    )
  })

  it('flips the action and its note with the language, in the same tick', async () => {
    const spanish = profileCopy('es')
    draw()
    expect(screen.getByRole('button', { name: copy.editInTerminal })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(screen.getByRole('button', { name: spanish.editInTerminal })).toBeTruthy()
    expect(screen.getByText(spanish.editNote)).toBeTruthy()
    expect(screen.queryByText(copy.editNote)).toBeNull()
  })
})
