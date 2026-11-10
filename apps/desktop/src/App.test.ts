// The shell's branches: no foreground chat keeps the placeholder; a foreground
// chat draws its chrome (the header title, its notices), its thread and its
// composer; a pending extension dialog blocks the composer over a thread that
// keeps updating; and the fatal launcher bar belongs to no chat at all.

import './test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen, waitFor } from '@testing-library/svelte'
import {
  activity,
  confirmDialog,
  plainNotice,
  resetTauriMock,
  restartNotice,
  viewState,
} from './test-support/tauri-mock'
import { serveSidebar } from './test-support/sidebar'
import App from './App.svelte'

vi.mock('@tauri-apps/api/core', () => import('./test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('./test-support/tauri-mock').then((m) => m.eventModule()))
vi.mock('@tauri-apps/api/webview', () => import('./test-support/webview').then((m) => m.webviewModule()))

const MESSAGE = {
  id: 'msg-1',
  role: 'User' as const,
  text: 'hello from the pushed thread',
  streaming: false,
  error: null,
  images: [],
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

describe('App', () => {
  it('keeps the No chat open placeholder while no chat is foreground', async () => {
    serveSidebar({ view: viewState({ foreground_chat: null, messages: [] }) })
    render(App)

    expect(await screen.findByText('No chat open')).toBeTruthy()
    expect(screen.queryByRole('region', { name: 'Chat thread' })).toBeNull()
  })

  it('draws the foreground chat thread instead of the placeholder', async () => {
    serveSidebar({
      view: viewState({ foreground_chat: 'chat-1', messages: [MESSAGE] }),
    })
    render(App)

    expect(await screen.findByText(MESSAGE.text)).toBeTruthy()
    expect(screen.queryByText('No chat open')).toBeNull()
    expect(screen.getByRole('region', { name: 'Chat thread' })).toBeTruthy()
  })

  it('draws the open chat as its header, its thread and its composer', async () => {
    serveSidebar({
      view: viewState({
        foreground_chat: 'chat-1',
        foreground_title: 'Tauri skeleton: window and bridge',
        messages: [MESSAGE],
      }),
    })
    render(App)

    expect(
      await screen.findByRole('heading', { name: 'Tauri skeleton: window and bridge' }),
    ).toBeTruthy()
    expect(screen.getByRole('region', { name: 'Chat thread' })).toBeTruthy()
    expect(screen.getByRole('textbox', { name: 'Message' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Send' })).toBeTruthy()
  })

  it('shows the restart notice of a dead chat above its thread', async () => {
    serveSidebar({
      view: viewState({
        foreground_chat: 'c:chat-1',
        foreground_title: 'Dead one',
        messages: [MESSAGE],
        activity: [activity('c:chat-1', 'Ended')],
        notices: [restartNotice(1, 'Chat died: the child exited.', 'c:chat-1')],
      }),
    })
    render(App)

    const alert = await screen.findByRole('alert')
    expect(alert.textContent).toContain('Chat died: the child exited.')
    expect(screen.getByRole('button', { name: 'Restart chat' })).toBeTruthy()
    // The ended chat takes no prompt until it is restarted, and says why.
    expect(screen.getByRole('button', { name: 'Send' }).getAttribute('aria-disabled')).toBe('true')
  })

  it('draws a notice of every level, with its level in words', async () => {
    serveSidebar({
      view: viewState({
        foreground_chat: 'c:chat-1',
        foreground_title: 'Quiet one',
        messages: [MESSAGE],
        notices: [plainNotice(1, 'Could not read 3 chat files.', 'Warning')],
      }),
    })
    render(App)

    expect(await screen.findByText('Could not read 3 chat files.')).toBeTruthy()
    expect(screen.getByText('Warning')).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Dismiss' })).toBeTruthy()
  })

  it('draws no info notice, only the warning beside it', async () => {
    serveSidebar({
      view: viewState({
        foreground_chat: 'c:chat-1',
        foreground_title: 'Quiet one',
        messages: [MESSAGE],
        notices: [
          plainNotice(1, 'Skill registry refreshed (19 skills)', 'Info'),
          plainNotice(2, 'Could not read 3 chat files.', 'Warning'),
        ],
      }),
    })
    render(App)

    expect(await screen.findByText('Could not read 3 chat files.')).toBeTruthy()
    expect(screen.queryByText('Skill registry refreshed (19 skills)')).toBeNull()
  })

  it('asks the welcome question instead of the app while a choice is due', async () => {
    serveSidebar({
      view: viewState({
        shell: { pi_path: 'C:/Users/someone/.pi/agent', choice: null, ask: true, launcher_found: true, install: { type: 'Idle' }, restart_needed: false },
      }),
    })
    render(App)

    expect(await screen.findByRole('heading', { name: 'Welcome to gentle shell' })).toBeTruthy()
    expect(screen.queryByRole('button', { name: 'New session' })).toBeNull()
  })

  it('asks which home first when pi was found, even with no gentle-shell yet', async () => {
    // Someone with pi but without gentle-shell must be recognised for what they
    // have: the question about which home comes first, and only then the ask to
    // install the agent.
    serveSidebar({
      view: viewState({
        shell: {
          pi_path: 'C:/Users/someone/.pi/agent',
          choice: null,
          ask: true,
          launcher_found: false,
          install: { type: 'Idle' },
          restart_needed: false,
        },
      }),
    })
    render(App)

    expect(await screen.findByRole('heading', { name: 'Welcome to gentle shell' })).toBeTruthy()
    expect(screen.queryByRole('heading', { name: 'gentle-shell is not installed' })).toBeNull()
  })

  it('asks to install gentle-shell when nothing at all was found', async () => {
    serveSidebar({
      view: viewState({
        shell: {
          pi_path: null,
          choice: null,
          ask: false,
          launcher_found: false,
          install: { type: 'Idle' },
          restart_needed: false,
        },
      }),
    })
    render(App)

    expect(await screen.findByRole('heading', { name: 'gentle-shell is not installed' })).toBeTruthy()
    expect(screen.queryByRole('button', { name: 'New session' })).toBeNull()
  })

  it('asks to install gentle-shell once the home is chosen and the agent is still missing', async () => {
    serveSidebar({
      view: viewState({
        shell: {
          pi_path: 'C:/Users/someone/.pi/agent',
          choice: 'link',
          ask: false,
          launcher_found: false,
          install: { type: 'Idle' },
          restart_needed: false,
        },
      }),
    })
    render(App)

    expect(await screen.findByRole('heading', { name: 'gentle-shell is not installed' })).toBeTruthy()
  })

  it('draws the app and no question once a home is chosen', async () => {
    serveSidebar({
      view: viewState({
        shell: { pi_path: 'C:/Users/someone/.pi/agent', choice: 'link', ask: false, launcher_found: true, install: { type: 'Idle' }, restart_needed: false },
      }),
    })
    render(App)

    expect(await screen.findByText('No chat open')).toBeTruthy()
    expect(screen.queryByRole('heading', { name: 'Welcome to gentle shell' })).toBeNull()
  })

  it('shows the fatal launcher bar above everything, with no chat open', async () => {
    serveSidebar({
      view: viewState({ foreground_chat: null, fatal: 'gentle-shell is not available' }),
    })
    render(App)

    const alert = await screen.findByRole('alert')
    expect(alert.textContent).toContain('gentle-shell is not available')
    expect(screen.getByText('No chat open')).toBeTruthy()
    expect(screen.queryByRole('button', { name: 'Dismiss' })).toBeNull()
  })

  it('draws no fatal bar when the launcher resolved', async () => {
    serveSidebar({ view: viewState({ foreground_chat: null, fatal: null }) })
    render(App)

    expect(await screen.findByText('No chat open')).toBeTruthy()
    expect(screen.queryByRole('alert')).toBeNull()
  })

  it('blocks the composer with a pending dialog while the thread keeps updating', async () => {
    const fixture = serveSidebar({
      view: viewState({
        foreground_chat: 'chat-1',
        foreground_title: 'Waiting one',
        messages: [MESSAGE],
        dialog: confirmDialog('d1'),
      }),
    })
    render(App)

    const modal = await screen.findByRole('dialog')
    expect(modal.getAttribute('aria-modal')).toBe('true')
    // The focus opens inside the modal, never in the composer behind it.
    await waitFor(() => expect(document.activeElement?.closest('[role="dialog"]')).toBe(modal))

    // The state keeps arriving behind it: the answer waits, the thread does not.
    fixture.view = viewState({
      foreground_chat: 'chat-1',
      foreground_title: 'Waiting one',
      messages: [MESSAGE, { ...MESSAGE, id: 'msg-2', text: 'a second line' }],
      dialog: confirmDialog('d1'),
    })
    fixture.push()

    expect(await screen.findByText('a second line')).toBeTruthy()
    expect(screen.getByRole('dialog')).toBeTruthy()
  })
})
