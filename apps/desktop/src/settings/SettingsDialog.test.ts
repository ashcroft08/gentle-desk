// The Settings dialog (settings-dialog S2-S6, L4): a rail of three sections on
// the left and the selected section's content in the pane on the right - the two
// controls that left the footer (the theme picker and the home setting), each
// with its own short line, with the welcome screen's explanation of the two
// homes, and About and updates: the app's name, the version this run runs as,
// and the manual check (the check itself is AboutUpdates.test.ts's).
// Dismiss, Escape and the backdrop close it, and closing sends nothing: each
// section persists through its own command.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import type { ShellView_Serialize as ShellView } from '../contract/bindings'
import { handlerArgs, resetTauriMock, viewState } from '../test-support/tauri-mock'
import { serveSidebar } from '../test-support/sidebar'
import Sidebar from '../sidebar/Sidebar.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

const PI = 'C:/Users/someone/.pi/agent'

/** The words the dialog draws, quoted here so a change of copy is a change here. */
const TITLE = 'Settings'
const APPEARANCE = 'Appearance'
const HOME = 'Shell home'
const ABOUT = 'About and updates'
/** The rail's three entries, in the order the dialog draws them. */
const RAIL = [APPEARANCE, HOME, ABOUT]
const APPEARANCE_HINT = 'The look of this window.'
const HOME_HINT = 'Which pi this shell runs on.'
const ABOUT_HINT = 'Which build this is, and whether a newer one exists.'
/** The generic line the stacked dialog drew under its title: it is gone now. */
const OLD_HINT = 'The look of this window, and the home gentle shell runs on.'
const EXPLAIN =
  'My pi uses the pi installation already on the machine: sign-ins, local models and chats appear in the terminal. Separate creates a clean space at ~/.gentle-shell/agent and touches nothing else.'
/** About and updates (S5): the name, and the version this run runs as. */
const APP_NAME = 'Gentle Shell Desktop'
const VERSION = '0.1.9'
const VERSION_LINE = 'Version 0.1.9'
const VERSION_UNKNOWN = 'Version unknown'

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

function shell(over: Partial<ShellView> = {}): ShellView {
  return {
    pi_path: PI,
    choice: 'isolated',
    ask: false,
    launcher_found: true,
    install: { type: 'Idle' },
    restart_needed: false,
    ...over,
  }
}

/** The dialog, opened the way the user opens it: the footer's gear. */
async function openSettings(
  over: Partial<ShellView> = {},
  appVersion: string | null = VERSION,
): Promise<HTMLElement> {
  serveSidebar({
    view: viewState({ shell: shell(over), total_sessions: 1, app_version: appVersion }),
  })
  render(Sidebar)
  await fireEvent.click(await screen.findByRole('button', { name: TITLE }))
  return await screen.findByRole('dialog')
}

/** The rail's entries, in the order the dialog draws them. */
function rail(dialog: HTMLElement): HTMLElement[] {
  return within(dialog).getAllByRole('tab')
}

/** The one section drawn in the pane: only the selected one is there. */
function pane(dialog: HTMLElement): HTMLElement {
  return within(dialog).getByRole('tabpanel')
}

/** Picks a section in the rail, the way the user does. */
async function pick(dialog: HTMLElement, name: string): Promise<void> {
  await fireEvent.click(within(dialog).getByRole('tab', { name }))
}

/** The intents the bridge received, oldest first. */
function intents(): unknown[] {
  return handlerArgs('dispatch_intent').map((args) => args.intent)
}

