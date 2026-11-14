// The sidebar footer's gear, when an update is waiting (0.1.14): a 6px accent
// dot when a row the app updates from a terminal is behind, and never a number
// or a word on the bar. The dot is decoration; the gear's own accessible name is
// the second channel, so the promise is not carried by colour alone.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import type { UpdatesView_Serialize as UpdatesView } from '../contract/bindings'
import { resetTauriMock, viewState, workspaceRow, chatRow } from '../test-support/tauri-mock'
import { serveSidebar } from '../test-support/sidebar'
import Sidebar from './Sidebar.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The two names the gear wears: its own, and its own with the promise. */
const SETTINGS = 'Settings'
const WAITING = 'Settings - an update is available'

/** The pushed check, with pi behind or level. */
function updates(behind: boolean): UpdatesView {
  const row = behind
    ? { current: '1.5.2', latest: '2.0.5', update_available: true }
    : { current: '2.0.5', latest: '2.0.5', update_available: false }
  return {
    status: 'Ready',
    desk: { current: '0.1.14', latest: '0.1.14', update_available: false },
    pi: row,
    shell: { current: '0.4.0', latest: null, update_available: null },
  }
}

/** The sidebar, one workspace and one chat, with `view` as the pushed state. */
function draw(view: UpdatesView): HTMLElement {
  serveSidebar({
    view: viewState({ total_sessions: 1, updates: view }),
    rows: [workspaceRow('gentle-desk'), chatRow('/sessions/a.jsonl', { title: 'First chat' })],
    total: 2,
  })
  const { container } = render(Sidebar)
  return container
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

describe('The gear that promises an update', () => {
  it('draws one dot on the gear when a terminal row is behind', async () => {
    const container = draw(updates(true))

    const gear = await screen.findByRole('button', { name: WAITING })
    const dot = container.querySelector('.foot .ic .dot')
    expect(dot).not.toBeNull()
    expect(gear.contains(dot)).toBe(true)
    // A dot, not a count: the bar carries no number and no word of its own.
    expect(dot?.textContent).toBe('')
  })

  it('says it in words too, so the colour is never the only channel', async () => {
    draw(updates(true))

    expect(await screen.findByRole('button', { name: WAITING })).toBeTruthy()
    expect(screen.queryByRole('button', { name: SETTINGS })).toBeNull()
  })

  it('draws no dot when nothing this app updates from a terminal is behind', async () => {
    const container = draw(updates(false))

    const gear = await screen.findByRole('button', { name: SETTINGS })
    expect(container.querySelector('.dot')).toBeNull()
    expect(gear.getAttribute('aria-label')).toBe(SETTINGS)
  })
})
