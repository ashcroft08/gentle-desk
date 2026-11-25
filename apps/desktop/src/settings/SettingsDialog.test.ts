// The Settings dialog (settings-dialog S2-S6, L4; 0.1.12 S1-S7): a rail of
// eight sections on the left and the selected section's content in the pane on
// the right - the two controls that left the footer (the theme picker and the
// home setting), the Updates and Account panes (0.1.12), Usage and About: the
// app's name and the version this run runs as. Profiles left the dialog
// (profiles-editor T2d, L21-L22): the chat's own Profile tab holds them.
// Its closing keys, its focus and the keyboard walk of the rail are
// SettingsDialog.keys.test.ts's; this file is about what the pane draws.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, within } from '@testing-library/svelte'
import { tick } from 'svelte'
import type {
  PackagesView_Serialize as PackagesView,
  SettingsSnapshot_Serialize as SettingsSnapshot,
  UpdatesView_Serialize as UpdatesView,
} from '../contract/bindings'
import { intents, openSettings, pane, pick, rail, shell, snapshotOf } from '../test-support/settings'
import { pushEvent, resetTauriMock, viewState } from '../test-support/tauri-mock'
import { RAIL_NOTE_ID } from './sections'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The words the dialog draws, quoted here so a change of copy is a change here. */
const TITLE = 'Settings'
const GENERAL = 'General'
const APPEARANCE = 'Appearance'
const HOME = 'Shell home'
const UPDATES = 'Updates'
const ACCOUNT = 'Account'
const USAGE = 'Usage'
const ABOUT = 'About'
/** The rail's nine entries, in the order the dialog draws them (0.9.0 general S1
 * put General first; 0.1.12; 0.2.0; T2d dropped Profiles; notifications T4 added
 * Notifications after Language). */
const RAIL = [GENERAL, APPEARANCE, HOME, UPDATES, ACCOUNT, USAGE, 'Language', 'Notifications', ABOUT]
/** The General pane's own hint, verbatim from the approved artboard's G4 table. */
const GENERAL_HINT = 'What this app needs on the machine, and how to fix what is missing.'
const APPEARANCE_HINT = 'The look of this window.'
const HOME_HINT = 'Which pi this shell runs on.'
const UPDATES_HINT = 'What this machine runs, and whether something newer is out.'
const ACCOUNT_HINT = 'Which providers are signed in, and where to sign in or out.'
const USAGE_HINT = "What each subscription has used, as the shell's own sources report it."
const ABOUT_HINT = 'Which build this is.'
/** The Notifications section's own line (notifications T4, the approved artboard). */
const NOTIFICATIONS = 'Notifications'
const NOTIFICATIONS_HINT = 'A tone and a system notification while the window is not in front.'
/** The generic line the stacked dialog drew under its title: it is gone now. */
const OLD_HINT = 'The look of this window, and the home gentle shell runs on.'
const EXPLAIN =
  'My pi uses the pi installation already on the machine: sign-ins, local models and chats appear in the terminal. Separate creates a clean space at ~/.gentle-shell/agent and touches nothing else.'
/** About (S5): the name, and the version this run runs as. */
const APP_NAME = 'Gentle Shell Desktop'
const VERSION = '0.1.9'
const VERSION_LINE = 'Version 0.1.9'
const VERSION_UNKNOWN = 'Version unknown'

/**
 * The phrase the rail's update dot carries in words (packages-status S8), quoted
 * here so a change of copy is a change here. It is the entry's DESCRIPTION,
 * carried by a hidden line the entry points at: the entry's NAME stays the word
 * it has always been.
 */
const RAIL_WAITING = 'An update is available'

/** The hidden line the waiting entry describes itself with, as it reads. */
function noteOf(dialog: HTMLElement): string | null {
  return dialog.querySelector(`#${RAIL_NOTE_ID}`)?.textContent ?? null
}

