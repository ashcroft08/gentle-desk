// The sidebar's New session with NO workspace selected (welcome-screen R1): it
// falls back to the very workspace view A's primary action names, so the button
// is not left mute beside a main pane that already offers to open a session.
// Rust words the refusal of a folder that is gone, so it is printed verbatim.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import { resetTauriMock, viewState, chatRow, workspaceRow } from '../test-support/tauri-mock'
import { serveSidebar, type SidebarFixture } from '../test-support/sidebar'
import { welcomeView, welcomeWorkspace } from '../test-support/welcome'
import Sidebar from './Sidebar.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The words the design draws, quoted here so a change of copy is a change here. */
const NO_WORKSPACE = 'Select a workspace to start a session.'
const EMPTY_REASON =
  'Use New workspace to choose a folder or create a new one. A new session needs a workspace.'
const GONE = 'This folder no longer exists.'
const LAST = 'C:\\dev\\gentle-desk'

let fixture: SidebarFixture

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

/** The tree of a run that worked in `LAST` and has one other workspace too. */
function servedView(view: ReturnType<typeof viewState>): SidebarFixture {
  return serveSidebar({
    view,
    rows: [workspaceRow('gentle-desk', { path: LAST }), chatRow('c1', { title: 'First chat' })],
    total: 2,
  })
}

describe('New session without a selected workspace', () => {
  it('falls back to the last workspace and starts the chat there', async () => {
    fixture = servedView(viewState({ total_sessions: 2, welcome: welcomeView() }))
    render(Sidebar)

    const button = await vi.waitFor(() =>
      screen.getByRole('button', { name: 'New session' }),
    )
    await vi.waitFor(() => expect(button.getAttribute('aria-disabled')).toBeNull())
    // Nothing explains a disabled button, because it is not disabled: no line and
    // no `aria-describedby`.
    expect(screen.queryByText(NO_WORKSPACE)).toBeNull()
    expect(button.getAttribute('aria-describedby')).toBeNull()

    await fireEvent.click(button)
    expect(fixture.intents()).toEqual([
      { type: 'NewChatInWorkspace', data: { path: LAST } },
    ])
  })

  it('keeps the selected workspace and ignores the fallback', async () => {
    // The workspace of the foreground chat wins, so the last workspace is only
    // ever a fallback: here the chat clicked names another folder.
    fixture = servedView(
      viewState({
        total_sessions: 2,
        foreground_chat: 'c1',
        welcome: welcomeView({ last: welcomeWorkspace({ path: 'C:\\dev\\other' }) }),
      }),
    )
    render(Sidebar)

    const button = await vi.waitFor(() =>
      screen.getByRole('button', { name: 'New session' }),
    )
    await vi.waitFor(() => expect(button.getAttribute('aria-disabled')).toBeNull())
    expect(screen.queryByText(NO_WORKSPACE)).toBeNull()

    await fireEvent.click(button)
    expect(fixture.intents()).toEqual([
      { type: 'NewChatInWorkspace', data: { path: 'C:\\dev\\gentle-desk' } },
    ])
  })

  it('refuses a last workspace that is gone and prints the reason Rust worded', async () => {
    fixture = servedView(
      viewState({
        total_sessions: 2,
        welcome: welcomeView({
          last: welcomeWorkspace({ path: LAST, available: false, reason: GONE }),
        }),
      }),
    )
    render(Sidebar)

    const reason = await screen.findByText(GONE)
    const button = screen.getByRole('button', { name: 'New session' })
    expect(button.getAttribute('aria-disabled')).toBe('true')
    expect(button.getAttribute('aria-describedby')).toBe(reason.id)
    expect(reason.id).toBe('new-session-why')
    // The fallback's own words, never the sidebar's: Rust already localized it.
    expect(screen.queryByText(NO_WORKSPACE)).toBeNull()

    await fireEvent.click(button)
    expect(fixture.intents()).toEqual([])
  })

  it('says what it always said when there is no last workspace at all', async () => {
    fixture = servedView(viewState({ total_sessions: 2, welcome: null }))
    render(Sidebar)

    const reason = await screen.findByText(NO_WORKSPACE)
    const button = screen.getByRole('button', { name: 'New session' })
    expect(button.getAttribute('aria-disabled')).toBe('true')
    expect(button.getAttribute('aria-describedby')).toBe(reason.id)

    await fireEvent.click(button)
    expect(fixture.intents()).toEqual([])
  })

  it('says the same when the welcome names no last workspace', async () => {
    // No configured workspace: there is nothing to be last in.
    fixture = servedView(
      viewState({ total_sessions: 2, welcome: welcomeView({ last: null }) }),
    )
    render(Sidebar)

    const reason = await screen.findByText(NO_WORKSPACE)
    const button = screen.getByRole('button', { name: 'New session' })
    expect(button.getAttribute('aria-disabled')).toBe('true')
    expect(button.getAttribute('aria-describedby')).toBe(reason.id)
  })

  it('leaves the first-run empty state pointing at its own reason', async () => {
    fixture = serveSidebar({
      view: viewState({ list_status: { type: 'Complete' }, total_sessions: 0 }),
    })
    render(Sidebar)

    const reason = await screen.findByText(EMPTY_REASON)
    const button = screen.getByRole('button', { name: 'New session' })
    expect(reason.id).toBe('workspace-empty-why')
    expect(button.getAttribute('aria-disabled')).toBe('true')
    expect(button.getAttribute('aria-describedby')).toBe(reason.id)
  })
})
