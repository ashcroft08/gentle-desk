// The Updates pane (0.1.12, S2, S3, S4, S7): ONE status card, drawn from the
// check the app ran when it opened. The Update button is the terminal's, and it
// is dead while any chat is open (S4) with the reason said in place; "Check
// again" only ever asks for a check. No test here reaches a registry: the check
// is Rust's and arrives as the pushed slice.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, within } from '@testing-library/svelte'
import type { UpdatesView_Serialize as UpdatesView } from '../contract/bindings'
import { handlerArgs, resetTauriMock, setHandler } from '../test-support/tauri-mock'
import UpdatesPane from './UpdatesPane.svelte'
import {
  CHECKING,
  CHECK_AGAIN,
  UPDATE,
  UPDATE_AVAILABLE,
  UP_TO_DATE,
  gateHint,
} from './updates-copy'

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

function draw(view: UpdatesView = updates(), openChats = 0): HTMLElement {
  const { container } = render(UpdatesPane, { props: { updates: view, openChats } })
  return container as HTMLElement
}

function intents(): unknown[] {
  return handlerArgs('dispatch_intent').map((args) => args.intent)
}

describe('the Updates pane', () => {
  it('draws one card: the tag, the pi headline, the quiet versions and the two buttons', () => {
    const pane = draw()

    expect(within(pane).getByText(UPDATE_AVAILABLE)).toBeTruthy()
    expect(pane.querySelector('.tag')?.classList.contains('warn')).toBe(true)
    expect(pane.querySelector('.headline')?.textContent?.replace(/\s+/g, ' ')).toBe(
      'pi 1.5.2 \u2192 2.0.5',
    )
    // The quiet versions row says what this machine runs, unknown included.
    expect(pane.querySelector('.versions')?.textContent?.replace(/\s+/g, ' ')).toContain(
      'This app 0.1.12 \u00b7 pi 1.5.2 \u00b7 shell 4.7.0',
    )
    expect(within(pane).getByRole('button', { name: UPDATE })).toBeTruthy()
    expect(within(pane).getByRole('button', { name: CHECK_AGAIN })).toBeTruthy()
  })

  it('says everything is up to date, with no Update to press', () => {
    const pane = draw(current())

    expect(within(pane).getByText(UP_TO_DATE)).toBeTruthy()
    expect(pane.querySelector('.tag')?.classList.contains('ok')).toBe(true)
    expect(within(pane).queryByRole('button', { name: UPDATE })).toBeNull()
    expect(within(pane).getByRole('button', { name: CHECK_AGAIN })).toBeTruthy()
  })

  it('draws the checking line without taking the pane away', () => {
    const pane = draw(updates({ status: 'Checking' }))

    expect(within(pane).getByText(CHECKING)).toBeTruthy()
    // The versions it already showed stay, and Check again stays usable.
    expect(pane.querySelector('.versions')).toBeTruthy()
    expect(within(pane).getByRole('button', { name: CHECK_AGAIN })).toBeTruthy()
  })

  it('checks nothing before the app has checked, and never calls the registry', () => {
    const pane = draw(updates({ status: 'Unknown' }))
    expect(pane.querySelector('.card')).toBeNull()
    expect(intents()).toEqual([])
  })

  it('disables Update and says why while any chat is open', () => {
    const pane = draw(updates(), 2)
    const update = within(pane).getByRole('button', { name: UPDATE }) as HTMLButtonElement

    expect(update.disabled).toBe(true)
    const hint = within(pane).getByText(gateHint(2))
    expect(hint.getAttribute('role')).toBe('status')
    expect(update.getAttribute('aria-describedby')).toBe(hint.id)
  })

  it('lets Update run with every chat closed, in a terminal window', async () => {
    const pane = draw()
    const update = within(pane).getByRole('button', { name: UPDATE }) as HTMLButtonElement

    expect(update.disabled).toBe(false)
    expect(within(pane).queryByText(gateHint(2))).toBeNull()
    await fireEvent.click(update)
    expect(handlerArgs('open_terminal')).toEqual([{ action: 'Update' }])
  })

  it('asks for a check again, and only ever a check', async () => {
    const pane = draw()

    await fireEvent.click(within(pane).getByRole('button', { name: CHECK_AGAIN }))
    expect(intents()).toEqual([{ type: 'CheckUpdates' }])
    expect(handlerArgs('open_terminal')).toEqual([])
  })
})