/** The Extensions block's slice: unasked unless a test says otherwise. */
function packages(over: Partial<PackagesView> = {}): PackagesView {
  return {
    available: false,
    behind: [],
    unknown: false,
    unchecked: false,
    truncated: false,
    checked: null,
    ...over,
  }
}

/**
 * A check with every row level - nothing waits, so the rail's dot has nothing to
 * say - that a test moves one row (or the packages slice) away from.
 */
function level(over: Partial<UpdatesView> = {}): UpdatesView {
  return {
    status: 'Ready',
    desk: { current: '0.1.12', latest: '0.1.12', update_available: false },
    pi: { current: '1.5.2', latest: '1.5.2', update_available: false },
    shell: { current: '4.8.0', latest: '4.8.0', update_available: false },
    packages: packages({ available: true }),
    ...over,
  }
}

/** A row no read answered for: unknown, never "behind". */
const UNKNOWN_ROW = { current: null, latest: null, update_available: null }

/**
 * The check the Updates section draws with Gentle Shell behind: what the two
 * command lines of updates-by-home S4 are read from.
 */
function updateCheck(): UpdatesView {
  return {
    status: 'Ready',
    desk: { current: '0.1.12', latest: '0.1.12', update_available: false },
    pi: { current: '1.5.2', latest: '2.0.5', update_available: true },
    shell: { current: '4.8.0', latest: '5.0.0', update_available: true },
    packages: { available: false, behind: [], unknown: false, unchecked: false, truncated: false, checked: null },
  }
}

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

