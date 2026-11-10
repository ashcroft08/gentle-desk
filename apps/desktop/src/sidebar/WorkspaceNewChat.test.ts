import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import { resetTauriMock, viewState, chatRow, workspaceRow } from '../test-support/tauri-mock'
import { rowOf, serveSidebar, type SidebarFixture } from '../test-support/sidebar'
import Sidebar from './Sidebar.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

let fixture: SidebarFixture

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

describe('workspace new-chat icon', () => {
  it('stays hidden until the workspace is hovered or focused, and sends the path', async () => {
    fixture = serveSidebar({
      view: viewState({ total_sessions: 2 }),
      rows: [workspaceRow('one', { path: 'C:\\dev\\one' }), chatRow('a', { title: 'Alpha' })],
      total: 2,
    })
    render(Sidebar)

    const row = await vi.waitFor(() => rowOf('Alpha'))
    const plus = screen.getByRole('button', { name: 'New chat in one' })
    expect(plus.getAttribute('data-visible')).toBe('false')

    // Hovering any row of the section reveals its icon, not only its header.
    await fireEvent.mouseEnter(row)
    expect(plus.getAttribute('data-visible')).toBe('true')

    await fireEvent.click(plus)
    expect(fixture.intents()).toEqual([
      { type: 'NewChatInWorkspace', data: { path: 'C:\\dev\\one' } },
    ])
  })

  it('keeps the icon away until the workspace is hovered, even with focus inside the row', async () => {
    // It used to appear on any focus inside the row, so clicking it pinned it
    // open for good. A keyboard user still reaches it: the row's own tab stop
    // makes the button tabbable, and the stylesheet shows it on focus-visible.
    fixture = serveSidebar({
      view: viewState({ total_sessions: 1 }),
      rows: [workspaceRow('one')],
      total: 1,
    })
    render(Sidebar)

    const row = await vi.waitFor(() => rowOf('one'))
    const plus = screen.getByRole('button', { name: 'New chat in one' })
    expect(plus.getAttribute('data-visible')).toBe('false')

    await fireEvent.focusIn(row)

    expect(plus.getAttribute('data-visible')).toBe('false')
    expect(row.getAttribute('tabindex')).toBe('0')
    expect(plus.getAttribute('tabindex')).toBe('0')
  })

  it('shows the reason of a folder that is gone and refuses to start a chat there', async () => {
    fixture = serveSidebar({
      view: viewState({ total_sessions: 1 }),
      rows: [
        workspaceRow('archive', { available: false, reason: 'This folder no longer exists.' }),
      ],
      total: 1,
    })
    render(Sidebar)

    const reason = await screen.findByText('This folder no longer exists.')
    const plus = screen.getByRole('button', { name: 'New chat in archive' })
    expect(plus.getAttribute('aria-disabled')).toBe('true')
    expect(plus.getAttribute('aria-describedby')).toBe(reason.id)

    await fireEvent.click(plus)
    expect(fixture.intents()).toEqual([])
  })
})
