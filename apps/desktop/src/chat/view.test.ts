// The open chat's chrome, one test per slice it picks: the header's title, the
// foreground chat's mark (which is what the composer's state is decided from),
// the notices, and no chrome at all while no chat is open.

import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  activity,
  plainNotice,
  pushEvent,
  resetTauriMock,
  restartNotice,
  setHandler,
  viewState,
} from '../test-support/tauri-mock'
import { chatProfile } from '../test-support/chat-profile'
import { helperRow, helpersPanel } from '../test-support/helpers'
import { helperThread as openThread } from '../test-support/helper-thread'
import { chatOf, createChatView } from './view'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

beforeEach(() => {
  resetTauriMock()
})

describe('chatOf', () => {
  it('carries the title and the mark of the foreground chat', () => {
    const state = viewState({
      foreground_chat: 'chat-1',
      foreground_title: 'Tauri skeleton: window and bridge',
      activity: [activity('chat-1', 'Working'), activity('chat-2', 'Opening')],
    })

    expect(chatOf(state)).toEqual({
      chat: 'chat-1',
      title: 'Tauri skeleton: window and bridge',
      mark: 'Working',
      notices: [],
      helpers: null,
      selectedHelper: null,
      helperThread: null,
      dialog: null,
      images: [],
      imagesNotice: null,
      // The mock's default chip: the child's truth before it answers.
      modelChip: {
        status: { type: 'Unknown' },
        models: [],
        current: null,
        thinking_level: null,
        thinking_levels: [],
      },
      // How full its context is: null until the chat's own child answers.
      contextUsage: { tokens: null, percent: null, window: null },
      // The folder it runs in: absent for a chat whose session is not listed.
      folder: null,
      // And the branch on it: absent for the same reason, and for a folder no
      // repository holds (R3).
      branch: null,
      // Nothing read this chat's profile yet (the Profile tab reads it).
      chatProfile: null,
      // And nothing answered for its session mode yet (T3c): no control.
      sessionMode: null,
      fatal: null,
      whatsNew: null,
      shell: { pi_path: null, choice: null, ask: false, launcher_found: true, install: { type: 'Idle' }, restart_needed: false },
    })
  })

  it("carries the foreground chat's own context reading, nulls included", () => {
    const state = viewState({
      foreground_chat: 'chat-1',
      context_usage: { tokens: 70_000, percent: 35, window: 200_000 },
    })

    expect(chatOf(state).contextUsage).toEqual({ tokens: 70_000, percent: 35, window: 200_000 })
  })

  it("carries the profile view the shell holds for the foreground chat", () => {
    const view = chatProfile({ chat_id: 'chat-1' })
    const state = viewState({ foreground_chat: 'chat-1', chat_profile: view })

    expect(chatOf(state).chatProfile).toBe(view)
    // The Profile tab is a view OF a chat: with none open there is none either.
    expect(chatOf(viewState({ chat_profile: view })).chatProfile).toBeNull()
  })

  it('carries the session mode the foreground chat answered, refusal and attempt', () => {
    const sessionMode = { mode: 'super-yolo', refusal: null, attempt: 2 }
    const state = viewState({ foreground_chat: 'chat-1', session_mode: sessionMode })

    expect(chatOf(state).sessionMode).toEqual(sessionMode)
    // The mode is part of a chat's chrome: with no chat open there is none.
    expect(chatOf(viewState({ session_mode: sessionMode })).sessionMode).toBeNull()
  })

  it("carries the folder the foreground chat runs in, name and whole path", () => {
    const folder = { name: 'gentle-desk', path: 'C:\\Users\\josep\\dev\\gentle-desk' }
    const state = viewState({ foreground_chat: 'chat-1', foreground_folder: folder })

    expect(chatOf(state).folder).toEqual(folder)
    // The chip is part of a chat's chrome: with none open there is no folder.
    expect(chatOf(viewState({ foreground_folder: folder })).folder).toBeNull()
  })

  it("carries the branch the foreground chat's folder is on, name and where", () => {
    const branch = { name: 'feat/0.7.0-transcript', path: 'C:\\Users\\josep\\dev\\gentle-desk' }
    const state = viewState({ foreground_chat: 'chat-1', foreground_branch: branch })

    expect(chatOf(state).branch).toEqual(branch)
    // The chip belongs to a chat's chrome too: with none open there is no branch.
    expect(chatOf(viewState({ foreground_branch: branch })).branch).toBeNull()
  })

  it('carries no branch for a folder git had nothing to say about (R3)', () => {
    // Rust answers `null` for a folder outside any repository, and the composer
    // reads exactly that: absent, never a placeholder to fill the row.
    const state = viewState({
      foreground_chat: 'chat-1',
      foreground_folder: { name: 'gentle-desk', path: 'C:\\dev\\gentle-desk' },
      foreground_branch: null,
    })

    expect(chatOf(state).branch).toBeNull()
  })

  it('leaves an idle chat unmarked, so its composer is ready', () => {
    const state = viewState({ foreground_chat: 'chat-1', foreground_title: 'Idle one' })

    expect(chatOf(state).mark).toBeNull()
  })

  it('carries the helpers panel, the selection and the open thread', () => {
    const helpers = helpersPanel([helperRow('h1')], { running: 1 })
    const helperThread = openThread({ id: 'h1' })
    const state = viewState({
      foreground_chat: 'chat-1',
      helpers,
      selected_helper: 'h1',
      helper_thread: helperThread,
    })

    expect(chatOf(state).helpers).toBe(helpers)
    expect(chatOf(state).selectedHelper).toBe('h1')
    expect(chatOf(state).helperThread).toBe(helperThread)
  })

  it('carries every pushed notice, unchanged and in order', () => {
    const first = restartNotice(1, 'The chat process exited.', 'chat-1')
    const second = plainNotice(2, 'Could not read 3 chat files.')
    const state = viewState({ foreground_chat: 'chat-1', notices: [first, second] })

    expect(chatOf(state).notices).toEqual([first, second])
  })

  it('draws no chrome while no chat is open', () => {
    const state = viewState({
      foreground_title: 'a title with no chat',
      notices: [restartNotice(1, 'The chat process exited.', 'chat-1')],
    })

    expect(chatOf(state)).toEqual({
      chat: null,
      title: null,
      mark: null,
      notices: [],
      helpers: null,
      selectedHelper: null,
      helperThread: null,
      dialog: null,
      images: [],
      imagesNotice: null,
      modelChip: null,
      contextUsage: null,
      folder: null,
      branch: null,
      chatProfile: null,
      sessionMode: null,
      fatal: null,
      whatsNew: null,
      shell: { pi_path: null, choice: null, ask: false, launcher_found: true, install: { type: 'Idle' }, restart_needed: false },
    })
    expect(chatOf(null).chat).toBeNull()
  })
})

