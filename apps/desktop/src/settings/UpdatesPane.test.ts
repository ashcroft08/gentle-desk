// The Updates pane (0.1.12, S2, S3, S4, S7): ONE status card, drawn from the
// check the dialog's one load ran (settings-snapshot). The Update button is
// the terminal's, and it is dead while any chat is open (S4) with the reason
// said in place; "Check again" only ever asks the dialog for another load. No
// test here reaches a registry: the check is Rust's and arrives in the object.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, within } from '@testing-library/svelte'
import type { UpdatesView_Serialize as UpdatesView } from '../contract/bindings'
import { handlerArgs, resetTauriMock, setHandler } from '../test-support/tauri-mock'
import UpdatesPane from './UpdatesPane.svelte'
import { updatesCopy } from './updates-copy'

/** The pane's own words in English, read through the dictionary (0.2.0, D4). */
const copy = updatesCopy('en')

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => null)
  setHandler('open_terminal', () => null)
})

/** The check the user's machine really answers: pi 1.5.2 against 2.0.5. */
function updates(over: Partial<UpdatesView> = {}): UpdatesView {
  return {
    status: 'Ready',
    desk: { current: '0.1.12', latest: '0.1.12', update_available: false },
    pi: { current: '1.5.2', latest: '2.0.5', update_available: true },
    shell: { current: '4.7.0', latest: null, update_available: null },
    ...over,
  }
}

