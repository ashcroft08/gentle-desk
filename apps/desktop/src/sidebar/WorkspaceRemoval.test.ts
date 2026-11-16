// Removing a workspace from the sidebar (S1, S3, S8): the workspace header row's
// own "..." menu and its one destructive item, the removal dialog drawn from the
// offer, and the command the app sends once the user confirms it.
//
// The removal itself is Rust's (`remove_workspace`), so these tests assert what
// the view ASKS for - the offer of the row the user picked, and the confirmation
// only the dialog's last button sends - and that Cancel or Escape leaves nothing
// behind.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import {
  chatRow,
  deferred,
  handlerArgs,
  resetTauriMock,
  setHandler,
  viewState,
  workspaceRow,
} from '../test-support/tauri-mock'
import { serveSidebar } from '../test-support/sidebar'
import Sidebar from './Sidebar.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The row, the folder it names and the two chats under it. */
const LABEL = 'gentle-desk'
const PATH = 'C:\\dev\\gentle-desk'
const ELIDED = 'C:\\…\\gentle-desk'
const FIRST = 'C:\\pi\\sessions\\--dev-gentle-desk--\\2026_s1.jsonl'
const SECOND = 'C:\\pi\\sessions\\--dev-gentle-desk--\\2026_s2.jsonl'

/** The dialog's own words, quoted from artboard B and the maintainer's anatomy. */
const MENU_ITEM = 'Remove workspace...'
const QUESTION = 'Remove this workspace?'
const SCOPE = '2 chats (1.7 MB)'
const SHOW = `Show ${SCOPE}`
const CONFIRM = 'Delete 2 chats and remove'
const KEEP_CONFIRM = 'Remove'
const KEEP_SCOPE = 'forgets the bookmark'

/** 1.7 MB, the measured workspace of the spec (S2). */
const BYTES = 1_800_000

/** The two failure lines the dialog draws, quoted as it draws them (R3-002, R3-003). */
const READ_FAILED =
  'The chats of this workspace could not be read. Nothing was removed. Try again from the workspace menu, or cancel.'
const REMOVE_FAILED =
  'The removal did not finish. Nothing more will be deleted unless you try again.'

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

/** The offer Rust answers for a workspace of `chats` chats, before anything happens. */
function offered(chats: number, bytes: number | null): void {
  setHandler('workspace_removal_offer', () => ({ key: LABEL, path: PATH, label: LABEL, chats, bytes }))
  setHandler('remove_workspace', () => viewState({ total_sessions: 0 }))
}

/** One workspace with two chats and its offer, drawn as the sidebar draws it. */
function serve(chats = 2): void {
  serveSidebar({
    view: viewState({ total_sessions: 2 }),
    rows: [
      workspaceRow(LABEL, { path: PATH, chat_count: chats }),
      chatRow(FIRST, { title: 'First chat' }),
      chatRow(SECOND, { title: 'Second chat' }),
    ],
    total: 3,
  })
  if (chats === 0) offered(0, 0)
  else offered(chats, BYTES)
  render(Sidebar)
}

/** The workspace header's own menu, opened the way the user opens it. */
async function openRowMenu(): Promise<HTMLElement> {
  const trigger = await screen.findByRole('button', { name: `Workspace actions for ${LABEL}` })
  await fireEvent.click(trigger)
  return await screen.findByRole('menu', { name: `Workspace actions for ${LABEL}` })
}

/** The removal dialog, opened from that menu's one destructive item, once the
    offer the dialog asks for has been drawn. */
async function openDialog(): Promise<HTMLElement> {
  const menu = await openRowMenu()
  await fireEvent.click(within(menu).getByRole('menuitem', { name: MENU_ITEM }))
  const dialog = await screen.findByRole('dialog')
  await within(dialog).findByText(QUESTION)
  await within(dialog).findAllByRole('button')
  return dialog
}

