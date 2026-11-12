// The theme control as 01a draws it: a button named with the current theme that
// opens a listbox of the six themes, drawn in the page's own colors (the native
// select's popup ignored the page theme, which is why 01a has none). The control
// lives in the Settings dialog's Appearance section now (settings-dialog S3), so
// every test here opens that dialog first and then behaves as it always did.
import '../test-support/dom-env'
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import { handlerArgs, resetTauriMock, viewState } from '../test-support/tauri-mock'
import { serveSidebar } from '../test-support/sidebar'
import type { ThemeView } from '../contract/bindings'
import Sidebar from './Sidebar.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/**
 * The six options 01a draws, each label beside the theme identifier it stands
 * for: quoted so a change of copy, or a label leaking into an intent, is a
 * change here.
 */
const OPTIONS: readonly (readonly [ThemeView, string])[] = [
  ['system', 'System'],
  ['dark', 'Dark'],
  ['Gentle', 'Gentle'],
  ['Gentleman-Cute', 'Gentleman-Cute'],
  ['Gentleman-Sexy', 'Gentleman-Sexy'],
  ['light', 'Light'],
]

/** The six labels the list draws, in the drawn order. */
const NAMES = OPTIONS.map(([, label]) => label)

/** The label the control names one theme with. */
function labelFor(theme: ThemeView): string {
  const option = OPTIONS.find(([id]) => id === theme)
  if (option === undefined) throw new Error(`no label for the theme ${theme}`)
  return option[1]
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

/**
 * The sidebar over one state. The first read of the state is a command, so the
 * mount waits for it: the button names the theme the state carries, not the
 * default it starts with.
 */
async function mountSidebar(theme: ThemeView = 'system') {
  const fixture = serveSidebar({ view: viewState({ theme, total_sessions: 1 }) })
  render(Sidebar)
  await openSettings()
  await vi.waitFor(() =>
    expect(themeButton().getAttribute('aria-label')).toBe(`Theme: ${labelFor(theme)}`),
  )
  return fixture
}

/** The picker's only home now: the Settings dialog the footer's gear opens. */
async function openSettings(): Promise<void> {
  await fireEvent.click(await screen.findByRole('button', { name: 'Settings' }))
  await screen.findByRole('dialog')
}

/** The control in the dialog's Appearance section, whatever theme it names. */
function themeButton(): HTMLElement {
  return screen.getByRole('button', { name: /^Theme: / })
}

/** The same control, found by the theme it SHOWS: the drawn name and nothing else. */
function buttonNamed(name: string): HTMLElement {
  return screen.getByRole('button', { name })
}

async function openList(): Promise<HTMLElement> {
  await fireEvent.click(themeButton())
  return await screen.findByRole('listbox', { name: 'Theme' })
}

/** The option the list has moved the focus to. */
function focusedOption(label: string): void {
  expect(document.activeElement).toBe(screen.getByRole('option', { name: label }))
}

async function keydown(target: Element, key: string, init: KeyboardEventInit = {}): Promise<void> {
  await fireEvent.keyDown(target, { key, ...init })
}

/** The `SetTheme` intents the bridge received, oldest first. */
function themeIntents(): ThemeView[] {
  return handlerArgs('dispatch_intent')
    .map((args) => args.intent as { type: string; data?: { theme: ThemeView } })
    .filter((intent) => intent.type === 'SetTheme')
    .map((intent) => intent.data!.theme)
}

describe('the theme picker', () => {
  it('is a listbox button that shows the current theme', async () => {
    await mountSidebar('Gentle')
    const button = buttonNamed('Theme: Gentle')

    expect(button.getAttribute('aria-haspopup')).toBe('listbox')
    expect(button.getAttribute('aria-expanded')).toBe('false')

    await openList()

    expect(button.getAttribute('aria-expanded')).toBe('true')
    expect(screen.getByRole('listbox', { name: 'Theme' })).not.toBeNull()
  })

  it('draws each theme as its label over the identifier it keeps', async () => {
    await mountSidebar('dark')

    // The button names the LABEL, so the control is not mute in lowercase.
    expect(screen.getByRole('button', { name: 'Theme: Dark' })).not.toBeNull()
    expect(screen.queryByRole('button', { name: 'Theme: dark' })).toBeNull()

    const list = await openList()

    expect(within(list).getAllByRole('option').map((option) => option.textContent?.trim())).toEqual(
      NAMES,
    )
    // The label is drawn, never sent: the value each option carries is the
    // theme identifier the config, the contract and the stylesheet use.
    expect(
      within(list).getAllByRole('option').map((option) => option.getAttribute('data-value')),
    ).toEqual(OPTIONS.map(([theme]) => theme))

    await fireEvent.click(screen.getByRole('option', { name: 'Dark' }))

    expect(themeIntents()).toEqual(['dark'])
  })

  it('draws a quiet color swatch beside every row', async () => {
    await mountSidebar('system')
    const list = await openList()

    const dots = within(list)
      .getAllByRole('option')
      .map((option) => option.querySelector('.swatch'))
    expect(dots).toHaveLength(OPTIONS.length)
    for (const dot of dots) {
      // The dot is decoration: the option's name is still its word.
      expect(dot?.getAttribute('aria-hidden')).toBe('true')
      expect((dot as HTMLElement).style.getPropertyValue('--swatch').trim()).not.toBe('')
    }
  })

  it('opens exactly the six drawn names, in the drawn order, on the current one', async () => {
    await mountSidebar('Gentle')
    const list = await openList()

    expect(within(list).getAllByRole('option').map((option) => option.textContent?.trim())).toEqual(
      NAMES,
    )
    expect(screen.getByRole('option', { name: 'Gentle' }).getAttribute('aria-selected')).toBe('true')
    expect(screen.getByRole('option', { name: 'Dark' }).getAttribute('aria-selected')).toBe('false')
    await vi.waitFor(() => focusedOption('Gentle'))
  })

  it('moves with the arrow keys and Home and End, and stops at both ends', async () => {
    await mountSidebar('system')
    const list = await openList()

    await keydown(list, 'ArrowDown')
    focusedOption('Dark')
    await keydown(list, 'ArrowDown')
    focusedOption('Gentle')
    await keydown(list, 'ArrowUp')
    focusedOption('Dark')
    await keydown(list, 'Home')
    focusedOption('System')
    await keydown(list, 'ArrowUp')
    focusedOption('System')
    await keydown(list, 'End')
    focusedOption('Light')
    await keydown(list, 'ArrowDown')
    focusedOption('Light')
  })

  it('keeps the focus inside the list while it is open', async () => {
    await mountSidebar('system')
    const list = await openList()
    focusedOption('System')

    await keydown(list, 'Tab')
    focusedOption('Dark')
    await keydown(list, 'Tab', { shiftKey: true })
    focusedOption('System')
    // Tab never leaves the list: from the first option it walks to the last and
    // stops there, as the arrow keys do.
    for (let step = 0; step < 6; step += 1) await keydown(list, 'Tab')
    focusedOption('Light')
    expect(list.contains(document.activeElement)).toBe(true)
  })

  it('selects the focused theme with Enter, once, and closes back to the button', async () => {
    const fixture = await mountSidebar('system')
    const list = await openList()

    await keydown(list, 'ArrowDown')
    await keydown(list, 'ArrowDown')
    await keydown(list, 'Enter')

    expect(themeIntents()).toEqual(['Gentle'])
    expect(screen.queryByRole('listbox')).toBeNull()
    expect(document.activeElement).toBe(themeButton())
    // The answer is Rust's push, which carries the chosen theme back.
    fixture.view = viewState({ theme: 'Gentle' })
    fixture.push()
    await vi.waitFor(() => buttonNamed('Theme: Gentle'))
  })

  it('selects with Space and with a click, one intent per choice', async () => {
    await mountSidebar('system')
    const list = await openList()

    await keydown(list, 'End')
    await keydown(list, ' ')
    expect(themeIntents()).toEqual(['light'])

    await fireEvent.click(themeButton())
    await fireEvent.click(screen.getByRole('option', { name: 'Gentleman-Cute' }))

    expect(themeIntents()).toEqual(['light', 'Gentleman-Cute'])
    expect(screen.queryByRole('listbox')).toBeNull()
    expect(document.activeElement).toBe(themeButton())
  })

  it('closes on Escape and gives the focus back to the button', async () => {
    await mountSidebar('system')
    const list = await openList()

    await keydown(list, 'Escape')

    expect(screen.queryByRole('listbox')).toBeNull()
    expect(document.activeElement).toBe(themeButton())
    expect(themeIntents()).toEqual([])
  })

  it('closes when the user clicks outside it, and keeps the focus in the dialog', async () => {
    await mountSidebar('system')
    const dialog = screen.getByRole('dialog')
    await openList()

    // With the picker in the dialog, the page behind it is out of reach: the
    // rest of the card is what "outside the list" means now.
    await fireEvent.click(within(dialog).getByRole('heading', { name: 'Settings' }))

    expect(screen.queryByRole('listbox')).toBeNull()
    // The dialog traps the focus (settings-dialog S6), so it never reaches the
    // page behind the card.
    expect(dialog.contains(document.activeElement)).toBe(true)
  })
})
