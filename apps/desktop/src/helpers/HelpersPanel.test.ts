// The helpers panel, one test per rule: the counters line above the list, one
// card per helper carrying its status as the mark's accessible name, the plain
// line for a chat with none, and the selection that click or Enter sends.

import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import type { HelperRowView_Serialize as HelperRow } from '../contract/bindings'
import {
  DOT,
  HELPERS_LIST_LABEL,
  NO_HELPERS,
  cardMeta,
} from './copy'
import HelpersPanel from './HelpersPanel.svelte'
import { helperRow, helpersPanel } from '../test-support/helpers'
import { handlerArgs, resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

const STATUSES: [HelperRow['status'], string][] = [
  ['Running', 'running'],
  ['Waiting', 'waiting'],
  ['Queued', 'queued'],
  ['Done', 'done'],
  ['Failed', 'failed'],
  ['Cancelled', 'cancelled'],
]

/** The six helpers of the stand-in, one per status. */
function sixHelpers(): HelperRow[] {
  return STATUSES.map(([status, label], index) =>
    helperRow(`h${index}`, { status, status_label: label, title: `${status} helper` }),
  )
}

function draw(over: { tasks?: HelperRow[]; selected?: string | null; counts?: Record<string, number> } = {}) {
  const helpers = helpersPanel(over.tasks ?? sixHelpers(), over.counts)
  render(HelpersPanel, { props: { helpers, selected: over.selected ?? null } })
  return helpers
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

describe('HelpersPanel', () => {
  it('draws one card per helper, with its status as the mark accessible name', () => {
    draw()

    const list = screen.getByRole('listbox', { name: HELPERS_LIST_LABEL })
    const options = within(list).getAllByRole('option')
    expect(options).toHaveLength(6)
    STATUSES.forEach(([, label], index) => {
      expect(within(options[index]).getByRole('img', { name: label })).toBeTruthy()
    })
  })

  it('names each card by its agent and its task title', () => {
    draw({ tasks: [helperRow('h1', { agent: 'explore', title: 'Inventory of sidebar modules' })] })

    const option = screen.getByRole('option')
    expect(option.textContent).toContain('explore')
    expect(option.textContent).toContain('Inventory of sidebar modules')
  })

  it('draws the counters line above the list, exactly as Rust spelled it', () => {
    const helpers = draw({ counts: { running: 1, queued: 1, waiting: 1, finished: 2 } })

    const line = screen.getByText(
      `1 running ${DOT} 1 queued ${DOT} 1 waiting ${DOT} 2 finished`,
    )

    expect(line.textContent).toBe(helpers.summary_line)
  })

  it('says a chat has no helpers with a plain line and no listbox', () => {
    draw({ tasks: [] })

    expect(screen.getByText(NO_HELPERS)).toBeTruthy()
    expect(screen.queryByRole('listbox')).toBeNull()
  })

  it('marks the selected card and keeps it in the tab order', () => {
    draw({ selected: 'h2' })

    const options = screen.getAllByRole('option')
    expect(options[2].getAttribute('aria-selected')).toBe('true')
    expect(options[2].getAttribute('tabindex')).toBe('0')
    expect(options[0].getAttribute('aria-selected')).toBe('false')
  })

  it('sends SelectHelper with the id of the card that was clicked', async () => {
    draw()

    await fireEvent.click(screen.getAllByRole('option')[3])

    expect(intents()).toEqual([{ type: 'SelectHelper', data: { helper_id: 'h3' } }])
  })

  it('sends SelectHelper with the id of the card Enter is pressed on', async () => {
    draw({ selected: 'h1' })

    await fireEvent.keyDown(screen.getAllByRole('option')[1], { key: 'Enter' })

    expect(intents()).toEqual([{ type: 'SelectHelper', data: { helper_id: 'h1' } }])
  })

  it('walks the list with the arrow keys and takes the focus along', async () => {
    draw({ selected: 'h0' })
    const options = screen.getAllByRole('option')

    await fireEvent.keyDown(options[0], { key: 'ArrowDown' })
    expect(document.activeElement).toBe(screen.getAllByRole('option')[1])

    await fireEvent.keyDown(screen.getAllByRole('option')[1], { key: 'ArrowUp' })
    expect(document.activeElement).toBe(screen.getAllByRole('option')[0])

    await fireEvent.keyDown(screen.getAllByRole('option')[0], { key: 'End' })
    expect(document.activeElement).toBe(screen.getAllByRole('option')[5])

    await fireEvent.keyDown(screen.getAllByRole('option')[5], { key: 'Home' })
    expect(document.activeElement).toBe(screen.getAllByRole('option')[0])
  })

  it('draws the counts, the last step and the failure of a card', () => {
    draw({
      tasks: [
        helperRow('h1', {
          agent: 'explore',
          turns: 4,
          tool_calls: 3,
          last_step: 'grep',
        }),
        helperRow('h2', {
          status: 'Failed',
          status_label: 'failed',
          error: 'model quota exceeded',
        }),
      ],
    })

    const options = screen.getAllByRole('option')
    expect(options[0].textContent).toContain(cardMeta(4, 3))
    expect(options[0].textContent).toContain('grep')
    expect(options[1].textContent).toContain('model quota exceeded')
  })
})
