// The Settings dialog (settings-dialog S2-S6): the two controls that left the
// footer - the theme picker and the home setting - one section each, with the
// explanation the welcome screen gives for the two homes, and the shell of the
// About and updates section T2 fills in. Dismiss, Escape and the backdrop close
// it, and closing sends nothing: each section persists through its own command.
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
const EXPLAIN =
  'My pi uses the pi installation already on the machine: sign-ins, local models and chats appear in the terminal. Separate creates a clean space at ~/.gentle-shell/agent and touches nothing else.'

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
async function openSettings(over: Partial<ShellView> = {}): Promise<HTMLElement> {
  serveSidebar({ view: viewState({ shell: shell(over), total_sessions: 1 }) })
  render(Sidebar)
  await fireEvent.click(await screen.findByRole('button', { name: TITLE }))
  return await screen.findByRole('dialog')
}

/** One of the dialog's sections, found by the title it draws. */
function section(dialog: HTMLElement, name: string): HTMLElement {
  return within(dialog).getByRole('region', { name })
}

/** The intents the bridge received, oldest first. */
function intents(): unknown[] {
  return handlerArgs('dispatch_intent').map((args) => args.intent)
}

describe('the Settings dialog', () => {
  it('opens from the footer gear with Appearance, Shell home and About and updates in order', async () => {
    const dialog = await openSettings()

    expect(within(dialog).getByRole('heading', { level: 2, name: TITLE })).toBeTruthy()
    expect(
      within(dialog)
        .getAllByRole('region')
        .map((drawn) => drawn.querySelector('h3')?.textContent?.trim()),
    ).toEqual([APPEARANCE, HOME, ABOUT])
  })

  it('draws the theme picker in Appearance, named with the current theme', async () => {
    const dialog = await openSettings()
    const appearance = section(dialog, APPEARANCE)
    const button = within(appearance).getByRole('button', { name: 'Theme: System' })

    expect(button.getAttribute('aria-haspopup')).toBe('listbox')
    await fireEvent.click(button)
    expect(within(appearance).getByRole('listbox', { name: 'Theme' })).toBeTruthy()
  })

  it('draws the home setting and the two-home explanation in Shell home', async () => {
    const dialog = await openSettings({ choice: 'link' })
    const home = section(dialog, HOME)

    expect(within(home).getByText(EXPLAIN)).toBeTruthy()
    expect(
      within(home).getByRole('button', { name: 'My pi' }).getAttribute('aria-pressed'),
    ).toBe('true')
    expect(
      within(home).getByRole('button', { name: 'Separate' }).getAttribute('aria-pressed'),
    ).toBe('false')
  })

  it('keeps the restart line while a home change waits for a restart', async () => {
    const dialog = await openSettings({ choice: 'link', restart_needed: true })

    expect(section(dialog, HOME).querySelector('[role="status"]')?.textContent).toContain(
      'Restart to use this home.',
    )
  })

  it('sends the home the user picks in the dialog, once', async () => {
    const dialog = await openSettings({ choice: 'isolated' })

    await fireEvent.click(within(section(dialog, HOME)).getByRole('button', { name: 'My pi' }))

    expect(intents()).toEqual([{ type: 'SetShellHome', data: { choice: 'link' } }])
  })

  it('leaves About and updates as a titled shell with no fields yet', async () => {
    const dialog = await openSettings()
    const about = section(dialog, ABOUT)

    expect(about.querySelectorAll('input, button, select, a').length).toBe(0)
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

  it('starts the focus on the first control and keeps it in the dialog', async () => {
    const dialog = await openSettings()
    const theme = within(dialog).getByRole('button', { name: 'Theme: System' })
    await vi.waitFor(() => expect(document.activeElement).toBe(theme))

    const close = within(dialog).getByRole('button', { name: 'Close' })
    close.focus()
    await fireEvent.keyDown(close, { key: 'Tab' })

    expect(document.activeElement).toBe(theme)
  })
})