describe('createChatView', () => {
  it('follows a push that opens a chat and one that closes it', async () => {
    setHandler('get_view_state', () =>
      viewState({ foreground_chat: 'chat-1', foreground_title: 'First' }),
    )
    const view = createChatView()
    await view.ready

    expect(view.get()).toMatchObject({ chat: 'chat-1', title: 'First' })

    pushEvent('state-changed', viewState({ foreground_chat: null }))
    await vi.waitFor(() => expect(view.get().chat).toBeNull())
    view.dispose()
  })

  it('follows a push whose only change is how the install is going', async () => {
    const shell = {
      pi_path: null,
      choice: null,
      ask: false,
      launcher_found: false,
      restart_needed: false,
    }
    setHandler('get_view_state', () =>
      viewState({ sessions_revision: 1, shell: { ...shell, install: { type: 'Idle' }, restart_needed: false } }),
    )
    const view = createChatView()
    await view.ready

    pushEvent(
      'state-changed',
      viewState({ sessions_revision: 2, shell: { ...shell, install: { type: 'Running' } } }),
    )
    await vi.waitFor(() => expect(view.get().shell.install).toEqual({ type: 'Running' }))

    pushEvent(
      'state-changed',
      viewState({ sessions_revision: 3, shell: { ...shell, launcher_found: true, install: { type: 'Done' } } }),
    )
    await vi.waitFor(() => expect(view.get().shell.launcher_found).toBe(true))
    expect(view.get().shell.install).toEqual({ type: 'Done' })
    view.dispose()
  })

  it('follows a push whose only change is the answer to the welcome question', async () => {
    const pi = 'C:/Users/someone/.pi/agent'
    setHandler('get_view_state', () =>
      viewState({ sessions_revision: 1, shell: { pi_path: pi, choice: null, ask: true, launcher_found: true, install: { type: 'Idle' }, restart_needed: false } }),
    )
    const view = createChatView()
    await view.ready
    expect(view.get().shell.ask).toBe(true)

    // Nothing else moved: no chat, no notice, no helper. Only the choice.
    pushEvent(
      'state-changed',
      viewState({ sessions_revision: 2, shell: { pi_path: pi, choice: 'link', ask: false, launcher_found: true, install: { type: 'Idle' }, restart_needed: false } }),
    )

    await vi.waitFor(() => expect(view.get().shell.ask).toBe(false))
    expect(view.get().shell.choice).toBe('link')
    view.dispose()
  })
})
