// The Settings dialog's ONE read (settings-snapshot): the card asks for the
// plain data of every section once, when it opens, and every page draws from that
// one object. What a section draws is SettingsDialog.test.ts's, and each pane's
// own rendering is its own file's; this file is about the read - how often it
// runs, what the dialog draws until it lands, and what a section asks for when
// the user presses "Check again".
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, within } from '@testing-library/svelte'
import type { SettingsSnapshot_Serialize as SettingsSnapshot } from '../contract/bindings'
import { intents, openSettings, pane, pick, snapshotOf } from '../test-support/settings'
import {
  deferred,
  handlerArgs,
  resetTauriMock,
  setHandler,
  viewState,
} from '../test-support/tauri-mock'
import { usageCopy } from './usage-copy'
import { accountCopy } from './account-copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The rail's eight entries, in the order the dialog draws them. */
const RAIL = [
  'Appearance',
  'Shell home',
  'Updates',
  'Account',
  'Usage',
  'Language',
  'Notifications',
  'About',
]
const USAGE = 'Usage'
const ACCOUNT = 'Account'
const CLOSE = 'Close'
/** The quiet line a pane draws while a load is on its way. */
const READING = usageCopy('en').reading

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

/** Lets every microtask and timer the card scheduled settle. */
async function settle(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, 0))
}

/** How many times the dialog asked for its one load. */
function loads(): number {
  return handlerArgs('settings_snapshot').length
}

/** One load's answer, built from the slice the push carried. */
function answered(over: Partial<SettingsSnapshot> = {}): SettingsSnapshot {
  return { ...snapshotOf(viewState()), ...over }
}

describe('the Settings dialog and its one read', () => {
  it('asks for the load once when it opens, and not once per section', async () => {
    const dialog = await openSettings()

    expect(loads()).toBe(1)
    // Walking the whole rail reads nothing: every page draws from the one object.
    for (const entry of RAIL) {
      await pick(dialog, entry)
    }
    expect(loads()).toBe(1)
    // And no pane asks Rust for a slice of its own any more.
    expect(intents()).toEqual([])
  })

  it('asks again on the next open, once per open', async () => {
    const dialog = await openSettings()
    expect(loads()).toBe(1)

    await fireEvent.click(within(dialog).getByRole('button', { name: CLOSE }))
    await vi.waitFor(() => expect(dialog.isConnected).toBe(false))

    await fireEvent.click(await within(document.body).findByRole('button', { name: 'Settings' }))
    await within(document.body).findByRole('dialog')

    await vi.waitFor(() => expect(loads()).toBe(2))
  })

  it('draws what the push carried until the load lands, then the loaded data', async () => {
    const held = deferred<SettingsSnapshot>()
    const dialog = await openSettings({}, '0.1.9', {
      // The push knows nothing about the usage yet: the pane says it is reading.
      usage: { status: { type: 'Unknown' }, providers: [] },
    }, held.promise)

    await pick(dialog, USAGE)
    expect(within(pane(dialog)).getByText(READING)).toBeTruthy()

    held.resolve(
      answered({
        usage: {
          status: { type: 'Installed' },
          providers: [
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
      }),
    )

    await vi.waitFor(() => expect(pane(dialog).textContent).toContain('individual goat'))
    expect(pane(dialog).textContent).toContain('5%')
    // The one object answered for the pane: no second read was needed.
    expect(loads()).toBe(1)
  })

  it('keeps drawing the pushed content when the load does not answer at all', async () => {
    const dialog = await openSettings({}, '0.1.9', {
      auth: {
        status: { type: 'Ready' },
        providers: [{ name: 'anthropic', status: 'Ready', reason: null }],
      },
    })
    // A load that fails is the command's own refusal, and it leaves the card
    // exactly as it was: no crash, no empty pane.
    setHandler('settings_snapshot', () => Promise.reject(new Error('the load refused')))

    await pick(dialog, ACCOUNT)

    expect(within(pane(dialog)).getByText('anthropic')).toBeTruthy()
    expect(within(pane(dialog)).getByText(accountCopy('en').connected)).toBeTruthy()
  })

  it('asks for one more load when a section checks again, and says it is reading', async () => {
    const dialog = await openSettings({}, '0.1.9', {
      usage: { status: { type: 'Missing', data: 'not installed' }, providers: [] },
    })
    await pick(dialog, USAGE)
    expect(loads()).toBe(1)

    const held = deferred<SettingsSnapshot>()
    setHandler('settings_snapshot', () => held.promise)
    await fireEvent.click(within(pane(dialog)).getByRole('button', { name: 'Check again' }))

    // The ask is visible: the same quiet line the first read drew.
    expect(loads()).toBe(2)
    await vi.waitFor(() => expect(pane(dialog).textContent).toContain(READING))

    held.resolve(
      answered({ usage: { status: { type: 'Installed' }, providers: [] } }),
    )
    await vi.waitFor(() => expect(pane(dialog).textContent).not.toContain(READING))
    // Still one path: the click was the load, not an intent of its own.
    expect(intents()).toEqual([])
  })

  it('coalesces a burst of asks into one load after the one in flight', async () => {
    const dialog = await openSettings()
    await pick(dialog, ACCOUNT)
    expect(loads()).toBe(1)

    const held = deferred<SettingsSnapshot>()
    setHandler('settings_snapshot', () => held.promise)
    // A focus, a watcher notice and a click, all while the first ask is out.
    window.dispatchEvent(new Event('focus'))
    await fireEvent.click(within(pane(dialog)).getByRole('button', { name: 'Check again' }))
    await vi.waitFor(() => expect(loads()).toBe(2))

    held.resolve(answered())
    // One follow-up for the burst, never one per ask.
    await vi.waitFor(() => expect(loads()).toBe(3))
  })

  it('starts nothing after the card closes, not even an ask it had queued', async () => {
    const dialog = await openSettings()
    await pick(dialog, ACCOUNT)
    expect(loads()).toBe(1)

    const held = deferred<SettingsSnapshot>()
    setHandler('settings_snapshot', () => held.promise)
    window.dispatchEvent(new Event('focus'))
    await vi.waitFor(() => expect(loads()).toBe(2))
    // A second ask while the first is out: queued, not started.
    await fireEvent.click(within(pane(dialog)).getByRole('button', { name: 'Check again' }))
    await settle()
    expect(loads()).toBe(2)

    await fireEvent.click(within(dialog).getByRole('button', { name: CLOSE }))
    await vi.waitFor(() => expect(dialog.isConnected).toBe(false))

    held.resolve(answered())
    await settle()

    // No check runs with Settings closed (rpc-settings-sync T3b-2).
    expect(loads()).toBe(2)
  })
})
