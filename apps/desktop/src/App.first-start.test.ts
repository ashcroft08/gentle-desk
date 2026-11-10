// The first-start screens of the shell: which home to use, and the ask to
// install gentle-shell. Kept apart from App.test.ts because these cases are
// about what the window holds before any chat exists, and together they would
// push that file past the line limit.

import './test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import { resetTauriMock, viewState } from './test-support/tauri-mock'
import { serveSidebar } from './test-support/sidebar'
import App from './App.svelte'

vi.mock('@tauri-apps/api/core', () => import('./test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('./test-support/tauri-mock').then((m) => m.eventModule()))
vi.mock('@tauri-apps/api/webview', () => import('./test-support/webview').then((m) => m.webviewModule()))

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

describe('the first-start screens', () => {
  it('keeps the No chat open placeholder while no chat is foreground', async () => {
    serveSidebar({ view: viewState({ foreground_chat: null, messages: [] }) })
    render(App)

    expect(await screen.findByText('No chat open')).toBeTruthy()
    expect(screen.queryByRole('region', { name: 'Chat thread' })).toBeNull()
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
})
