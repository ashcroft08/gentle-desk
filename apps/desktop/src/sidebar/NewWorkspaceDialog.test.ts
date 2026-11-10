// The create-folder dialog, which no artboard draws: a name, Cancel and Create.
// Rust validates the name and opens the parent picker (workspaces-design Q6),
// so this side only refuses the empty name; a refusal comes back as the
// pushed `workspace_notice`, which the sidebar draws after the dialog closes.
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
} from '../test-support/tauri-mock'
import { serveSidebar, type SidebarFixture } from '../test-support/sidebar'
import Sidebar from './Sidebar.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

const CREATE_ITEM = 'Create a new folder'
const NAME_LABEL = 'Folder name'
const NOTICE = 'That folder is not there.'

let fixture: SidebarFixture

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  fixture = serveSidebar({
    view: viewState({ total_sessions: 1 }),
    rows: [workspaceRow('repo')],
    total: 1,
  })
  render(Sidebar)
})

function trigger(): HTMLElement {
  return screen.getByRole('button', { name: 'New workspace' })
}

function nameInput(): HTMLElement {
  return screen.getByLabelText(NAME_LABEL)
}

function createButton(): HTMLElement {
  return screen.getByRole('button', { name: 'Create' })
}

async function openDialog(): Promise<HTMLElement> {
  await fireEvent.click(trigger())
  const menu = await screen.findByRole('menu')
  await fireEvent.click(within(menu).getByRole('menuitem', { name: CREATE_ITEM }))
  const dialog = await screen.findByRole('dialog')
  await vi.waitFor(() => expect(document.activeElement).toBe(nameInput()))
  return dialog
}

async function typeName(value: string): Promise<void> {
  await fireEvent.input(nameInput(), { target: { value } })
}

async function closed(dialog: HTMLElement): Promise<void> {
  await vi.waitFor(() => expect(dialog.isConnected).toBe(false))
}

describe('Create a new folder', () => {
  it('asks for the name in a labelled input that holds the focus', async () => {
    const dialog = await openDialog()
    expect(dialog.getAttribute('aria-modal')).toBe('true')
    expect(nameInput().tagName).toBe('INPUT')
    expect(document.activeElement).toBe(nameInput())
  })

  it('keeps Create disabled while the trimmed name is empty', async () => {
    await openDialog()
    expect(createButton().hasAttribute('disabled')).toBe(true)

    await typeName('   ')
    expect(createButton().hasAttribute('disabled')).toBe(true)

    await typeName('repo')
    expect(createButton().hasAttribute('disabled')).toBe(false)
  })

  it('creates once with the trimmed name, busy, and closes when the command resolves', async () => {
    const pending = deferred<unknown>()
    setHandler('create_workspace', () => pending.promise)
    const dialog = await openDialog()

    await typeName('  new folder  ')
    const create = createButton()
    await fireEvent.click(create)
    await fireEvent.click(create)

    expect(handlerArgs('create_workspace').map((args) => args.name)).toEqual(['new folder'])
    await vi.waitFor(() => {
      const busy = screen.getByRole('button', { name: 'Creating...' })
      expect(busy.hasAttribute('disabled')).toBe(true)
    })

    pending.resolve(fixture.view)
    await closed(dialog)
    expect(document.activeElement).toBe(trigger())
  })

  it('closes on Cancel without creating and gives the focus back to the icon', async () => {
    const dialog = await openDialog()
    await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }))

    await closed(dialog)
    expect(handlerArgs('create_workspace')).toEqual([])
    expect(document.activeElement).toBe(trigger())
  })

  it('closes on Escape without creating and gives the focus back to the icon', async () => {
    const dialog = await openDialog()
    await fireEvent.keyDown(document.body, { key: 'Escape' })

    await closed(dialog)
    expect(handlerArgs('create_workspace')).toEqual([])
    expect(document.activeElement).toBe(trigger())
  })

  it('shows the refusal the next pushed state carries once the dialog is gone', async () => {
    const pending = deferred<unknown>()
    setHandler('create_workspace', () => pending.promise)
    const dialog = await openDialog()
    await typeName('repo')
    await fireEvent.click(createButton())

    fixture.view = viewState({ total_sessions: 1, workspace_notice: NOTICE })
    pending.resolve(fixture.view)
    await closed(dialog)
    fixture.push()

    const line = await screen.findByText(NOTICE)
    expect(line.getAttribute('role')).toBe('status')
  })
})
