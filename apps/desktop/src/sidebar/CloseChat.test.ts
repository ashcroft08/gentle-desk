// Close chat (0.1.14): the chat row's actions menu names the way out of the
// Updates pane's gate. The item sits next to Rename and Delete, is the menu's
// own structure and ARIA (one menuitem, reachable by the keyboard like its
// neighbours), and it sends one intent: the state closes the chat, this page
// never stops a child itself.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import { resetTauriMock, viewState, workspaceRow, chatRow } from '../test-support/tauri-mock'
import { serveSidebar, type SidebarFixture } from '../test-support/sidebar'
import Sidebar from './Sidebar.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

const TITLE = 'First chat'
const SESSION = 'C:\\pi\\sessions\\--dev-gentle-desk--\\2026_s1.jsonl'
const ACTIONS = `Chat actions for ${TITLE}`

let fixture: SidebarFixture

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

/** One workspace with one chat, its actions menu opened. */
async function openRowMenu(): Promise<HTMLElement> {
  fixture = serveSidebar({
    view: viewState({ total_sessions: 1, open_chats: 1 }),
    rows: [workspaceRow('gentle-desk'), chatRow(SESSION, { title: TITLE })],
    total: 2,
  })
  render(Sidebar)
  await fireEvent.click(await screen.findByRole('button', { name: ACTIONS }))
  return await screen.findByRole('menu', { name: ACTIONS })
}

describe("The chat row's Close chat", () => {
  it('sits between Rename and Delete, one menuitem each', async () => {
    const menu = await openRowMenu()

    expect(within(menu).getAllByRole('menuitem').map((item) => item.textContent?.trim())).toEqual([
      'Rename',
      'Close chat',
      'Delete',
    ])
  })

  it('sends the one intent that closes that chat, and nothing else', async () => {
    const menu = await openRowMenu()

    await fireEvent.click(within(menu).getByRole('menuitem', { name: 'Close chat' }))

    expect(fixture.intents()).toEqual([{ type: 'CloseChat', data: { chat_id: SESSION } }])
  })

  it('is reachable with the keyboard, like the items beside it', async () => {
    fixture = serveSidebar({
      view: viewState({ total_sessions: 1, open_chats: 1 }),
      rows: [workspaceRow('gentle-desk'), chatRow(SESSION, { title: TITLE })],
      total: 2,
    })
    render(Sidebar)

    // The keyboard opens the menu on the trigger, and the arrow keys walk the
    // items: Close chat is one stop of the same walk.
    const trigger = await screen.findByRole('button', { name: ACTIONS })
    trigger.focus()
    await fireEvent.keyDown(trigger, { key: 'ArrowDown' })
    const menu = await screen.findByRole('menu', { name: ACTIONS })
    await fireEvent.keyDown(menu, { key: 'ArrowDown' })

    const item = within(menu).getByRole('menuitem', { name: 'Close chat' })
    item.focus()
    expect(document.activeElement).toBe(item)
    await fireEvent.keyDown(item, { key: 'Enter' })

    await vi.waitFor(() =>
      expect(fixture.intents()).toEqual([{ type: 'CloseChat', data: { chat_id: SESSION } }]),
    )
  })
})
