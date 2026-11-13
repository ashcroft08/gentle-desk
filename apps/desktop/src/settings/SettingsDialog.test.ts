// The Settings dialog (settings-dialog S2-S6, L4; 0.1.12 S1-S7): a rail of six
// sections on the left and the selected section's content in the pane on the
// right - the two controls that left the footer (the theme picker and the home
// setting), the Updates, Profiles and Account panes (0.1.12) and About: the
// app's name and the version this run runs as.
// Its closing keys, its focus and the keyboard walk of the rail are
// SettingsDialog.keys.test.ts's; this file is about what the pane draws.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, within } from '@testing-library/svelte'
import { intents, openSettings, pane, pick, rail } from '../test-support/settings'
import { resetTauriMock } from '../test-support/tauri-mock'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The words the dialog draws, quoted here so a change of copy is a change here. */
const TITLE = 'Settings'
const APPEARANCE = 'Appearance'
const HOME = 'Shell home'
const UPDATES = 'Updates'
const PROFILES = 'Profiles'
const ACCOUNT = 'Account'
const ABOUT = 'About'
/** The rail's six entries, in the order the dialog draws them (0.1.12). */
const RAIL = [APPEARANCE, HOME, UPDATES, PROFILES, ACCOUNT, ABOUT]
const APPEARANCE_HINT = 'The look of this window.'
const HOME_HINT = 'Which pi this shell runs on.'
const UPDATES_HINT = 'What this machine runs, and whether something newer is out.'
const PROFILES_HINT = 'Your agent profiles, as the terminal keeps them.'
const ACCOUNT_HINT = 'Which providers are signed in, and where to sign in or out.'
const ABOUT_HINT = 'Which build this is.'
/** The generic line the stacked dialog drew under its title: it is gone now. */
const OLD_HINT = 'The look of this window, and the home gentle shell runs on.'
const EXPLAIN =
  'My pi uses the pi installation already on the machine: sign-ins, local models and chats appear in the terminal. Separate creates a clean space at ~/.gentle-shell/agent and touches nothing else.'
/** About (S5): the name, and the version this run runs as. */
const APP_NAME = 'Gentle Shell Desktop'
const VERSION = '0.1.9'
const VERSION_LINE = 'Version 0.1.9'
const VERSION_UNKNOWN = 'Version unknown'

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

describe('the Settings dialog', () => {
  it('opens from the footer gear with the six sections in its rail and Appearance first', async () => {
    const dialog = await openSettings()

    expect(within(dialog).getByRole('heading', { level: 2, name: TITLE })).toBeTruthy()
    expect(rail(dialog).map((entry) => entry.textContent?.trim())).toEqual(RAIL)
    expect(
      rail(dialog).map((entry) => entry.getAttribute('aria-selected')),
    ).toEqual(['true', 'false', 'false', 'false', 'false', 'false'])
    expect(within(dialog).getAllByRole('tabpanel')).toHaveLength(1)
    expect(within(dialog).getByRole('tabpanel', { name: APPEARANCE })).toBeTruthy()
  })

  it('draws one short hint per section and drops the old generic line', async () => {
    const dialog = await openSettings()

    expect(within(pane(dialog)).getByText(APPEARANCE_HINT)).toBeTruthy()
    expect(within(dialog).queryByText(OLD_HINT)).toBeNull()

    for (const [name, hint] of [
      [HOME, HOME_HINT],
      [UPDATES, UPDATES_HINT],
      [PROFILES, PROFILES_HINT],
      [ACCOUNT, ACCOUNT_HINT],
      [ABOUT, ABOUT_HINT],
    ]) {
      await pick(dialog, name)
      expect(within(pane(dialog)).getByText(hint)).toBeTruthy()
      expect(within(dialog).queryByText(APPEARANCE_HINT)).toBeNull()
    }
  })

  it('draws the three 0.1.12 panes in their own sections, one at a time', async () => {
    const dialog = await openSettings()

    await pick(dialog, UPDATES)
    expect(within(pane(dialog)).getByRole('button', { name: 'Check again' })).toBeTruthy()

    await pick(dialog, PROFILES)
    expect(within(pane(dialog)).getByRole('radiogroup')).toBeTruthy()
    expect(within(dialog).queryByRole('button', { name: 'Check again' })).toBeNull()

    await pick(dialog, ACCOUNT)
    expect(within(pane(dialog)).getByRole('button', { name: 'Check again' })).toBeTruthy()
    expect(within(pane(dialog)).getByRole('button', { name: 'Open login in terminal' })).toBeTruthy()
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

  it('switches to About: the name and the version this run runs as', async () => {
    const dialog = await openSettings()

    await pick(dialog, ABOUT)
    const about = pane(dialog)

    expect(within(about).getByText(APP_NAME)).toBeTruthy()
    expect(within(about).getByText(VERSION_LINE)).toBeTruthy()
    // The check lives in the Updates section now, not here.
    expect(within(about).queryByRole('button', { name: 'Check now' })).toBeNull()
    // One section at a time: the other two are not drawn.
    expect(within(dialog).queryByText(EXPLAIN)).toBeNull()
    expect(within(dialog).queryByRole('button', { name: 'Theme: System' })).toBeNull()
  })

  it('says it does not know the version when the bridge named none', async () => {
    const dialog = await openSettings({}, null)

    await pick(dialog, ABOUT)

    expect(within(pane(dialog)).getByText(VERSION_UNKNOWN)).toBeTruthy()
  })

  it('draws the pushed update check, with Update dead while a chat is open', async () => {
    const dialog = await openSettings({}, VERSION, {
      open_chats: 1,
      updates: {
        status: 'Ready',
        desk: { current: '0.1.12', latest: '0.1.12', update_available: false },
        pi: { current: '1.5.2', latest: '2.0.5', update_available: true },
        shell: { current: '4.7.0', latest: null, update_available: null },
      },
    })

    await pick(dialog, UPDATES)
    const update = within(pane(dialog)).getByRole('button', { name: 'Update' })

    expect(update.textContent?.trim()).toBe('Update')
    expect((update as HTMLButtonElement).disabled).toBe(true)
    expect(pane(dialog).textContent).toContain('1 chat is open')
  })
})
