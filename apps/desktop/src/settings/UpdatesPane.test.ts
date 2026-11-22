// The Updates pane (0.1.12, S2, S3, S4, S7; 0.8.1 settings-polish S6, S8): TWO
// cards drawn from the check the dialog's one load ran (settings-snapshot) - the
// three rows with their tag, their sentences, their actions and the last time it
// was read, and the extensions block, which no data drives. The terminal buttons
// are the shell's, and they are dead while any chat is open (S4) with the reason
// said in place; "Check again" only ever asks the dialog for another load. No test
// here reaches a registry: the check is Rust's and arrives in the object. The rows
// themselves are UpdateRows.test.ts's.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, within } from '@testing-library/svelte'
import type { UpdatesView_Serialize as UpdatesView } from '../contract/bindings'
import { handlerArgs, resetTauriMock, setHandler } from '../test-support/tauri-mock'
import UpdatesPane from './UpdatesPane.svelte'
import { updatesCopy } from './updates-copy'

/** The pane's own words in English, read through the dictionary (0.2.0, D4). */
const copy = updatesCopy('en')

/** The instant every check in this file is dated from, and the clock it is read at. */
const CHECKED_MS = 1_700_000_000_000
const NOW_MS = CHECKED_MS + 3 * 60_000

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

afterEach(() => {
  cleanup()
  vi.restoreAllMocks()
})

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => null)
  setHandler('open_terminal', () => null)
  // The pane words its age from the clock it is drawn at; three minutes after the
  // check is the house's own "3m".
  vi.spyOn(Date, 'now').mockReturnValue(NOW_MS)
})

/** The check the user's machine really answers: pi and Gentle Shell both behind. */
function updates(over: Partial<UpdatesView> = {}): UpdatesView {
  return {
    status: 'Ready',
    desk: { current: '0.1.12', latest: '0.1.12', update_available: false },
    pi: { current: '1.5.2', latest: '2.0.5', update_available: true },
    shell: { current: '4.8.0', latest: '4.9.2', update_available: true },

    packages: { available: false, behind: [], unknown: false, truncated: false, checked: null },
    checked_ms: CHECKED_MS,
    ...over,
  }
}

/** Everything current: the same rows with nothing newer published. */
function current(): UpdatesView {
  return updates({
    pi: { current: '2.0.5', latest: '2.0.5', update_available: false },
    shell: { current: '4.9.2', latest: '4.9.2', update_available: false },

    packages: { available: false, behind: [], unknown: false, truncated: false, checked: null },
  })
}

/** One ask for the dialog's load, as the pane sends it. */
function ask(): () => void {
  return vi.fn()
}

function draw(
  view: UpdatesView = updates(),
  openChats = 0,
  refreshing = false,
  onRefresh: () => void = ask(),
): HTMLElement {
  const { container } = render(UpdatesPane, {
    props: { updates: view, openChats, refreshing, onRefresh },
  })
  return container as HTMLElement
}

/** One row's element, by the key it is drawn under. */
function row(pane: HTMLElement, key: 'app' | 'pi' | 'shell'): HTMLElement {
  const found = pane.querySelector(`[data-row="${key}"]`)
  if (!(found instanceof HTMLElement)) throw new Error(`no ${key} row was drawn`)
  return found
}

