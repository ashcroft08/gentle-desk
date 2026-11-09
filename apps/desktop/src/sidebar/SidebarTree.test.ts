import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import { activity, resetTauriMock, viewState, chatRow, workspaceRow } from '../test-support/tauri-mock'
import { rowOf, serveSidebar, treeItems, type SidebarFixture } from '../test-support/sidebar'
import Sidebar from './Sidebar.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

let fixture: SidebarFixture

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

describe('Sidebar tree', () => {
  it('draws a workspace header and then its chats, in the order Rust flattened them', async () => {
    fixture = serveSidebar({
      view: viewState({ total_sessions: 4 }),
      rows: [
        workspaceRow('one'),
        chatRow('a', { title: 'Alpha' }),
        workspaceRow('two'),
        chatRow('b', { title: 'Bravo' }),
      ],
      total: 4,
    })
    render(Sidebar)

    await screen.findByText('Alpha')
    expect(treeItems().map((item) => item.textContent?.replace(/\s+/g, ' ').trim())).toEqual([
      'one',
      'Alpha now',
      'two',
      'Bravo now',
    ])
  })

  it('folds a header whose chats Rust left out and asks for the opposite state', async () => {
    fixture = serveSidebar({
      view: viewState({ total_sessions: 2 }),
      rows: [workspaceRow('one', { path: 'C:\\dev\\one', collapsed: true })],
      total: 1,
    })
    render(Sidebar)

    const header = await vi.waitFor(() => rowOf('one'))
    expect(treeItems()).toHaveLength(1)
    expect(header.getAttribute('aria-expanded')).toBe('false')
    expect(screen.queryByText('Alpha')).toBeNull()

    await fireEvent.click(header)
    expect(fixture.intents()).toEqual([
      { type: 'SetWorkspaceCollapsed', data: { path: 'C:\\dev\\one', collapsed: false } },
    ])
  })

  it('toggles an expanded header to collapsed', async () => {
    fixture = serveSidebar({
      view: viewState({ total_sessions: 2 }),
      rows: [workspaceRow('one', { path: 'C:\\dev\\one' }), chatRow('a', { title: 'Alpha' })],
      total: 2,
    })
    render(Sidebar)

    const header = await vi.waitFor(() => rowOf('one'))
    expect(header.getAttribute('aria-expanded')).toBe('true')
    await fireEvent.click(header)
    expect(fixture.intents()).toEqual([
      { type: 'SetWorkspaceCollapsed', data: { path: 'C:\\dev\\one', collapsed: true } },
    ])
  })

  it('marks the selected chat and sends its id when it is clicked', async () => {
    fixture = serveSidebar({
      view: viewState({
        foreground_chat: 'a',
        total_sessions: 2,
        activity: [activity('a', 'Working')],
      }),
      rows: [
        workspaceRow('one'),
        chatRow('a', { title: 'Alpha', modified_ago: '2h ago' }),
        chatRow('b', { title: 'Bravo' }),
      ],
      total: 3,
    })
    render(Sidebar)

    const row = await vi.waitFor(() => rowOf('Alpha'))
    expect(row.getAttribute('aria-selected')).toBe('true')
    expect(rowOf('Bravo').getAttribute('aria-selected')).toBe('false')
    expect(screen.getByText('2h ago')).toBeTruthy()
    expect(screen.getByRole('img', { name: 'Working' })).toBeTruthy()

    await fireEvent.click(row)
    expect(fixture.intents()).toEqual([{ type: 'OpenChat', data: { chat_id: 'a' } }])
  })

  it('keeps one roving tab stop and moves it with the arrow keys, Home and End', async () => {
    fixture = serveSidebar({
      view: viewState({ total_sessions: 3 }),
      rows: [workspaceRow('one'), chatRow('a', { title: 'Alpha' }), workspaceRow('two')],
      total: 3,
    })
    render(Sidebar)

    await screen.findByText('Alpha')
    const items = treeItems()
    expect(items.filter((item) => item.getAttribute('tabindex') === '0')).toHaveLength(1)

    items[0].focus()
    await fireEvent.keyDown(items[0], { key: 'ArrowDown' })
    expect(document.activeElement).toBe(items[1])
    expect(items[1].getAttribute('tabindex')).toBe('0')

    await fireEvent.keyDown(items[1], { key: 'End' })
    expect(document.activeElement).toBe(items[2])
    await fireEvent.keyDown(items[2], { key: 'Home' })
    expect(document.activeElement).toBe(items[0])
    await fireEvent.keyDown(items[0], { key: 'ArrowUp' })
    expect(document.activeElement).toBe(items[0])
  })

  it('activates the focused row with Enter or Space', async () => {
    fixture = serveSidebar({
      view: viewState({ total_sessions: 3 }),
      rows: [
        workspaceRow('one', { path: 'C:\\dev\\one' }),
        chatRow('a', { title: 'Alpha' }),
      ],
      total: 2,
    })
    render(Sidebar)

    await screen.findByText('Alpha')
    const items = treeItems()
    await fireEvent.keyDown(items[0], { key: 'Enter' })
    expect(fixture.intents()).toEqual([
      { type: 'SetWorkspaceCollapsed', data: { path: 'C:\\dev\\one', collapsed: true } },
    ])
    await fireEvent.keyDown(items[1], { key: ' ' })
    expect(fixture.intents().at(-1)).toEqual({ type: 'OpenChat', data: { chat_id: 'a' } })
  })
})
