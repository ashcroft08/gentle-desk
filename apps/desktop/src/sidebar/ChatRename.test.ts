// Renaming a chat from the sidebar (T2, D1(a), S4): the row's own menu, the
// title that turns into an input, and the command the app dispatches with the
// name the user typed.
//
// The write itself is Rust's (`rename_session`), so these tests assert what the
// view ASKS for - the session path and the typed name - and what it does with a
// refusal the next push carries.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import {
  handlerArgs,
  resetTauriMock,
  viewState,
  workspaceRow,
  chatRow,
} from '../test-support/tauri-mock'
import { serveSidebar } from '../test-support/sidebar'
import Sidebar from './Sidebar.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The two items the artboard draws, quoted so a change of copy is a change here. */
const RENAME = 'Rename'
const DELETE = 'Delete'
const TITLE = 'First chat'
const SESSION = 'C:\\pi\\sessions\\--dev-gentle-desk--\\2026_s1.jsonl'

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

/** One workspace with one chat, the row this slice renames. */
function serve(): void {
  serveSidebar({
    view: viewState({ total_sessions: 1 }),
    rows: [workspaceRow('gentle-desk'), chatRow(SESSION, { title: TITLE })],
    total: 2,
  })
  render(Sidebar)
}

/** The row's actions menu, opened the way the user opens it. */
async function openRowMenu(): Promise<HTMLElement> {
  const trigger = await screen.findByRole('button', { name: `Chat actions for ${TITLE}` })
  await fireEvent.click(trigger)
  return await screen.findByRole('menu', { name: `Chat actions for ${TITLE}` })
}

/** The inline input the row's title became. */
async function titleInput(): Promise<HTMLElement> {
  return await screen.findByRole('textbox', { name: 'Chat name' })
}

describe('Renaming a chat', () => {
  it('offers Rename and Delete on the row, and Delete does nothing yet', async () => {
    serve()
    const menu = await openRowMenu()

    const rename = within(menu).getByRole('menuitem', { name: RENAME })
    const remove = within(menu).getByRole('menuitem', { name: DELETE })
    expect(rename).toBeTruthy()
    expect(remove.getAttribute('aria-disabled')).toBe('true')

    // The stub is DRAWN but wired to nothing: no command leaves the view.
    await fireEvent.click(remove)
    expect(handlerArgs('rename_session')).toEqual([])
    expect(screen.getByText(TITLE)).toBeTruthy()
  })

  it('turns the row title into an input that commits the typed name on Enter', async () => {
    serve()
    const menu = await openRowMenu()
    await fireEvent.click(within(menu).getByRole('menuitem', { name: RENAME }))

    const input = await titleInput()
    expect((input as HTMLInputElement).value).toBe(TITLE)
    // The input is ready to type in the moment it replaces the title.
    await vi.waitFor(() => expect(document.activeElement).toBe(input))
    await fireEvent.input(input, { target: { value: 'Season two' } })
    await fireEvent.keyDown(input, { key: 'Enter' })

    expect(handlerArgs('rename_session')).toEqual([{ sessionId: SESSION, name: 'Season two' }])
    expect(screen.queryByRole('textbox', { name: 'Chat name' })).toBeNull()
  })

  it('cancels on Escape and dispatches nothing', async () => {
    serve()
    const menu = await openRowMenu()
    await fireEvent.click(within(menu).getByRole('menuitem', { name: RENAME }))

    const input = await titleInput()
    await fireEvent.input(input, { target: { value: 'Never sent' } })
    await fireEvent.keyDown(input, { key: 'Escape' })

    expect(handlerArgs('rename_session')).toEqual([])
    expect(screen.queryByRole('textbox', { name: 'Chat name' })).toBeNull()
    expect(screen.getByText(TITLE)).toBeTruthy()
  })

  it('draws the reason a refused rename left', async () => {
    serveSidebar({
      view: viewState({
        total_sessions: 1,
        sessions_notice: 'This chat is working. Rename it when the turn ends.',
      }),
      rows: [workspaceRow('gentle-desk'), chatRow(SESSION, { title: TITLE })],
      total: 2,
    })
    render(Sidebar)

    const notice = await screen.findByText(
      'This chat is working. Rename it when the turn ends.',
    )
    expect(notice.getAttribute('role')).toBe('status')
  })
})