describe('the Updates pane', () => {
  it('draws the three rows, the tag, the note and when it last checked', () => {
    const pane = draw()

    expect(within(pane).getByText(copy.updateAvailable)).toBeTruthy()
    expect(pane.querySelector('.tag')?.classList.contains('warn')).toBe(true)
    expect(row(pane, 'pi').querySelector('.mono')?.textContent).toBe('1.5.2 \u2192 2.0.5')
    expect(within(pane).getByRole('button', { name: copy.update })).toBeTruthy()
    expect(within(pane).getByRole('button', { name: copy.updateShell })).toBeTruthy()
    // The two terminal buttons are the note's subject, word by word.
    expect(pane.querySelector('.note')?.textContent).toBe(copy.updateInTerminal)
    // D11: the metadata line is when it was read, through the house's own age.
    expect(pane.querySelector('.versions')?.textContent).toBe('Last checked: 3m ago')
  })

  it('says everything is up to date, with no terminal action to press', () => {
    const pane = draw(current())

    expect(within(pane).getByText(copy.upToDate)).toBeTruthy()
    expect(pane.querySelector('.tag')?.classList.contains('ok')).toBe(true)
    expect(within(pane).queryByRole('button', { name: copy.update })).toBeNull()
    expect(within(pane).queryByRole('button', { name: copy.updateShell })).toBeNull()
    expect(pane.querySelector('.note')).toBeNull()
    expect(within(pane).getByRole('button', { name: copy.checkAgain })).toBeTruthy()
  })

  it('never says up to date while a row could not be read, and says which and why', () => {
    // S8, D12: the tag goes quiet and the row carries the short clause. Not one
    // word of the client's message reaches the screen.
    const pane = draw(
      updates({
        pi: { current: '2.0.5', latest: '2.0.5', update_available: false },
        shell: {
          current: '4.8.0',
          latest: null,
          update_available: null,
          reason: 'Offline',
        },
      }),
    )

    expect(within(pane).getByText(copy.notCheckedTag)).toBeTruthy()
    expect(pane.querySelector('.tag')?.classList.contains('quiet')).toBe(true)
    expect(within(pane).queryByText(copy.upToDate)).toBeNull()
    expect(row(pane, 'shell').querySelector('.reason')?.textContent).toBe(
      'Could not check the latest gentle-shell version on npm: offline.',
    )
  })

  it('opens the shell upgrade in its own terminal window', async () => {
    const pane = draw()

    await fireEvent.click(within(pane).getByRole('button', { name: copy.updateShell }))
    expect(handlerArgs('open_terminal')).toEqual([{ action: 'UpgradeShell' }])
    expect(handlerArgs('dispatch_intent')).toEqual([])
  })

  it('always offers Update extensions, in a block that says why it has no state', () => {
    // Nothing with the check drives this button: extensions are updated by the
    // shell's own updater and there is no way to know which are behind, so it is
    // offered with everything up to date too, in a card of its own (D14).
    const pane = draw(current())
    const extensions = within(pane).getByRole('button', {
      name: copy.updateExtensions,
    }) as HTMLButtonElement
    const block = pane.querySelector('.card.ext')

    expect(extensions.disabled).toBe(false)
    expect(block?.contains(extensions)).toBe(true)
    expect(block?.querySelector('.quiet')?.textContent).toBe(copy.extensionsNote)
    // It is not inside the card of the three rows: no version drives it.
    expect(pane.querySelector('.card.updates')?.contains(extensions)).toBe(false)
  })

  it('offers Update extensions before the check has answered, and never checks', () => {
    const pane = draw(updates({ status: 'Unknown', checked_ms: undefined }))

    expect(within(pane).getByRole('button', { name: copy.updateExtensions })).toBeTruthy()
    expect(within(pane).getByRole('button', { name: copy.checkAgain })).toBeTruthy()
    expect(handlerArgs('dispatch_intent')).toEqual([])
  })

  it('opens the extensions update in a terminal window with every chat closed', async () => {
    const pane = draw(current())

    await fireEvent.click(within(pane).getByRole('button', { name: copy.updateExtensions }))
    expect(handlerArgs('open_terminal')).toEqual([{ action: 'UpdateExtensions' }])
    expect(handlerArgs('dispatch_intent')).toEqual([])
  })

  it('keeps Update extensions dead while a chat is open, and says why', () => {
    const pane = draw(updates(), 1)
    const extensions = within(pane).getByRole('button', {
      name: copy.updateExtensions,
    }) as HTMLButtonElement
    const hint = within(pane).getByText(copy.gateHint(1))

    expect(extensions.disabled).toBe(true)
    expect(hint.getAttribute('role')).toBe('status')
    expect(extensions.getAttribute('aria-describedby')).toBe(hint.id)
    // Nothing opens from a dead button.
    expect(handlerArgs('open_terminal')).toEqual([])
  })

  it('draws the checking line without taking the pane away, and keeps the last time', () => {
    const pane = draw(updates({ status: 'Checking' }))

    expect(within(pane).getByText(copy.checking)).toBeTruthy()
    // The last read it knew is a datum, not a state: it stays.
    expect(pane.querySelector('.versions')?.textContent).toBe('Last checked: 3m ago')
    expect(within(pane).getByRole('button', { name: copy.checkAgain })).toBeTruthy()
    expect(within(pane).getByRole('button', { name: copy.updateExtensions })).toBeTruthy()
  })

  it('says the check is running while the user\'s own re-check is on its way', () => {
    const pane = draw(updates(), 0, true)

    expect(within(pane).getByText(copy.checking)).toBeTruthy()
    expect(pane.querySelector('.card.updates')).toBeNull()
    expect(pane.querySelector('.versions')?.textContent).toBe('Last checked: 3m ago')
  })

  it('checks nothing before the app has checked, and never calls the registry', () => {
    const again = ask()
    const pane = draw(updates({ status: 'Unknown', checked_ms: undefined }), 0, false, again)

    expect(pane.querySelector('.card.updates')).toBeNull()
    // D13: the quiet "not checked yet" line is gone - the last-checked line says
    // it, once, and it is the one that also serves after a check.
    expect(pane.querySelector('.quiet')?.textContent).not.toBe('Not checked yet.')
    expect(within(pane).getByText(copy.checkedNever)).toBeTruthy()
    expect(pane.querySelector('.versions')?.textContent).toBe('Last checked: never')
    // Nothing of its own ran: the dialog's one load is where the check comes from.
    expect(again).not.toHaveBeenCalled()
  })

  it('disables the terminal actions and says why while any chat is open', () => {
    const pane = draw(updates(), 2)
    const update = within(pane).getByRole('button', { name: copy.update }) as HTMLButtonElement
    const upgrade = within(pane).getByRole('button', {
      name: copy.updateShell,
    }) as HTMLButtonElement

    expect(update.disabled).toBe(true)
    expect(upgrade.disabled).toBe(true)
    const hint = within(pane).getByText(copy.gateHint(2))
    expect(hint.getAttribute('role')).toBe('status')
    expect(update.getAttribute('aria-describedby')).toBe(hint.id)
    // The gate explains the buttons above it instead of the note they would have.
    expect(pane.querySelector('.note')).toBeNull()
  })

  it('lets Update run with every chat closed, in a terminal window', async () => {
    const pane = draw()
    const update = within(pane).getByRole('button', { name: copy.update }) as HTMLButtonElement

    expect(update.disabled).toBe(false)
    expect(within(pane).queryByText(copy.gateHint(2))).toBeNull()
    await fireEvent.click(update)
    expect(handlerArgs('open_terminal')).toEqual([{ action: 'Update' }])
  })

  it('asks for a check again, and only ever a check', async () => {
    const again = ask()
    const pane = draw(updates(), 0, false, again)
    const check = within(pane).getByRole('button', { name: copy.checkAgain })
    // The cheap one wears no accent: the terminal buttons are the loud ones, and
    // the check is the only button in its row that can never install anything.
    expect(check.classList.contains('accent')).toBe(false)

    await fireEvent.click(check)

    expect(again).toHaveBeenCalledTimes(1)
    // A check, never an install: no terminal window opens and no intent leaves.
    expect(handlerArgs('dispatch_intent')).toEqual([])
    expect(handlerArgs('open_terminal')).toEqual([])
  })

  it('offers Skip this version on the row whose version is the newest (D9)', async () => {
    const again = ask()
    const pane = draw(updates(), 0, false, again)

    // 4.9.2 is the greatest of the two offered versions, so that row carries it.
    expect(within(row(pane, 'pi')).queryByRole('button', { name: copy.skipVersion })).toBeNull()
    const skip = within(row(pane, 'shell')).getByRole('button', { name: copy.skipVersion })
    expect(skip.classList.contains('accent')).toBe(false)
    await fireEvent.click(skip)
    // The version that row publishes is the one the intent names, and the pane
    // asks for the one read again so the card shows the skip it just made.
    expect(handlerArgs('dispatch_intent')).toEqual([
      { intent: { type: 'SkipUpdate', data: { version: '4.9.2' } } },
    ])
    expect(again).toHaveBeenCalledTimes(1)
  })

  it('draws no Skip this version while nothing newer is offered, and none before a check', () => {
    const upToDate = draw(current())
    expect(within(upToDate).queryByRole('button', { name: copy.skipVersion })).toBeNull()
    expect(within(upToDate).queryByRole('button', { name: copy.stopSkipping })).toBeNull()

    const notChecked = draw(updates({ status: 'Unknown' }))
    expect(within(notChecked).queryByRole('button', { name: copy.skipVersion })).toBeNull()
  })

  it('offers Stop skipping on the skipped row, with the sentence that says what it means', async () => {
    const again = ask()
    const pane = draw(
      updates({
        pi: { current: '2.0.5', latest: '2.0.5', update_available: false },
        skipped: '4.9.2',
      }),
      0,
      false,
      again,
    )

    // The check the user asked for says the truth: the version is published, and
    // it is the one they skipped - so the way back is what is offered.
    expect(within(pane).queryByRole('button', { name: copy.skipVersion })).toBeNull()
    const stop = within(row(pane, 'shell')).getByRole('button', { name: copy.stopSkipping })
    expect(stop.classList.contains('accent')).toBe(false)
    expect(row(pane, 'shell').querySelector('.reason')?.textContent).toBe(
      copy.skippedNote('4.9.2'),
    )
    await fireEvent.click(stop)
    expect(handlerArgs('dispatch_intent')).toEqual([{ intent: { type: 'UnskipUpdate' } }])
    expect(again).toHaveBeenCalledTimes(1)
  })
})
