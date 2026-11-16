// The panel's two columns (T3, 01b): the helper list is always drawn on the
// left and the selected helper's thread is drawn beside it, one test per rule -
// both columns at once, another card swapping the right column in place, the
// counters and the list kept when there is no thread for the selected card or
// when the thread names another helper, Back sending CloseHelper once, and the
// keyboard that stays in the list while the thread is beside it.

import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import type { HelperRowView_Serialize as HelperRow } from '../contract/bindings'
import { helperRow, helpersPanel } from '../test-support/helpers'
import { helperThread } from '../test-support/helper-thread'
import { handlerArgs, resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import HelpersPanel from './HelpersPanel.svelte'
import { helpersCopy } from './copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The panel's own words, as the panel drew them before 0.2.0 (English). */
const copy = helpersCopy('en')

const ROWS: HelperRow[] = [
  helperRow('h1', { agent: 'explore', title: 'Inventory of sidebar modules' }),
  helperRow('h2', {
    agent: 'verify',
    title: 'Contract tests for the bridge',
    status: 'Done',
    status_label: 'done',
  }),
]

/** Draws the panel with two cards, optionally open on one of them. */
function draw(selected: string | null = null, thread: ReturnType<typeof helperThread> | null = null) {
  const helpers = helpersPanel(ROWS)
  return render(HelpersPanel, { props: { helpers, selected, thread } })
}

/** The list column's boxes, in the order the panel drew them. */
function options(): HTMLElement[] {
  return within(screen.getByRole('listbox', { name: copy.listLabel })).getAllByRole('option')
}

/** The intents the panel sent, oldest first. */
function intents(): unknown[] {
  return handlerArgs('dispatch_intent').map((args) => args.intent)
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
})

describe('the helpers panel two columns', () => {
  it('draws the list and the selected helper thread side by side', () => {
    draw('h1', helperThread({ id: 'h1' }))

    expect(options()).toHaveLength(2)
    expect(screen.getByRole('region', { name: copy.threadLabel })).toBeTruthy()
    expect(
      within(screen.getByRole('region', { name: copy.threadLabel })).getByRole('heading').textContent,
    ).toBe('explore - Inventory of sidebar modules')
  })

  it('draws the thread of another helper in place when the selection moves', async () => {
    const view = draw('h1', helperThread({ id: 'h1' }))

    await view.rerender({
      helpers: helpersPanel(ROWS),
      selected: 'h2',
      thread: helperThread({ id: 'h2', agent: 'verify', title: 'Contract tests for the bridge' }),
    })

    const cards = options()
    expect(cards).toHaveLength(2)
    expect(cards[0].getAttribute('aria-selected')).toBe('false')
    expect(cards[1].getAttribute('aria-selected')).toBe('true')
    expect(
      within(screen.getByRole('region', { name: copy.threadLabel })).getByRole('heading').textContent,
    ).toBe('verify - Contract tests for the bridge')
  })

  it('keeps the counters line above the list while a thread is drawn', () => {
    const helpers = helpersPanel(ROWS, { running: 1, finished: 1 })
    render(HelpersPanel, { props: { helpers, selected: 'h1', thread: helperThread({ id: 'h1' }) } })

    expect(screen.getByText(helpers.summary_line).textContent).toBe(helpers.summary_line)
    expect(options()).toHaveLength(2)
  })

  it('keeps the list when no thread arrived for the selected card', () => {
    draw('h1', null)

    expect(options()).toHaveLength(2)
    expect(screen.queryByRole('region', { name: copy.threadLabel })).toBeNull()
  })

  it('never draws another helper thread under the selected card', () => {
    draw('h1', helperThread({ id: 'h2' }))

    expect(options()).toHaveLength(2)
    expect(screen.queryByRole('region', { name: copy.threadLabel })).toBeNull()
  })

  it('sends CloseHelper once when Back is pressed', async () => {
    draw('h1', helperThread({ id: 'h1' }))

    await fireEvent.click(screen.getByRole('button', { name: copy.back }))

    expect(intents()).toEqual([{ type: 'CloseHelper' }])
  })

  it('keeps the keyboard in the list when the selected helper thread appears', async () => {
    const view = draw('h1', null)
    const card = options()[0]
    card.focus()

    await view.rerender({
      helpers: helpersPanel(ROWS),
      selected: 'h1',
      thread: helperThread({ id: 'h1' }),
    })

    expect(screen.getByRole('region', { name: copy.threadLabel })).toBeTruthy()
    expect(document.activeElement).toBe(options()[0])
    expect(document.activeElement).not.toBe(screen.getByRole('button', { name: copy.back }))
  })

  it('walks the list with the arrow keys while the thread is beside it', async () => {
    draw('h1', helperThread({ id: 'h1' }))

    await fireEvent.keyDown(options()[0], { key: 'ArrowDown' })

    expect(document.activeElement).toBe(options()[1])
  })
})