describe('the Settings dialog', () => {
  it('opens from the footer gear with the nine sections in its rail and General first', async () => {
    const dialog = await openSettings()

    expect(within(dialog).getByRole('heading', { level: 2, name: TITLE })).toBeTruthy()
    expect(rail(dialog).map((entry) => entry.textContent?.trim())).toEqual(RAIL)
    expect(
      rail(dialog).map((entry) => entry.getAttribute('aria-selected')),
    ).toEqual(['true', 'false', 'false', 'false', 'false', 'false', 'false', 'false', 'false'])
    expect(within(dialog).getAllByRole('tabpanel')).toHaveLength(1)
    expect(within(dialog).getByRole('tabpanel', { name: GENERAL })).toBeTruthy()
    expect(within(pane(dialog)).getByText(GENERAL_HINT)).toBeTruthy()
  })

  it('draws one short hint per section and drops the old generic line', async () => {
    const dialog = await openSettings()

    // The dialog opens on General (S1), whose hint is the line it draws there.
    expect(within(pane(dialog)).getByText(GENERAL_HINT)).toBeTruthy()
    expect(within(dialog).queryByText(OLD_HINT)).toBeNull()

    for (const [name, hint] of [
      [APPEARANCE, APPEARANCE_HINT],
      [HOME, HOME_HINT],
      [UPDATES, UPDATES_HINT],
      [ACCOUNT, ACCOUNT_HINT],
      [USAGE, USAGE_HINT],
      [NOTIFICATIONS, NOTIFICATIONS_HINT],
      [ABOUT, ABOUT_HINT],
    ]) {
      await pick(dialog, name)
      expect(within(pane(dialog)).getByText(hint)).toBeTruthy()
      expect(within(dialog).queryByText(GENERAL_HINT)).toBeNull()
    }
  })

  // profiles-editor T2d (L21, L22): the maintainer removed the Profiles pane
  // from Settings because a dialog made the profiles look global. They live in
  // the chat's own Profile tab now, so the section is gone - entry, hint, pane -
  // and opening the dialog asks the shell for nothing.
  it('no longer offers a Profiles section: profiles live in the chat', async () => {
    const dialog = await openSettings()

    expect(rail(dialog)).toHaveLength(9)
    expect(within(dialog).queryByRole('tab', { name: 'Profiles' })).toBeNull()
    expect(within(dialog).queryByText('Your agent profiles, as the terminal keeps them.')).toBeNull()
    // The pane's read is gone with it: no intent left the dialog.
    expect(intents()).toEqual([])
  })

  it('draws the two 0.1.12 panes in their own sections, one at a time', async () => {
    const dialog = await openSettings()

    await pick(dialog, UPDATES)
    expect(within(pane(dialog)).getByRole('button', { name: 'Check again' })).toBeTruthy()

    await pick(dialog, ACCOUNT)
    expect(within(pane(dialog)).getByRole('button', { name: 'Check again' })).toBeTruthy()
    expect(within(pane(dialog)).getByRole('button', { name: 'Open shell to sign in or out' })).toBeTruthy()
  })

  it('draws one theme card per theme in Appearance, the current one chosen', async () => {
    const dialog = await openSettings()
    // The card opens on General (S1): Appearance is one pick away.
    await pick(dialog, APPEARANCE)
    const chosen = within(pane(dialog)).getByRole('button', { name: 'System' })

    // The grid is the choice: six cards, the pushed theme pressed (ThemeCards
    // owns the rest - the miniature, the check and the intent).
    expect(chosen.getAttribute('aria-pressed')).toBe('true')
    expect(within(pane(dialog)).getByRole('group', { name: 'Theme' })).toBeTruthy()
    expect(within(pane(dialog)).getAllByRole('button')).toHaveLength(6)
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
    // The section the card opened on left the pane, and the rail says which one
    // is drawn.
    expect(within(dialog).queryByText(GENERAL_HINT)).toBeNull()
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
    expect(within(dialog).queryByRole('button', { name: 'System' })).toBeNull()
  })

  it('says it does not know the version when the bridge named none', async () => {
    const dialog = await openSettings({}, null)

    await pick(dialog, ABOUT)

    expect(within(pane(dialog)).getByText(VERSION_UNKNOWN)).toBeTruthy()
  })

  // notifications T4 (S4, S5): the section draws what the PUSH says - the same
  // live slice the theme follows - so it needs no reopen and no restart when the
  // setting moves on the Rust side, and the pane's own words are the artboards'.
  it('draws the notifications mode the push carries, and follows the push', async () => {
    const dialog = await openSettings({}, VERSION, { notifications: 'off' })

    await pick(dialog, NOTIFICATIONS)
    const choices = within(pane(dialog)).getAllByRole('radio')
    const tryIt = () => within(pane(dialog)).getByRole('button', { name: 'Try it' })

    expect(choices.map((choice) => choice.textContent?.trim())).toEqual([
      'When the window is not in front',
      'Off',
    ])
    expect(choices.map((choice) => choice.getAttribute('aria-checked'))).toEqual(['false', 'true'])
    expect(tryIt().getAttribute('aria-disabled')).toBe('true')
    expect(within(pane(dialog)).getByText(/There is nothing to try/)).toBeTruthy()

    // One push, no reopen: the dialog hands the new mode down in the same tick.
    pushEvent(
      'state-changed',
      viewState({ shell: shell(), total_sessions: 1, notifications: 'when-unfocused' }),
    )
    await tick()

    expect(
      within(pane(dialog))
        .getAllByRole('radio')
        .map((choice) => choice.getAttribute('aria-checked')),
    ).toEqual(['true', 'false'])
    expect(tryIt().getAttribute('aria-disabled')).toBe('false')
  })

  it('draws the pushed usage, which no open chat can hold back', async () => {
    const dialog = await openSettings({}, VERSION, {
      // Chats open: the Updates pane's gate counts them, the Usage pane's read
      // reads nothing from a chat and must not care.
      open_chats: 2,
      usage: {
        status: { type: 'Installed' },
        providers: [
          { provider: 'opencode-go', status: 'NoData', plan: null, error: null, limits: [] },
          {
            provider: 'commandcode',
            status: 'Ok',
            plan: 'individual goat',
            error: null,
            limits: [
              {
                name: 'command code',
                limit_reached: false,
                windows: [{ label: '5h', used_percent: 4.7, resets_in: 'resets in 2h' }],
              },
            ],
          },
        ],
      },
    })

    await pick(dialog, USAGE)
    const usage = pane(dialog)

    // The fixture's opencode-go answered no data, so it is not listed: only a
    // provider with a reading reaches the pane (the shell's own rule).
    expect(usage.textContent).toContain('commandcode')
    expect(usage.textContent).not.toContain('opencode-go')
    expect(usage.textContent).toContain('Plan: individual goat')
    expect(usage.textContent).toContain('5%')
    expect(usage.textContent).toContain('resets in 2h')
    // No gate: its one button is alive with two chats open, and no chat line is
    // drawn here.
    const again = within(usage).getByRole('button', { name: 'Check again' })
    expect((again as HTMLButtonElement).disabled).toBe(false)
    expect(usage.textContent).not.toContain('chats are open')
  })

  it('draws the pushed update check, with Update dead while a chat is open', async () => {
    const dialog = await openSettings({}, VERSION, {
      open_chats: 1,
      updates: {
        status: 'Ready',
        desk: { current: '0.1.12', latest: '0.1.12', update_available: false },
        pi: { current: '1.5.2', latest: '2.0.5', update_available: true },
        shell: { current: '4.7.0', latest: null, update_available: null },

        packages: { available: false, behind: [], unknown: false, unchecked: false, truncated: false, checked: null },
      },
    })

    await pick(dialog, UPDATES)
    const update = within(pane(dialog)).getByRole('button', { name: 'Update' })

    expect(update.textContent?.trim()).toBe('Update')
    expect((update as HTMLButtonElement).disabled).toBe(true)
    expect(pane(dialog).textContent).toContain('1 chat is open')
  })

  // updates-by-home S4 (T2): the two hidden facts the pane cannot read for itself
  // arrive from the push - the home (the dialog's own live `shell` slice) and the
  // measured version (the same `updates.shell.latest` the check carries).
  it('hands the Updates pane the Mi pi commands the push chose', async () => {
    const dialog = await openSettings({ choice: 'link' }, VERSION, { updates: updateCheck() })

    await pick(dialog, UPDATES)
    const lines = [...pane(dialog).querySelectorAll('[data-command]')].map(
      (line) => line.textContent?.trim(),
    )

    expect(lines).toEqual([
      'pi update',
      'pi install npm:gentle-pi@5.0.0',
      'pi update --extensions',
    ])
  })

  it('keeps the Separado commands in a dialog left on Separado', async () => {
    const dialog = await openSettings({ choice: 'isolated' }, VERSION, { updates: updateCheck() })

    await pick(dialog, UPDATES)

    expect(pane(dialog).textContent).toContain('gentle-shell upgrade')
    expect(pane(dialog).textContent).toContain('gentle-shell update --extensions')
    expect(pane(dialog).textContent).not.toContain('pi install')
  })
})

