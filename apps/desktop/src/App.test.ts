// The shell's branches this slice adds: no foreground chat keeps the placeholder;
// a foreground chat draws its chrome (the header title, the restart notice), its
// thread and its composer.

import './test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import { activity, resetTauriMock, restartNotice, viewState } from './test-support/tauri-mock'
import { serveSidebar } from './test-support/sidebar'
import App from './App.svelte'

vi.mock('@tauri-apps/api/core', () => import('./test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('./test-support/tauri-mock').then((m) => m.eventModule()))

const MESSAGE = {
  id: 'msg-1',
  role: 'User' as const,
  text: 'hello from the pushed thread',
  streaming: false,
  error: null,
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
})