/** Everything current: the same rows with nothing newer published. */
function current(): UpdatesView {
  return updates({
    pi: { current: '2.0.5', latest: '2.0.5', update_available: false },
    shell: { current: '4.7.0', latest: null, update_available: null },
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

describe('the Updates pane', () => {
  it('draws one card: the tag, the pi headline, the quiet versions and the two buttons', () => {
    const pane = draw()

    expect(within(pane).getByText(copy.updateAvailable)).toBeTruthy()
    expect(pane.querySelector('.tag')?.classList.contains('warn')).toBe(true)
    expect(pane.querySelector('.headline')?.textContent?.replace(/\s+/g, ' ')).toBe(
      'pi 1.5.2 \u2192 2.0.5',
    )
    // The quiet versions row says what this machine runs, unknown included.
    expect(pane.querySelector('.versions')?.textContent?.replace(/\s+/g, ' ')).toContain(
      'This app 0.1.12 \u00b7 pi 1.5.2 \u00b7 shell 4.7.0',
    )
    expect(within(pane).getByRole('button', { name: copy.update })).toBeTruthy()
    expect(within(pane).getByRole('button', { name: copy.checkAgain })).toBeTruthy()
  })

  it('says everything is up to date, with no Update to press', () => {
    const pane = draw(current())

    expect(within(pane).getByText(copy.upToDate)).toBeTruthy()
    expect(pane.querySelector('.tag')?.classList.contains('ok')).toBe(true)
    expect(within(pane).queryByRole('button', { name: copy.update })).toBeNull()
    expect(within(pane).getByRole('button', { name: copy.checkAgain })).toBeTruthy()
  })

  it('always offers Update extensions, with the note that says what it updates', () => {
    // Nothing with the check drives this button: extensions are updated by the
    // shell's own updater and there is no way to know which are behind, so it
    // is offered with everything up to date too.
    const pane = draw(current())
    const extensions = within(pane).getByRole('button', {
      name: copy.updateExtensions,
    }) as HTMLButtonElement

    expect(extensions.disabled).toBe(false)
    expect(within(pane).getByText(copy.updateExtensionsInTerminal)).toBeTruthy()
    // It sits next to Check again, in the card's own action row.
    expect(extensions.closest('.acts')?.textContent).toContain(copy.checkAgain)
  })

  it('offers Update extensions before the check has answered, and never checks', () => {
    const pane = draw(updates({ status: 'Unknown' }))

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

  it('draws the checking line without taking the pane away', () => {
    const pane = draw(updates({ status: 'Checking' }))

    expect(within(pane).getByText(copy.checking)).toBeTruthy()
    // The versions it already showed stay, and Check again stays usable.
    expect(pane.querySelector('.versions')).toBeTruthy()
    expect(within(pane).getByRole('button', { name: copy.checkAgain })).toBeTruthy()
  })

  it('puts the check before the two terminal actions, with no accent on it', () => {
    // The two terminal buttons open a window that now stays open; the check is
    // the cheap one, so it sits first and wears no accent: a hand landing on
    // the middle of the row must not start an update.
    const pane = draw()
    const row = pane.querySelector('.acts') as HTMLElement
    const names = [...row.querySelectorAll('button')].map((button) =>
      button.textContent?.replace(/\s+/g, ' ').trim(),
    )

    expect(names).toEqual([copy.checkAgain, copy.update, copy.skipVersion, copy.updateExtensions])
    const check = within(pane).getByRole('button', { name: copy.checkAgain })
    expect(check.classList.contains('accent')).toBe(false)
    expect([...row.querySelectorAll('.accent')].map((button) => button.textContent?.trim())).toEqual([
      copy.update,
    ])
  })

  it('checks nothing before the app has checked, and never calls the registry', () => {
    const again = ask()
    const pane = draw(updates({ status: 'Unknown' }), 0, false, again)
    expect(pane.querySelector('.card')).toBeNull()
    expect(within(pane).getByText(copy.notChecked)).toBeTruthy()
    // Nothing of its own ran: the dialog's one load is where the check comes from.
    expect(again).not.toHaveBeenCalled()
  })

  it("says the check is running while the user's own re-check is on its way", () => {
    // The card the last load answered, and a re-check in flight: the checking
    // line with the versions it already knew, so the click is never silent.
    const pane = draw(updates(), 0, true)

    expect(within(pane).getByText(copy.checking)).toBeTruthy()
    expect(pane.querySelector('.updates')).toBeNull()
    expect(pane.textContent).toContain('pi 1.5.2')
  })

  it('disables Update and says why while any chat is open', () => {
    const pane = draw(updates(), 2)
    const update = within(pane).getByRole('button', { name: copy.update }) as HTMLButtonElement

    expect(update.disabled).toBe(true)
    const hint = within(pane).getByText(copy.gateHint(2))
    expect(hint.getAttribute('role')).toBe('status')
    expect(update.getAttribute('aria-describedby')).toBe(hint.id)
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

    await fireEvent.click(within(pane).getByRole('button', { name: copy.checkAgain }))

    expect(again).toHaveBeenCalledTimes(1)
    // A check, never an install: no terminal window opens and no intent leaves.
    expect(handlerArgs('dispatch_intent')).toEqual([])
    expect(handlerArgs('open_terminal')).toEqual([])
  })
  it('offers Skip this version beside the one on offer, plain and unaccented', async () => {
    const again = ask()
    const pane = draw(updates(), 0, false, again)
    const skip = within(pane).getByRole('button', { name: copy.skipVersion })

    expect(skip.classList.contains('accent')).toBe(false)
    await fireEvent.click(skip)
    // The version the check offered is the one the intent names, and the pane
    // asks for the one read again so the card shows the skip it just made.
    expect(handlerArgs('dispatch_intent')).toEqual([
      { intent: { type: 'SkipUpdate', data: { version: '2.0.5' } } },
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

  it('offers Stop skipping when the version on offer is the skipped one', async () => {
    const again = ask()
    const pane = draw(updates({ skipped: '2.0.5' }), 0, false, again)

    // The check the user asked for says the truth: the version is published, and
    // it is the one they skipped - so the way back is what is offered.
    expect(within(pane).queryByRole('button', { name: copy.skipVersion })).toBeNull()
    const stop = within(pane).getByRole('button', { name: copy.stopSkipping })
    expect(stop.classList.contains('accent')).toBe(false)
    await fireEvent.click(stop)
    expect(handlerArgs('dispatch_intent')).toEqual([{ intent: { type: 'UnskipUpdate' } }])
    expect(again).toHaveBeenCalledTimes(1)
  })
})