// packages-status S8, T5 (L2): the gear's dot already promises something, but
// once Settings is open - and it opens on General - nothing said WHERE to go. So
// the "Actualizaciones" entry itself wears the same dot while `updateWaiting`
// (update/dot.ts, read here over the dialog's LIVE updates) says something waits
// - and only that entry does.
describe('the rail that points at Updates (packages-status S8)', () => {
  /** The dot a rail entry draws, or null when it draws none. */
  function dotOf(entry: HTMLElement): Element | null {
    return entry.querySelector('.dot')
  }

  it('lights the Updates entry when Gentle Shell is behind, and no other entry', async () => {
    const dialog = await openSettings({}, VERSION, {
      updates: level({ shell: { current: '4.8.0', latest: '5.0.0', update_available: true } }),
    })

    // The entry is still found by the name it has always had: the dot is a mark,
    // never a word added to the tab.
    const updatesTab = within(dialog).getByRole('tab', { name: UPDATES })
    expect(dotOf(updatesTab)).not.toBeNull()
    expect(updatesTab.textContent?.trim()).toBe(UPDATES)

    // The state has a second channel, so the promise is never the colour alone:
    // the entry describes itself with the hidden line, which adds no word to the
    // name it is looked up by.
    expect(updatesTab.getAttribute('aria-describedby')).toBe(RAIL_NOTE_ID)
    expect(noteOf(dialog)).toBe(RAIL_WAITING)

    // And the promise points at ONE entry: every other one stays bare.
    for (const entry of rail(dialog)) {
      if (entry === updatesTab) continue
      expect(dotOf(entry), entry.textContent ?? '').toBeNull()
      expect(entry.getAttribute('aria-describedby'), entry.textContent ?? '').toBeNull()
    }
  })

  it('lights it for a behind pi row too, the same rule the gear reads', async () => {
    const dialog = await openSettings({}, VERSION, {
      updates: level({ pi: { current: '1.5.2', latest: '2.0.5', update_available: true } }),
    })

    expect(dotOf(within(dialog).getByRole('tab', { name: UPDATES }))).not.toBeNull()
  })

  it('lights it for a package pi reports behind', async () => {
    const dialog = await openSettings({}, VERSION, {
      updates: level({
        packages: packages({ available: true, behind: [{ name: '@estebanforge/pi-antigravity-bridge' }] }),
      }),
    })

    expect(dotOf(within(dialog).getByRole('tab', { name: UPDATES }))).not.toBeNull()
  })

  it('stays bare while everything is current', async () => {
    const dialog = await openSettings({}, VERSION, { updates: level() })

    const updatesTab = within(dialog).getByRole('tab', { name: UPDATES })
    expect(dotOf(updatesTab)).toBeNull()
    expect(updatesTab.getAttribute('aria-describedby')).toBeNull()
  })

  it('stays bare for the states that are not a promise: unknown and unasked', async () => {
    // Rows no read answered for, and a packages read that could not run: nothing
    // is behind, so there is nothing to point at.
    const dialog = await openSettings({}, VERSION, {
      updates: level({ pi: UNKNOWN_ROW, shell: UNKNOWN_ROW, packages: packages({ unknown: true }) }),
    })

    expect(dotOf(within(dialog).getByRole('tab', { name: UPDATES }))).toBeNull()
  })

  it('stays bare when the only waiting version is the one the user skipped', async () => {
    // The skip rule is the gear's own (0.8.0 settings-polish S2): the user
    // already said no to 2.0.5, so the entry promises nothing either.
    const dialog = await openSettings({}, VERSION, {
      updates: level({
        skipped: '2.0.5',
        pi: { current: '1.5.2', latest: '2.0.5', update_available: true },
      }),
    })

    expect(dotOf(within(dialog).getByRole('tab', { name: UPDATES }))).toBeNull()
  })

  it('reads the same live updates the Updates pane draws: the push until the open lands', async () => {
    // The dot reads `read.snapshot?.updates ?? updates`, the one expression the
    // pane draws: the pushed slice until the dialog's one read answers, and that
    // answer afterwards - so the rail and the pane can never say different things.
    let settle: (snapshot: SettingsSnapshot) => void = () => {}
    const held = new Promise<SettingsSnapshot>((resolve) => {
      settle = resolve
    })
    const dialog = await openSettings(
      {},
      VERSION,
      { updates: level({ shell: { current: '4.8.0', latest: '5.0.0', update_available: true } }) },
      held,
    )

    expect(dotOf(within(dialog).getByRole('tab', { name: UPDATES }))).not.toBeNull()

    settle(snapshotOf(viewState({ shell: shell(), total_sessions: 1, updates: level() })))
    await vi.waitFor(() => {
      expect(dotOf(within(dialog).getByRole('tab', { name: UPDATES }))).toBeNull()
    })

    // The answer is level, so the promise is gone - the same tick the pane's own
    // data changed, with no reopen - and so is its description.
    const updatesTab = within(dialog).getByRole('tab', { name: UPDATES })
    expect(dotOf(updatesTab)).toBeNull()
    expect(updatesTab.getAttribute('aria-describedby')).toBeNull()
  })
})