describe('Removing a workspace', () => {
  it('offers removal from the row menu, and asks for the offer when it opens', async () => {
    serve()
    const menu = await openRowMenu()

    const item = within(menu).getByRole('menuitem', { name: MENU_ITEM })
    expect(item.getAttribute('aria-disabled')).not.toBe('true')
    expect(handlerArgs('workspace_removal_offer')).toEqual([])

    await fireEvent.click(item)

    expect(await screen.findByRole('dialog')).toBeTruthy()
    expect(handlerArgs('workspace_removal_offer')).toEqual([{ path: PATH }])
    expect(handlerArgs('remove_workspace')).toEqual([])
  })

  it('draws the question, the elided mono path and the scope in bold', async () => {
    serve()
    const dialog = await openDialog()

    expect(within(dialog).getByText(QUESTION)).toBeTruthy()
    // The path is plain monospaced text with its middle elided, never a code box.
    const path = within(dialog).getByText(ELIDED)
    expect(path.tagName).not.toBe('CODE')
    expect(path.className).toContain('mono')
    // The scope is the one bold thing in the consequence line (S3).
    expect(within(dialog).getByText(SCOPE).tagName).toBe('B')
    // Never the words of the store's own files (S3).
    expect(dialog.textContent ?? '').not.toMatch(/chat file/i)
    expect(dialog.textContent ?? '').not.toMatch(/\.jsonl/)
  })

  it('puts the destructive button last, red and labelled with its scope', async () => {
    serve()
    const dialog = await openDialog()

    const buttons = within(dialog).getAllByRole('button')
    expect(buttons.map((button) => button.textContent?.trim())).toEqual(['Cancel', CONFIRM])
    expect(buttons[1].classList.contains('d')).toBe(true)
    expect(buttons[0].classList.contains('d')).toBe(false)
    expect(buttons[0].className).toContain('btn')
  })

  it('opens the disclosure of what goes, by chat name and never by file name', async () => {
    serve()
    const dialog = await openDialog()

    const summary = within(dialog).getByText(SHOW)
    expect(summary.tagName).toBe('SUMMARY')

    await fireEvent.click(summary)

    const list = within(dialog).getByRole('list')
    expect(within(list).getAllByRole('listitem')).toHaveLength(2)
    expect(list.textContent ?? '').toContain('First chat')
    expect(list.textContent ?? '').not.toMatch(/\.jsonl/)
  })

  it('removes the bookmark alone when the workspace holds no chat', async () => {
    serve(0)
    const dialog = await openDialog()

    expect(within(dialog).getByText(KEEP_SCOPE).tagName).toBe('B')
    expect(within(dialog).queryByText('Show 0 chats')).toBeNull()
    const confirm = within(dialog).getByRole('button', { name: KEEP_CONFIRM })
    // Nothing is destroyed, so nothing is red.
    expect(confirm.classList.contains('d')).toBe(false)
    expect(confirm.className).toContain('btn')
  })

  it('opens the focus on Cancel, never on the red answer', async () => {
    serve()
    const dialog = await openDialog()

    expect(document.activeElement).toBe(within(dialog).getByRole('button', { name: 'Cancel' }))
  })

  it('draws no disclosure when the page does not hold every chat it counts', async () => {
    // A folded workspace carries no chat rows, so the offer's count is all the
    // evidence there is: the consequence line names it, and no list pretends to.
    serveSidebar({
      view: viewState({ total_sessions: 2 }),
      rows: [workspaceRow(LABEL, { path: PATH, chat_count: 2, collapsed: true })],
      total: 1,
    })
    offered(2, BYTES)
    render(Sidebar)

    const dialog = await openDialog()
    expect(within(dialog).getByText(SCOPE).tagName).toBe('B')
    expect(within(dialog).queryByText(SHOW)).toBeNull()
  })

  it('says so, invents no count and leaves a way out when the offer cannot be read (R3-002)', async () => {
    // The row is served with an offer that answers, and the read is refused before
    // the dialog asks for it: the state the failure is about.
    serve()
    setHandler('workspace_removal_offer', () => {
      throw new Error('the shell refused the read')
    })
    const dialog = await openDialog()

    // The line that says the read failed, in the consequence line's own style (S3).
    const alert = within(dialog).getByRole('alert')
    expect(alert.className).toContain('why')
    expect(alert.textContent).toBe(READ_FAILED)
    expect(alert.querySelector('b')?.textContent).toBe('Nothing was removed.')
    // No number is invented for a count that never arrived, and nothing is offered.
    expect(dialog.textContent ?? '').not.toMatch(/\d chats/)
    expect(handlerArgs('remove_workspace')).toEqual([])
    // The action row is still there, and Cancel is the way out of it.
    const buttons = within(dialog).getAllByRole('button').map((button) => button.textContent?.trim())
    expect(buttons).toEqual(['Cancel'])
    await fireEvent.click(within(dialog).getByRole('button', { name: 'Cancel' }))
    await vi.waitFor(() => expect(dialog.isConnected).toBe(false))
  })

  it('sends nothing when the user cancels', async () => {
    serve()
    const dialog = await openDialog()

    await fireEvent.click(within(dialog).getByRole('button', { name: 'Cancel' }))

    await vi.waitFor(() => expect(dialog.isConnected).toBe(false))
    expect(handlerArgs('remove_workspace')).toEqual([])
  })

  it('sends nothing when the dialog is dismissed with Escape', async () => {
    serve()
    const dialog = await openDialog()

    await fireEvent.keyDown(document.body, { key: 'Escape' })

    await vi.waitFor(() => expect(dialog.isConnected).toBe(false))
    expect(handlerArgs('remove_workspace')).toEqual([])
  })

  it('sends the path with the confirmation, once, and closes on the answer', async () => {
    serve()
    const pending = deferred<unknown>()
    setHandler('remove_workspace', () => pending.promise)
    const dialog = await openDialog()

    const confirm = within(dialog).getByRole('button', { name: CONFIRM })
    await fireEvent.click(confirm)
    await fireEvent.click(confirm)

    expect(handlerArgs('remove_workspace')).toEqual([{ path: PATH, confirmed: true }])
    expect(dialog.isConnected).toBe(true)

    pending.resolve(viewState({ total_sessions: 0 }))
    await vi.waitFor(() => expect(dialog.isConnected).toBe(false))
  })

  it('stays open with a line of its own when the removal is rejected (R3-003)', async () => {
    serve()
    const pending = deferred<unknown>()
    setHandler('remove_workspace', () => pending.promise)
    const dialog = await openDialog()

    await fireEvent.click(within(dialog).getByRole('button', { name: CONFIRM }))
    // The shape Tauri rejects a Rust `Err` with: the reason itself, not an Error.
    pending.reject('the removal could not run')

    const alert = await within(dialog).findByRole('alert')
    expect(alert.textContent).toBe(REMOVE_FAILED)
    expect(alert.querySelector('b')?.textContent).toBe('Nothing more will be deleted')
    // A rejected command is not a removal: it is neither closed over nor resent.
    expect(dialog.isConnected).toBe(true)
    expect(handlerArgs('remove_workspace')).toEqual([{ path: PATH, confirmed: true }])

    // The user's own second answer is a retry, and the answer it gets closes it.
    setHandler('remove_workspace', () => viewState({ total_sessions: 0 }))
    await fireEvent.click(within(dialog).getByRole('button', { name: CONFIRM }))
    await vi.waitFor(() => expect(dialog.isConnected).toBe(false))
    expect(handlerArgs('remove_workspace')).toHaveLength(2)
  })
})
