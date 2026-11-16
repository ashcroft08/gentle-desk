// Renaming the open chat from the header (chat-chrome-helpers S2): the pencil
// asks for the same write the row's inline rename asks for - the session path
// the state carries and the name the user typed, through the same
// `rename_session` command - so the header is a second door to one intent, not a
// second rename. The row's own door is pinned in `sidebar/ChatRename.test.ts`.

import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import { handlerArgs, resetTauriMock, viewState } from '../test-support/tauri-mock'
import { serveSidebar } from '../test-support/sidebar'
import App from '../App.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

const TITLE = 'Tauri skeleton: window and bridge'
const SESSION = 'C:\\pi\\sessions\\--dev-gentle-desk--\\2026_s1.jsonl'

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

describe('renaming the open chat from the header', () => {
  it('sends the typed name for the foreground chat on Enter', async () => {
    serveSidebar({
      view: viewState({ foreground_chat: SESSION, foreground_title: TITLE }),
    })
    render(App)
    await screen.findByRole('heading', { name: TITLE })

    await fireEvent.click(screen.getByRole('button', { name: 'Rename chat' }))
    const input = await screen.findByRole('textbox', { name: 'Chat name' })
    await fireEvent.input(input, { target: { value: 'Season two' } })
    await fireEvent.keyDown(input, { key: 'Enter' })

    expect(handlerArgs('rename_session')).toEqual([{ sessionId: SESSION, name: 'Season two' }])
  })

  it('sends nothing when the rename is cancelled', async () => {
    serveSidebar({
      view: viewState({ foreground_chat: SESSION, foreground_title: TITLE }),
    })
    render(App)
    await screen.findByRole('heading', { name: TITLE })

    await fireEvent.click(screen.getByRole('button', { name: 'Rename chat' }))
    const input = await screen.findByRole('textbox', { name: 'Chat name' })
    await fireEvent.input(input, { target: { value: 'Never sent' } })
    await fireEvent.keyDown(input, { key: 'Escape' })

    expect(handlerArgs('rename_session')).toEqual([])
  })
})
