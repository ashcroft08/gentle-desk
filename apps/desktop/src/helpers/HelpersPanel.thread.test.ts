// The panel's switch (T7b-2), one test per rule: a selected card opens its own
// thread in place of the list, the list stays when there is no thread for it,
// Back sends CloseHelper once, and the focus returns to the card that was open.

import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte'
import type { HelperRowView_Serialize as HelperRow } from '../contract/bindings'
import { helperRow, helpersPanel } from '../test-support/helpers'
import { helperThread } from '../test-support/helper-thread'
import { handlerArgs, resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import HelpersPanel from './HelpersPanel.svelte'
import { BACK_TO_HELPERS, HELPER_THREAD_LABEL, HELPERS_LIST_LABEL } from './copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

const ROWS: HelperRow[] = [
  helperRow('h1', { agent: 'explore', title: 'Inventory of sidebar modules' }),
  helperRow('h2', { agent: 'verify', title: 'Contract tests for the bridge' }),
]

/** Draws the panel with two cards, optionally open on one of them. */
function draw(selected: string | null = null, thread: ReturnType<typeof helperThread> | null = null) {
  const helpers = helpersPanel(ROWS)
  return render(HelpersPanel, { props: { helpers, selected, thread } })
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

describe('the helpers panel thread', () => {
  it('draws the selected helper own thread instead of the list', () => {
    draw('h1', helperThread({ id: 'h1' }))

    expect(screen.getByRole('region', { name: HELPER_THREAD_LABEL })).toBeTruthy()
    expect(screen.getByRole('heading').textContent).toBe(
      'explore - Inventory of sidebar modules',
    )
    expect(screen.queryByRole('listbox', { name: HELPERS_LIST_LABEL })).toBeNull()
    expect(screen.queryByRole('option')).toBeNull()
  })

  it('keeps the list when no thread arrived for the selected card', () => {
    draw('h1', null)

    expect(screen.getByRole('listbox', { name: HELPERS_LIST_LABEL })).toBeTruthy()
    expect(screen.queryByRole('region', { name: HELPER_THREAD_LABEL })).toBeNull()
  })

  it('never draws another helper thread under the selected card', () => {
    draw('h1', helperThread({ id: 'h2' }))

    expect(screen.getByRole('listbox', { name: HELPERS_LIST_LABEL })).toBeTruthy()
    expect(screen.queryByRole('region', { name: HELPER_THREAD_LABEL })).toBeNull()
  })

  it('sends CloseHelper once when Back is pressed', async () => {
    draw('h1', helperThread({ id: 'h1' }))

    await fireEvent.click(screen.getByRole('button', { name: BACK_TO_HELPERS }))

    expect(intents()).toEqual([{ type: 'CloseHelper' }])
  })

  it('gives the focus back to the card that was open', async () => {
    const view = draw('h1', helperThread({ id: 'h1' }))
    expect(document.activeElement).toBe(screen.getByRole('button', { name: BACK_TO_HELPERS }))

    await view.rerender({ helpers: helpersPanel(ROWS), selected: null, thread: null })

    await waitFor(() => expect(screen.getByRole('listbox', { name: HELPERS_LIST_LABEL })).toBeTruthy())
    const card = screen.getAllByRole('option')[0]
    expect(card.getAttribute('data-helper-id')).toBe('h1')
    expect(document.activeElement).toBe(card)
  })
})
