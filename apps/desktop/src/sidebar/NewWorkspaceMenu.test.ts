// The New workspace menu (workspaces-design Q1 option A, tauri-migration L13):
// the icon opens exactly the two drawn items, the menu semantics the artboard
// spells out (01a), and "Choose an existing folder" is the W3c add path.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import { handlerArgs, resetTauriMock, viewState, workspaceRow } from '../test-support/tauri-mock'
import { serveSidebar } from '../test-support/sidebar'
import Sidebar from './Sidebar.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The two labels the artboard draws, quoted so a change of copy is a change here. */
const CHOOSE = 'Choose an existing folder'
const CREATE = 'Create a new folder'
const EMPTY_REASON =
  'Use New workspace to choose a folder or create a new one. A new session needs a workspace.'

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

function openTree(): void {
  serveSidebar({
    view: viewState({ total_sessions: 1 }),
    rows: [workspaceRow('repo')],
    total: 1,
  })
  render(Sidebar)
}

function trigger(): HTMLElement {
  return screen.getByRole('button', { name: 'New workspace' })
}

async function openMenu(): Promise<HTMLElement> {
  await fireEvent.click(trigger())
  return await screen.findByRole('menu')
}

/** The item the menu has moved the focus to (one ArrowDown reaches the first). */
async function focusedItem(label: string): Promise<void> {
  await vi.waitFor(() =>
    expect(document.activeElement).toBe(screen.getByRole('menuitem', { name: label })),
  )
}

function keydown(key: string): void {
  fireEvent.keyDown(document.activeElement ?? document.body, { key })
}

describe('New workspace menu', () => {
  it('is a menu button whose expanded state follows the menu', async () => {
    openTree()
    const button = trigger()
    expect(button.getAttribute('aria-haspopup')).toBe('menu')
    expect(button.getAttribute('aria-expanded')).toBe('false')

    await openMenu()
    expect(button.getAttribute('aria-expanded')).toBe('true')

    await fireEvent.keyDown(document.body, { key: 'Escape' })
    await vi.waitFor(() => expect(screen.queryByRole('menu')).toBeNull())
    expect(button.getAttribute('aria-expanded')).toBe('false')
  })

  it('opens exactly the two drawn items', async () => {
    openTree()
    const menu = await openMenu()
    const items = within(menu).getAllByRole('menuitem')
    expect(items.map((item) => item.textContent?.trim())).toEqual([CHOOSE, CREATE])
  })

  it('moves with the arrow keys and Home and End, and Enter activates the item', async () => {
    openTree()
    await openMenu()

    keydown('ArrowDown')
    await focusedItem(CHOOSE)
    keydown('ArrowDown')
    await focusedItem(CREATE)
    keydown('ArrowUp')
    await focusedItem(CHOOSE)
    keydown('Home')
    await focusedItem(CHOOSE)
    keydown('End')
    await focusedItem(CREATE)

    keydown('Enter')
    expect(await screen.findByRole('dialog')).toBeTruthy()
  })

  it('closes on Escape and gives the focus back to the icon', async () => {
    openTree()
    await openMenu()

    await fireEvent.keyDown(document.body, { key: 'Escape' })
    await vi.waitFor(() => expect(screen.queryByRole('menu')).toBeNull())
    expect(document.activeElement).toBe(trigger())
  })

  it('adds one folder for the choose item and closes the menu', async () => {
    openTree()
    const menu = await openMenu()
    await fireEvent.click(within(menu).getByRole('menuitem', { name: CHOOSE }))

    await vi.waitFor(() => expect(handlerArgs('add_workspace').length).toBe(1))
    await vi.waitFor(() => expect(screen.queryByRole('menu')).toBeNull())
    expect(handlerArgs('create_workspace')).toEqual([])
  })

  it('opens from the icon with the keyboard and activates with Space', async () => {
    openTree()
    trigger().focus()
    keydown('ArrowDown')
    await focusedItem(CHOOSE)

    keydown(' ')
    await vi.waitFor(() => expect(handlerArgs('add_workspace').length).toBe(1))
  })

  it('opens the same menu from the empty first-run prompt', async () => {
    serveSidebar({ view: viewState({ list_status: { type: 'Complete' } }) })
    render(Sidebar)

    const reason = await screen.findByText(EMPTY_REASON)
    const prompt = within(reason.parentElement as HTMLElement).getByRole('button', {
      name: 'New workspace',
    })
    await fireEvent.click(prompt)

    const menu = await screen.findByRole('menu')
    expect(within(menu).getAllByRole('menuitem')).toHaveLength(2)
  })
})
