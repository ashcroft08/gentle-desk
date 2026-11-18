// The shell's branches: no foreground chat keeps the placeholder; a foreground
// chat draws its chrome (the header title, its notices), its thread and its
// composer; a pending extension dialog blocks the composer over a thread that
// keeps updating; and the fatal launcher bar belongs to no chat at all.

import './test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte'
import { tick } from 'svelte'
import {
  activity,
  confirmDialog,
  plainNotice,
  resetTauriMock,
  restartNotice,
  viewState,
} from './test-support/tauri-mock'
import { serveSidebar } from './test-support/sidebar'
import { setLanguage } from './i18n/language.svelte'
import { appCopy } from './app-copy'
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

/** One tool row of the pushed transcript: the only shape `messages` cannot carry. */
const TOOL = {
  kind: 'Tool' as const,
  data: {
    id: 'tool-call-1',
    tool_name: 'bash',
    args: { command: 'ls node_modules' },
    partial_result: null,
    result: 'mocker.d.ts',
    is_error: false,
    running: false,
    duration_ms: 200,
    truncated: false,
    dropped: false,
  },
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

describe('App', () => {

  it('draws the foreground chat thread instead of the placeholder', async () => {
    serveSidebar({
      view: viewState({
        foreground_chat: 'chat-1',
        messages: [MESSAGE],
        items: [{ kind: 'Message', data: MESSAGE }, TOOL],
      }),
    })
    render(App)

    expect(await screen.findByText(MESSAGE.text)).toBeTruthy()
    expect(screen.queryByText('No chat open')).toBeNull()
    expect(screen.getByRole('region', { name: 'Chat thread' })).toBeTruthy()
    // The shell hands the thread the ROWS, not only the messages: a tool row is
    // drawn as its card, which `messages` alone could never produce (T2b).
    expect(screen.getByText('ls node_modules')).toBeTruthy()
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

describe("the shell's own words", () => {
  /** The English words, exactly as the shell drew them before 0.2.0. */
  const ENGLISH = {
    skipToChat: 'Skip to chat',
    closeSidebar: 'Close sidebar',
    noChatOpen: 'No chat open',
    noChatHint: 'Choose a chat in the workspace tree, or start a new session.',
  }

  /** The Spanish words: neutral Spanish, accents allowed. */
  const SPANISH = {
    skipToChat: 'Ir al chat',
    closeSidebar: 'Cerrar la barra lateral',
    noChatOpen: 'Ningún chat abierto',
    noChatHint: 'Elige un chat en el árbol de espacios de trabajo o inicia una sesión nueva.',
  }

  /** The shell's words in one language, as the main area draws them. */
  const WORDS = Object.keys(ENGLISH) as (keyof typeof ENGLISH)[]

  it('draws the English words the shell already had, and Spanish for the same shape', () => {
    expect(appCopy('en')).toEqual(ENGLISH)
    expect(appCopy('es')).toEqual(SPANISH)
  })

  it('translates every word: nothing of the shell is left in English', () => {
    const [english, spanish] = [appCopy('en'), appCopy('es')]
    for (const word of WORDS) expect(spanish[word], word).not.toBe(english[word])
  })

  it('flips the empty main area in the same tick, with no push and no restart', async () => {
    setLanguage('en')
    serveSidebar({ view: viewState({ foreground_chat: null }) })
    render(App)

    expect(await screen.findByText(ENGLISH.noChatOpen)).toBeTruthy()
    expect(screen.getByText(ENGLISH.noChatHint)).toBeTruthy()
    expect(screen.getByRole('link', { name: ENGLISH.skipToChat })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(screen.getByText(SPANISH.noChatOpen)).toBeTruthy()
    expect(screen.getByText(SPANISH.noChatHint)).toBeTruthy()
    expect(screen.getByRole('link', { name: SPANISH.skipToChat })).toBeTruthy()

    setLanguage('en')
  })

  it('flips the floating sidebar scrim, and leaves the chat title exactly as Rust sent it', async () => {
    setLanguage('en')
    // A narrow window: opening the sidebar by hand floats it over the chat, with
    // the scrim that closes it (responsive-sidebar S3).
    Object.defineProperty(window, 'innerWidth', { value: 600, configurable: true })
    const TITLE = 'Tauri skeleton: window and bridge'
    serveSidebar({
      view: viewState({ foreground_chat: 'chat-1', foreground_title: TITLE }),
    })
    render(App)

    await fireEvent.click(await screen.findByRole('button', { name: 'Toggle sidebar' }))
    expect(screen.getByRole('button', { name: 'Close sidebar' })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(screen.getByRole('button', { name: 'Cerrar la barra lateral' })).toBeTruthy()
    // D7: the title came from Rust, so both languages draw it as it arrived.
    expect(screen.getByRole('heading', { name: TITLE })).toBeTruthy()

    setLanguage('en')
    Object.defineProperty(window, 'innerWidth', { value: 1024, configurable: true })
  })
})
