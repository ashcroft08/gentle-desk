// Deleting a chat from the sidebar (T3, D2(b), S4; the removal feature's S3
// anatomy): the row's own menu, the confirm dialog that names the chat in a plain
// elided path, and the command the app dispatches once the user confirms it.
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
/** The session file's path, as the dialog shows it: plain and elided in the middle. */
const ELIDED = 'C:\\…\\2026_s1.jsonl'
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

  it('draws the question, the scope in bold and a plain elided path', async () => {
    serve()
    const dialog = await openDialog()

    expect(within(dialog).getByText(DIALOG_TITLE)).toBeTruthy()
    // One consequence line, its scope the only bold run (S3).
    const scope = within(dialog).getByText('chat')
    expect(scope.tagName).toBe('B')
    expect(dialog.textContent ?? '').toContain('cannot be regenerated')
    expect(dialog.textContent ?? '').not.toMatch(/chat file/i)
    // The path is plain monospaced text with its middle elided, never a code box.
    const path = within(dialog).getByText(ELIDED)
    expect(path.tagName).not.toBe('CODE')
    expect(path.className).toContain('mono')
    expect(dialog.querySelector('code')).toBeNull()
  })

  it('puts the destructive answer last, red, and Cancel before it', async () => {
    serve()
    const dialog = await openDialog()

    const buttons = within(dialog).getAllByRole('button')
    expect(buttons.map((button) => button.textContent?.trim())).toEqual(['Cancel', DELETE])
    expect(buttons[1].classList.contains('d')).toBe(true)
    expect(buttons[0].classList.contains('d')).toBe(false)
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
