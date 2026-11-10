// The Chat | Helpers views of the shell (T7b-1), one test per rule: no chat
// means no tabs, the header's tabs switch the two panels, the composer keeps
// its draft and the thread keeps its place across a trip to Helpers, and the
// helpers panel follows the pushed state.

import './test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte'
import { serveSidebar } from './test-support/sidebar'
import { helperRow, helpersPanel } from './test-support/helpers'
import { resetTauriMock, viewState } from './test-support/tauri-mock'
import App from './App.svelte'

vi.mock('@tauri-apps/api/core', () => import('./test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('./test-support/tauri-mock').then((m) => m.eventModule()))

/** A state with the foreground chat open and its helpers panel filled. */
function open(over: Parameters<typeof viewState>[0] = {}) {
  return viewState({
    foreground_chat: 'chat-1',
    foreground_title: 'Tauri skeleton: window and bridge',
    helpers: helpersPanel([helperRow('h1', { title: 'Inventory of sidebar modules' })], {
      running: 1,
    }),
    ...over,
  })
}

/** Clicks a tab of the header and waits for the panels to follow. */
async function tab(name: RegExp): Promise<void> {
  await fireEvent.click(screen.getByRole('tab', { name }))
  await waitFor(() => expect(screen.getByRole('tab', { name }).getAttribute('aria-selected')).toBe('true'))
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

describe('the Chat | Helpers views', () => {
  it('draws no tabs while no chat is open', async () => {
    serveSidebar({ view: viewState() })

    render(App)

    await waitFor(() => expect(screen.getByText('No chat open')).toBeTruthy())
    expect(screen.queryByRole('tablist')).toBeNull()
    expect(screen.queryByRole('tab')).toBeNull()
  })

  it('switches the Chat and Helpers panels from the header tablist', async () => {
    serveSidebar({ view: open() })

    render(App)

    await waitFor(() => expect(screen.getByRole('tablist', { name: 'View' })).toBeTruthy())
    // The Chat view is the one drawn first: its thread and its composer.
    expect(screen.getByRole('region', { name: 'Chat thread' })).toBeTruthy()
    expect(screen.getByRole('textbox', { name: 'Message' })).toBeTruthy()
    expect(screen.queryByRole('listbox', { name: 'Helpers' })).toBeNull()

    await tab(/Helpers/)

    expect(screen.getByRole('listbox', { name: 'Helpers' })).toBeTruthy()
    expect(screen.getByRole('option')).toBeTruthy()
    // The Helpers view has no composer, as 01b draws it.
    expect(screen.queryByRole('textbox')).toBeNull()
  })

  it('keeps the composer draft across a trip to Helpers and back', async () => {
    serveSidebar({ view: open() })
    render(App)
    const box = await screen.findByRole('textbox', { name: 'Message' })
    await fireEvent.input(box, { target: { value: 'half a prompt' } })

    await tab(/Helpers/)
    await tab(/^Chat$/)

    expect((screen.getByRole('textbox', { name: 'Message' }) as HTMLTextAreaElement).value).toBe(
      'half a prompt',
    )
  })

  it('keeps the thread where the reader left it across a trip to Helpers', async () => {
    serveSidebar({
      view: open({
        messages: [
          { id: 'm1', role: 'Assistant', text: 'a long answer', streaming: false, error: null },
        ],
      }),
    })
    render(App)
    const region = await screen.findByRole('region', { name: 'Chat thread' })
    // jsdom lays nothing out, so the reader's place is faked the way a tall
    // thread reports it: more content than the box shows, scrolled up.
    Object.defineProperty(region, 'scrollHeight', { value: 2000, configurable: true })
    Object.defineProperty(region, 'clientHeight', { value: 400, configurable: true })
    region.scrollTop = 120
    await fireEvent.scroll(region)
    expect(region.scrollTop).toBe(120)

    await tab(/Helpers/)
    await tab(/^Chat$/)

    expect(screen.getByRole('region', { name: 'Chat thread' }).scrollTop).toBe(120)
  })

  it('draws the helpers the pushed state carries when it changes', async () => {
    const fixture = serveSidebar({ view: open() })
    render(App)
    await screen.findByRole('tablist', { name: 'View' })
    await tab(/Helpers/)
    expect(screen.queryByRole('option', { name: /Bridge event types/ })).toBeNull()

    fixture.view = open({
      helpers: helpersPanel(
        [
          helperRow('h1', { title: 'Inventory of sidebar modules' }),
          helperRow('h2', { title: 'Bridge event types', status: 'Waiting', status_label: 'waiting' }),
        ],
        { running: 1 },
      ),
    })
    fixture.push()

    await waitFor(() => expect(screen.getAllByRole('option')).toHaveLength(2))
    expect(screen.getByRole('option', { name: /Bridge event types/ })).toBeTruthy()
  })
})