describe('the Settings dialog', () => {
  it('opens from the footer gear with the three sections in its rail and Appearance first', async () => {
    const dialog = await openSettings()

    expect(within(dialog).getByRole('heading', { level: 2, name: TITLE })).toBeTruthy()
    expect(rail(dialog).map((entry) => entry.textContent?.trim())).toEqual(RAIL)
    expect(
      rail(dialog).map((entry) => entry.getAttribute('aria-selected')),
    ).toEqual(['true', 'false', 'false'])
    expect(within(dialog).getAllByRole('tabpanel')).toHaveLength(1)
    expect(within(dialog).getByRole('tabpanel', { name: APPEARANCE })).toBeTruthy()
  })

  it('draws one short hint per section and drops the old generic line', async () => {
    const dialog = await openSettings()

    expect(within(pane(dialog)).getByText(APPEARANCE_HINT)).toBeTruthy()
    expect(within(dialog).queryByText(OLD_HINT)).toBeNull()

    await pick(dialog, HOME)
    expect(within(pane(dialog)).getByText(HOME_HINT)).toBeTruthy()

    await pick(dialog, ABOUT)
    expect(within(pane(dialog)).getByText(ABOUT_HINT)).toBeTruthy()
    expect(within(dialog).queryByText(APPEARANCE_HINT)).toBeNull()
  })

  it('draws the theme picker in Appearance, named with the current theme', async () => {
    const dialog = await openSettings()
    const button = within(pane(dialog)).getByRole('button', { name: 'Theme: System' })

    expect(button.getAttribute('aria-haspopup')).toBe('listbox')
    await fireEvent.click(button)
    expect(within(pane(dialog)).getByRole('listbox', { name: 'Theme' })).toBeTruthy()
  })

  it('switches the pane to Shell home when its rail entry is picked', async () => {
    const dialog = await openSettings({ choice: 'link' })

    await pick(dialog, HOME)
    const home = pane(dialog)

    expect(within(home).getByText(EXPLAIN)).toBeTruthy()
    expect(within(home).getByRole('button', { name: 'My pi' }).getAttribute('aria-pressed')).toBe(
      'true',
    )
    expect(
      within(home).getByRole('button', { name: 'Separate' }).getAttribute('aria-pressed'),
    ).toBe('false')
    // Appearance left the pane, and the rail says which section is drawn.
    expect(within(dialog).queryByRole('button', { name: 'Theme: System' })).toBeNull()
    expect(within(dialog).getByRole('tab', { name: HOME }).getAttribute('aria-selected')).toBe(
      'true',
    )
    expect(
      within(dialog).getByRole('tab', { name: APPEARANCE }).getAttribute('aria-selected'),
    ).toBe('false')
  })

  it('keeps the restart line while a home change waits for a restart', async () => {
    const dialog = await openSettings({ choice: 'link', restart_needed: true })

    await pick(dialog, HOME)

    expect(pane(dialog).querySelector('[role="status"]')?.textContent).toContain(
      'Restart to use this home.',
    )
  })

  it('sends the home the user picks in the dialog, once', async () => {
    const dialog = await openSettings({ choice: 'isolated' })

    await pick(dialog, HOME)
    await fireEvent.click(within(pane(dialog)).getByRole('button', { name: 'My pi' }))

    expect(intents()).toEqual([{ type: 'SetShellHome', data: { choice: 'link' } }])
  })

  it('switches to About and updates: the name, the version and Check now', async () => {
    const dialog = await openSettings()

    await pick(dialog, ABOUT)
    const about = pane(dialog)

    expect(within(about).getByText(APP_NAME)).toBeTruthy()
    expect(within(about).getByText(VERSION_LINE)).toBeTruthy()
    expect(within(about).getByRole('button', { name: 'Check now' })).toBeTruthy()
    // One section at a time: the other two are not drawn.
    expect(within(dialog).queryByText(EXPLAIN)).toBeNull()
    expect(within(dialog).queryByRole('button', { name: 'Theme: System' })).toBeNull()
  })

  it('says it does not know the version when the bridge named none', async () => {
    const dialog = await openSettings({}, null)

    await pick(dialog, ABOUT)

    expect(within(pane(dialog)).getByText(VERSION_UNKNOWN)).toBeTruthy()
  })

  it('walks the rail with the arrow keys and Home and End, one tab stop', async () => {
    const dialog = await openSettings()

    // Only the selected entry takes the tab: the rest are reached by key.
    expect(rail(dialog).map((entry) => entry.getAttribute('tabindex'))).toEqual(['0', '-1', '-1'])

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
