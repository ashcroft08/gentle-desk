import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import { handlerArgs, resetTauriMock, viewState, workspaceRow, chatRow } from '../test-support/tauri-mock'
import { serveSidebar, type SidebarFixture } from '../test-support/sidebar'
import Sidebar from './Sidebar.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The words the design draws, quoted here so a change of copy is a change here. */
const EMPTY_TITLE = 'No workspaces yet'
const EMPTY_REASON =
  'Use New workspace to choose a folder or create a new one. A new session needs a workspace.'
const NO_WORKSPACE = 'Select a workspace to start a session.'

let fixture: SidebarFixture

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

describe('Sidebar', () => {
  it('draws the first-run empty state and points New session at its reason', async () => {
    fixture = serveSidebar({
      view: viewState({ list_status: { type: 'Complete' }, total_sessions: 0 }),
    })
    render(Sidebar)

    const reason = await screen.findByText(EMPTY_REASON)
    expect(screen.getByText(EMPTY_TITLE)).toBeTruthy()
    const button = screen.getByRole('button', { name: 'New session' })
    expect(button.getAttribute('aria-disabled')).toBe('true')
    expect(button.getAttribute('aria-describedby')).toBe(reason.id)
    expect(reason.id).toBe('workspace-empty-why')
  })

  it('says it is still reading instead of claiming there is no workspace', async () => {
    fixture = serveSidebar({
      view: viewState({ list_status: { type: 'Loading', data: { done: 4, total: 9 } } }),
      rows: [workspaceRow('repo')],
      total: 1,
    })
    render(Sidebar)

    await screen.findByText('Reading chats... 4 / 9')
    expect(screen.queryByText(EMPTY_TITLE)).toBeNull()
    expect(screen.queryByText(EMPTY_REASON)).toBeNull()
  })

  it('shows the config and workspace notices as plain status lines', async () => {
    fixture = serveSidebar({
      view: viewState({
        config_notice: 'The config file was set aside.',
        workspace_notice: 'That folder is not there.',
      }),
    })
    render(Sidebar)

    const config = await screen.findByText('The config file was set aside.')
    const workspace = screen.getByText('That folder is not there.')
    expect(config.getAttribute('role')).toBe('status')
    expect(workspace.getAttribute('role')).toBe('status')
  })

  it('disables New session with its reason until a workspace is current, then names its path', async () => {
    fixture = serveSidebar({
      view: viewState({ total_sessions: 1 }),
      rows: [workspaceRow('repo', { path: 'C:\\dev\\repo' }), chatRow('c1', { title: 'First chat' })],
      total: 2,
    })
    render(Sidebar)

    const reason = await screen.findByText(NO_WORKSPACE)
    const button = screen.getByRole('button', { name: 'New session' })
    expect(button.getAttribute('aria-disabled')).toBe('true')
    expect(button.getAttribute('aria-describedby')).toBe(reason.id)
    await fireEvent.click(button)
    expect(fixture.intents()).toEqual([])

    await fireEvent.click(screen.getByText('First chat'))
    expect(fixture.intents()).toEqual([{ type: 'OpenChat', data: { chat_id: 'c1' } }])

    await vi.waitFor(() => expect(button.getAttribute('aria-disabled')).toBeNull())
    await fireEvent.click(button)
    expect(fixture.intents().at(-1)).toEqual({
      type: 'NewChatInWorkspace',
      data: { path: 'C:\\dev\\repo' },
    })
  })

  it('adds a workspace through the New workspace menu', async () => {
    fixture = serveSidebar({
      view: viewState({ total_sessions: 1 }),
      rows: [workspaceRow('repo')],
      total: 1,
    })
    render(Sidebar)

    await screen.findByText('repo')
    await fireEvent.click(screen.getByRole('button', { name: 'New workspace' }))
    const menu = await screen.findByRole('menu')
    await fireEvent.click(
      within(menu).getByRole('menuitem', { name: 'Choose an existing folder' }),
    )
    await vi.waitFor(() => expect(handlerArgs('add_workspace').length).toBe(1))
  })

  it('marks the run Preview in the footer, with the promise in its tooltip', async () => {
    fixture = serveSidebar({ view: viewState({ total_sessions: 0 }) })
    render(Sidebar)

    const chip = await screen.findByText('Preview')
    expect(chip.getAttribute('title')).toBe(
      'Preview: under active development - fixes ship fast',
    )
  })

  it('keeps the footer to the Preview chip and the Settings gear', async () => {
    // A pi is on the machine, so the home setting would be drawn if it still
    // lived here: the footer draws the chip and the gear and nothing else (S1).
    fixture = serveSidebar({
      view: viewState({
        total_sessions: 0,
        shell: {
          pi_path: 'C:/Users/someone/.pi/agent',
          choice: 'link',
          ask: false,
          launcher_found: true,
          install: { type: 'Idle' },
          restart_needed: false,
        },
      }),
    })
    const { container } = render(Sidebar)

    const gear = await screen.findByRole('button', { name: 'Settings' })
    const chip = screen.getByText('Preview')
    const foot = container.querySelector('.foot')
    expect(foot).not.toBeNull()
    expect(foot?.contains(chip)).toBe(true)
    expect(foot?.contains(gear)).toBe(true)
    // The two controls that live in the Settings dialog now are not drawn until
    // the user opens it.
    expect(screen.queryByRole('button', { name: /^Theme: / })).toBeNull()
    expect(screen.queryByRole('group', { name: 'Home' })).toBeNull()
  })
})
