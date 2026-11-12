// The what's-new slice reaches the window through ONE door (T2, S4): the modal.
// It is not a notice, so the sidebar draws none of it; it is not about a chat,
// so it stands whatever chat is open; and the dismiss is the one command the
// web side sends for it. Rust's next push (the cleared slice) ends the modal.
import './test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import { handlerArgs, resetTauriMock, setHandler, viewState } from './test-support/tauri-mock'
import { serveSidebar } from './test-support/sidebar'
import App from './App.svelte'

vi.mock('@tauri-apps/api/core', () => import('./test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('./test-support/tauri-mock').then((m) => m.eventModule()))

const VERSION = '0.1.8'
const TITLE = "What's new in gentle-desk 0.1.8"
const NOTE = 'A dropped image can be taken away before it is sent.'

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

describe('The what\u2019s-new modal in the shell', () => {
  it('shows the pushed notes in the modal, and nowhere in the sidebar', async () => {
    serveSidebar({
      view: viewState({
        foreground_chat: 'chat-1',
        foreground_title: 'Some chat',
        whats_new: { version: VERSION, notes: NOTE },
      }),
    })
    render(App)

    const dialog = await screen.findByRole('dialog')
    expect(within(dialog).getByText(TITLE)).toBeTruthy()
    expect(within(dialog).getByText(NOTE)).toBeTruthy()

    // The slice is not a notice: no notice strip is drawn, and the only line
    // that says it is the modal's own.
    expect(document.querySelector('.notices')).toBeNull()
    expect(screen.getAllByText(NOTE)).toHaveLength(1)
  })

  it('shows no modal while the run follows no update', async () => {
    serveSidebar({ view: viewState({ whats_new: null }) })
    render(App)

    expect(await screen.findByText('No chat open')).toBeTruthy()
    expect(screen.queryByRole('dialog')).toBeNull()
  })

  it('sends the dismiss once, and the push that clears the slice ends it', async () => {
    const fixture = serveSidebar({
      view: viewState({ whats_new: { version: VERSION, notes: NOTE } }),
    })
    setHandler('store_last_run_version', () => viewState({ whats_new: null }))
    render(App)

    const dialog = await screen.findByRole('dialog')
    await fireEvent.click(within(dialog).getByRole('button', { name: 'Got it' }))

    await vi.waitFor(() => expect(dialog.isConnected).toBe(false))
    expect(handlerArgs('store_last_run_version')).toHaveLength(1)

    // Rust stores the version and pushes the cleared slice; nothing comes back.
    fixture.view = viewState({ whats_new: null })
    fixture.push()
    await vi.waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
  })
})
