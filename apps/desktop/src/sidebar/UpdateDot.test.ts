// The sidebar footer's gear, when an update is waiting (0.1.14): a 6px accent
// dot when a row the app updates from a terminal is behind, and never a number
// or a word on the bar. The dot is decoration; the gear's own accessible name is
// the second channel, so the promise is not carried by colour alone.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import type {
  PackagesView_Serialize as PackagesView,
  UpdatesView_Serialize as UpdatesView,
} from '../contract/bindings'
import { resetTauriMock, viewState, workspaceRow, chatRow } from '../test-support/tauri-mock'
import { serveSidebar } from '../test-support/sidebar'
import Sidebar from './Sidebar.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The two names the gear wears: its own, and its own with the promise. */
const SETTINGS = 'Settings'
const WAITING = 'Settings - an update is available'

/** The Extensions block's slice: unasked unless a test says otherwise. */
function packages(over: Partial<PackagesView> = {}): PackagesView {
  return { available: false, behind: [], unknown: false, truncated: false, unchecked: false, checked: null, ...over }
}

/** The pushed check, with pi behind or level. */
function updates(behind: boolean, packagesView: PackagesView = packages()): UpdatesView {
  const row = behind
    ? { current: '1.5.2', latest: '2.0.5', update_available: true }
    : { current: '2.0.5', latest: '2.0.5', update_available: false }
  return {
    status: 'Ready',
    desk: { current: '0.1.14', latest: '0.1.14', update_available: false },
    pi: row,
    shell: { current: '0.4.0', latest: null, update_available: null },

    packages: packagesView,
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

  it('draws the dot for a package pi reports as behind (packages-status T3)', async () => {
    // The packages slice joins the waiting rule: a behind extension is a promise
    // exactly like a behind pi or shell row.
    const container = draw(
      updates(false, packages({ available: true, behind: [{ name: 'pi-hud' }] })),
    )

    const gear = await screen.findByRole('button', { name: WAITING })
    expect(container.querySelector('.foot .ic .dot')).not.toBeNull()
    expect(gear.getAttribute('aria-label')).toBe(WAITING)
  })

  it('stays dark for a packages read that never ran or could not run', async () => {
    // Unasked and unknown are never a promise, and neither is an answered
    // listing with nothing behind - or one pi's own check has not run for yet
    // (packages-status T6, S6).
    for (const slice of [
      packages(),
      packages({ unknown: true }),
      packages({ available: true }),
      packages({ available: true, unchecked: true }),
    ]) {
      cleanup()
      const container = draw(updates(false, slice))

      const gear = await screen.findByRole('button', { name: SETTINGS })
      expect(container.querySelector('.dot')).toBeNull()
      expect(gear.getAttribute('aria-label')).toBe(SETTINGS)
    }
  })
})
