// The shell's one branch this slice adds: no foreground chat keeps the
// placeholder, a foreground chat draws its thread in the main area.

import './test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import { resetTauriMock, viewState } from './test-support/tauri-mock'
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
})
