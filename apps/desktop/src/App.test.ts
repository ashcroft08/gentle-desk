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
import { welcomeView, welcomeWorkspace } from './test-support/welcome'
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

  it('draws the rows a session file painted, with no Copy, and then the chat’s own', async () => {
    // chat-open-latency T1/T2: a stored chat opens on the rows its file holds,
    // and the child's answer replaces them in the SAME slice - the flag travels
    // with the rows, so the thread never draws a frame without them.
    const FILE_ROW = { ...MESSAGE, id: 'prov-0', text: 'a pregunta' }
    const FILED = [
      { kind: 'Message' as const, data: FILE_ROW },
      {
        kind: 'Message' as const,
        data: { ...MESSAGE, id: 'prov-1', role: 'Assistant' as const, text: 'a resposta' },
      },
    ]
    const fixture = serveSidebar({
      view: viewState({
        foreground_chat: 'chat-1',
        messages: [FILE_ROW],
        items: FILED,
        transcript_provisional: true,
        // The chat is opening: that is exactly when the file's rows are drawn.
        activity: [activity('chat-1', 'Opening')],
      }),
    })
    render(App)

    expect(await screen.findByText('a resposta')).toBeTruthy()
    expect(screen.queryByRole('button', { name: 'Copy' })).toBeNull()
    expect(screen.getByText('Opening chat')).toBeTruthy()

    // The child's answer: the same turns under its own ids, and Copy is back.
    fixture.view = viewState({
      foreground_chat: 'chat-1',
      messages: [{ ...MESSAGE, id: 'msg-0', text: 'a pregunta' }, { ...MESSAGE, id: 'msg-1', role: 'Assistant', text: 'a resposta' }],
      items: [
        { kind: 'Message', data: { ...MESSAGE, id: 'msg-0', text: 'a pregunta' } },
        { kind: 'Message', data: { ...MESSAGE, id: 'msg-1', role: 'Assistant', text: 'a resposta' } },
      ],
      transcript_provisional: false,
    })
    fixture.push()

    expect(await screen.findByRole('button', { name: 'Copy' })).toBeTruthy()
    expect(screen.getByText('a resposta')).toBeTruthy()
    expect(screen.queryByText('Opening chat')).toBeNull()
  })

  it('draws the composer’s folder chip for the folder Rust pushed', async () => {
    const path = 'C:\\Users\\josep\\dev\\gentle-desk'
    serveSidebar({
      view: viewState({
        foreground_chat: 'chat-1',
        foreground_title: 'Gentle desk',
        messages: [MESSAGE],
        foreground_folder: { name: 'gentle-desk', path },
      }),
    })
    const { container } = render(App)

    const chip = (await waitFor(() => {
      const found = container.querySelector('.composer .row .folder')
      expect(found).toBeTruthy()
      return found as HTMLElement
    })) as HTMLElement

    // The last segment only, with the whole path in the hover (S2, R4).
    expect(chip.textContent?.trim()).toBe('gentle-desk')
    expect(chip.textContent).not.toContain('Users')
    expect(chip.getAttribute('title')).toBe(path)
  })

  it('draws the composer’s branch chip for the branch Rust pushed, after the folder', async () => {
    const path = 'C:\\Users\\josep\\dev\\gentle-desk'
    serveSidebar({
      view: viewState({
        foreground_chat: 'chat-1',
        foreground_title: 'Gentle desk',
        messages: [MESSAGE],
        foreground_folder: { name: 'gentle-desk', path },
        foreground_branch: { name: 'feat/0.7.0-transcript', path },
      }),
    })
    const { container } = render(App)

    const chip = (await waitFor(() => {
      const found = container.querySelector('.composer .row .branch')
      expect(found).toBeTruthy()
      return found as HTMLElement
    })) as HTMLElement

    // The branch name only, with the directory it was read from in the hover.
    expect(chip.textContent?.trim()).toBe('feat/0.7.0-transcript')
    expect(chip.getAttribute('title')).toBe(path)
    // It sits AFTER the folder, inside the group the one hairline opens.
    expect(chip.previousElementSibling?.className).toBe('folder')
  })

  it('draws no branch chip for a folder no repository holds, and no gap either', async () => {
    const path = 'C:\\Users\\josep\\dev\\gentle-desk'
    serveSidebar({
      view: viewState({
        foreground_chat: 'chat-1',
        foreground_title: 'Gentle desk',
        messages: [MESSAGE],
        foreground_folder: { name: 'gentle-desk', path },
        foreground_branch: null,
      }),
    })
    const { container } = render(App)

    const folder = (await waitFor(() => {
      const found = container.querySelector('.composer .row .folder')
      expect(found).toBeTruthy()
      return found as HTMLElement
    })) as HTMLElement

    // R3: the folder is drawn and the branch is simply not there - the row
    // moves on to its flexible space, with nothing standing in the gap.
    expect(container.querySelector('.composer .row .branch')).toBeNull()
    expect(folder.closest('.grp')!.nextElementSibling?.className).toBe('grow')
  })

  it('draws no folder chip for a chat whose folder Rust did not name', async () => {
    serveSidebar({
      view: viewState({ foreground_chat: 'chat-1', foreground_title: 'Fresh one', messages: [MESSAGE] }),
    })
    const { container } = render(App)

    expect(await screen.findByRole('region', { name: 'Chat thread' })).toBeTruthy()
    expect(container.querySelector('.composer .row .folder')).toBeNull()
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






  it('draws view B for a new chat, and its own composer keeps the draft when it lands', async () => {
    // welcome-screen T3 (S1, S3, R1): a NEW chat - the fresh id Rust builds for a
    // chat with no session file - is the centred view, and the box it draws is the
    // one composer the app owns, still there once the first row arrives.
    const path = 'C:\\Users\\josep\\dev\\gentle-desk'
    const fixture = serveSidebar({
      view: viewState({
        foreground_chat: 'new-1',
        foreground_folder: { name: 'gentle-desk', path },
      }),
    })
    render(App)

    expect(await screen.findByRole('heading', { level: 2, name: 'New session' })).toBeTruthy()
    expect(screen.getByText('in gentle-desk')).toBeTruthy()
    const box = screen.getByRole('textbox', { name: 'Message' }) as HTMLTextAreaElement
    await fireEvent.input(box, { target: { value: 'hola' } })

    fixture.view = viewState({
      foreground_chat: 'new-1',
      foreground_folder: { name: 'gentle-desk', path },
      messages: [MESSAGE],
      items: [{ kind: 'Message', data: MESSAGE }],
    })
    fixture.push()

    expect(await screen.findByText(MESSAGE.text)).toBeTruthy()
    expect(screen.queryByRole('heading', { level: 2, name: 'New session' })).toBeNull()
    expect(screen.getByRole('textbox', { name: 'Message' })).toBe(box)
    expect(box.value).toBe('hola')
  })

  it('draws view A where the placeholder was, and keeps the page\u2019s one h1', async () => {
    serveSidebar({
      view: viewState({
        foreground_chat: null,
        welcome: welcomeView({
          last: welcomeWorkspace({ path: 'C:\\dev\\curriculum', label: 'Curriculum' }),
        }),
      }),
    })
    render(App)

    // The pushed slice is the screen: its own h2 under the header's h1, one
    // action for the last workspace, and no placeholder hint left behind.
    expect(
      await screen.findByRole('heading', { level: 2, name: 'Pick up where you left off' }),
    ).toBeTruthy()
    expect(screen.getByRole('heading', { level: 1, name: 'Home' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'New session in Curriculum' })).toBeTruthy()
    expect(screen.queryByText('Choose a chat in the workspace tree, or start a new session.')).toBeNull()
  })

  it('names view A in the page\u2019s one h1, in either language, with no old words left', async () => {
    // R1: what is on screen is view A, so the header says where the reader is
    // (the home dictionary's own word) instead of that no chat is open.
    setLanguage('en')
    serveSidebar({
      view: viewState({
        foreground_chat: null,
        welcome: welcomeView({
          last: welcomeWorkspace({ path: 'C:\\dev\\curriculum', label: 'Curriculum' }),
        }),
      }),
    })
    render(App)

    expect(await screen.findByRole('heading', { level: 1, name: 'Home' })).toBeTruthy()
    expect(screen.getAllByRole('heading', { level: 1 })).toHaveLength(1)
    expect(screen.queryByText('No chat open')).toBeNull()
    expect(screen.queryByText('Choose a chat in the workspace tree, or start a new session.')).toBeNull()

    setLanguage('es')
    await tick()

    expect(screen.getByRole('heading', { level: 1, name: 'Inicio' })).toBeTruthy()
    expect(screen.getAllByRole('heading', { level: 1 })).toHaveLength(1)
    expect(screen.queryByText('Ningún chat abierto')).toBeNull()

    setLanguage('en')
  })

  it('keeps the old header over the welcome with no configured workspace', async () => {
    // The welcome's no-workspace face draws the rose and the old hint, but no
    // view A title of its own, so the header still says no chat is open.
    serveSidebar({
      view: viewState({ foreground_chat: null, welcome: welcomeView({ last: null }) }),
    })
    render(App)

    expect(await screen.findByText('No chat open')).toBeTruthy()
    expect(await screen.findByRole('img', { name: 'The gentle-shell rose' })).toBeTruthy()
    expect(screen.getByText('Choose a chat in the workspace tree, or start a new session.')).toBeTruthy()
    expect(screen.queryByRole('heading', { level: 1, name: 'Home' })).toBeNull()
  })

  it('keeps today’s placeholder while the push carries no welcome at all', async () => {
    // The previous behaviour of the no-chat branch: a state with no view A (the
    // slice Rust only fills while no chat is open) still says both old lines.
    serveSidebar({ view: viewState({ foreground_chat: null }) })
    render(App)

    expect(await screen.findByText('No chat open')).toBeTruthy()
    expect(screen.getByText('Choose a chat in the workspace tree, or start a new session.')).toBeTruthy()
    expect(screen.queryByText('Pick up where you left off')).toBeNull()
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
