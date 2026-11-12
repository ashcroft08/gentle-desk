// Deleting a chat from the sidebar (T3, D2(b), S4): the row's own menu, the
// confirm dialog that names the REAL file of that chat, and the command the app
// dispatches once the user confirms it.
//
// The file removal itself is Rust's (`delete_session`), so these tests assert
// what the view ASKS for - the session path of the row, and the confirmation
// only the dialog's Delete sends - and that a Cancel leaves nothing behind.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import {
  deferred,
  handlerArgs,
  resetTauriMock,
  setHandler,
  viewState,
  workspaceRow,
  chatRow,
} from '../test-support/tauri-mock'
import { serveSidebar } from '../test-support/sidebar'
import Sidebar from './Sidebar.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The items the artboard draws, and the dialog's own words, quoted. */
const DELETE = 'Delete'
const TITLE = 'First chat'
const DIALOG_TITLE = 'Delete this chat?'
const DIALOG_BODY = 'The chat file is removed from disk and cannot be restored.'
const SESSION = 'C:\\pi\\sessions\\--dev-gentle-desk--\\2026_s1.jsonl'

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

/** One workspace with one chat, the row this slice deletes. */
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

/** The confirm dialog, opened from the row's Delete, once it is drawn. */
async function openDialog(): Promise<HTMLElement> {
  const menu = await openRowMenu()
  await fireEvent.click(within(menu).getByRole('menuitem', { name: DELETE }))
  return await screen.findByRole('dialog')
}

describe('Deleting a chat', () => {
  it('keeps Delete enabled on the row, and asks before anything is removed', async () => {
    serve()
    const menu = await openRowMenu()

    const remove = within(menu).getByRole('menuitem', { name: DELETE })
    expect(remove.getAttribute('aria-disabled')).not.toBe('true')

    await fireEvent.click(remove)

    expect(await screen.findByRole('dialog')).toBeTruthy()
    expect(handlerArgs('delete_session')).toEqual([])
  })

  it('names the real file of that chat in the dialog', async () => {
    serve()
    const dialog = await openDialog()

    expect(within(dialog).getByText(DIALOG_TITLE)).toBeTruthy()
    expect(within(dialog).getByText(DIALOG_BODY)).toBeTruthy()
    expect(within(dialog).getByText(SESSION).tagName).toBe('CODE')
  })

  it('dispatches nothing when the user cancels', async () => {
    serve()
    const dialog = await openDialog()

    await fireEvent.click(within(dialog).getByRole('button', { name: 'Cancel' }))

    await vi.waitFor(() => expect(dialog.isConnected).toBe(false))
    expect(handlerArgs('delete_session')).toEqual([])
    expect(screen.getByText(TITLE)).toBeTruthy()
  })

  it('dispatches nothing when the dialog is dismissed with Escape', async () => {
    serve()
    const dialog = await openDialog()

    await fireEvent.keyDown(document.body, { key: 'Escape' })

    await vi.waitFor(() => expect(dialog.isConnected).toBe(false))
    expect(handlerArgs('delete_session')).toEqual([])
  })

  it('sends the path with the confirmation, once, and closes on the answer', async () => {
    serve()
    const pending = deferred<unknown>()
    setHandler('delete_session', () => pending.promise)
    const dialog = await openDialog()

    const confirm = within(dialog).getByRole('button', { name: DELETE })
    await fireEvent.click(confirm)
    await fireEvent.click(confirm)

    expect(handlerArgs('delete_session')).toEqual([{ sessionId: SESSION, confirmed: true }])
    expect(dialog.isConnected).toBe(true)

    pending.resolve(viewState({ total_sessions: 0 }))
    await vi.waitFor(() => expect(dialog.isConnected).toBe(false))
  })
})
