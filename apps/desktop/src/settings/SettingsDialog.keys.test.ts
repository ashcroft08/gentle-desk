// The Settings dialog's own keys and focus (settings-dialog L4): the rail is one
// vertical tablist with a single tab stop, the arrows walk it and wrap, Home and
// End jump to the ends, and the dialog closes from Escape, its Close button and
// the backdrop without sending anything. What each pane draws is
// SettingsDialog.test.ts's.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, within } from '@testing-library/svelte'
import { intents, openSettings, pane, rail } from '../test-support/settings'
import { resetTauriMock } from '../test-support/tauri-mock'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The words this file keys on, quoted so a change of copy is a change here. */
const APPEARANCE = 'Appearance'
const HOME = 'Shell home'
const ABOUT = 'About'
/** The two-home line, drawn by the section the arrows land on. */
const EXPLAIN =
  'My pi uses the pi installation already on the machine: sign-ins, local models and chats appear in the terminal. Separate creates a clean space at ~/.gentle-shell/agent and touches nothing else.'

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

describe('the Settings dialog keys', () => {
  it('walks the rail with the arrow keys and Home and End, one tab stop', async () => {
    const dialog = await openSettings()

    // Only the selected entry takes the tab: the rest are reached by key.
    expect(rail(dialog).map((entry) => entry.getAttribute('tabindex'))).toEqual([
      '0',
      '-1',
      '-1',
      '-1',
      '-1',
      '-1',
    ])

    await fireEvent.keyDown(within(dialog).getByRole('tab', { name: APPEARANCE }), {
      key: 'ArrowDown',
    })
    expect(within(dialog).getByRole('tab', { name: HOME }).getAttribute('aria-selected')).toBe(
      'true',
    )
    expect(document.activeElement).toBe(within(dialog).getByRole('tab', { name: HOME }))
    expect(within(pane(dialog)).getByText(EXPLAIN)).toBeTruthy()

    await fireEvent.keyDown(within(dialog).getByRole('tab', { name: HOME }), { key: 'End' })
    expect(within(dialog).getByRole('tab', { name: ABOUT }).getAttribute('aria-selected')).toBe(
      'true',
    )

    await fireEvent.keyDown(within(dialog).getByRole('tab', { name: ABOUT }), { key: 'Home' })
    expect(
      within(dialog).getByRole('tab', { name: APPEARANCE }).getAttribute('aria-selected'),
    ).toBe('true')

    // The rail wraps at both ends, as the chat header's tablist does.
    await fireEvent.keyDown(within(dialog).getByRole('tab', { name: APPEARANCE }), {
      key: 'ArrowUp',
    })
    expect(within(dialog).getByRole('tab', { name: ABOUT }).getAttribute('aria-selected')).toBe(
      'true',
    )
  })

  it('closes on Escape and sends nothing: the sections keep their own paths', async () => {
    const dialog = await openSettings()

    await fireEvent.keyDown(document.body, { key: 'Escape' })

    await vi.waitFor(() => expect(dialog.isConnected).toBe(false))
    expect(intents()).toEqual([])
  })

  it('closes from its Close button and sends nothing', async () => {
    const dialog = await openSettings()

    await fireEvent.click(within(dialog).getByRole('button', { name: 'Close' }))

    await vi.waitFor(() => expect(dialog.isConnected).toBe(false))
    expect(intents()).toEqual([])
  })

  it('closes when the pointer lands on the backdrop', async () => {
    const dialog = await openSettings()

    const overlay = document.querySelector('.ovl')
    expect(overlay).toBeTruthy()
    // jsdom has no PointerEvent, so the press is a MouseEvent carrying the
    // coordinates the primitive checks: a backdrop press lands far from the
    // dialog's own box (WhatsNewDialog.test.ts does the same).
    const backdrop = new MouseEvent('pointerdown', {
      bubbles: true,
      cancelable: true,
      clientX: 500,
      clientY: 500,
    })
    await fireEvent(overlay as HTMLElement, backdrop)

    await vi.waitFor(() => expect(dialog.isConnected).toBe(false))
    expect(intents()).toEqual([])
  })

  it('starts the focus on the selected rail entry and keeps it in the dialog', async () => {
    const dialog = await openSettings()
    const entry = within(dialog).getByRole('tab', { name: APPEARANCE })
    await vi.waitFor(() => expect(document.activeElement).toBe(entry))

    const close = within(dialog).getByRole('button', { name: 'Close' })
    close.focus()
    await fireEvent.keyDown(close, { key: 'Tab' })

    expect(document.activeElement).toBe(entry)
  })
})
