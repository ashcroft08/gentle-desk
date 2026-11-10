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
      dialog: null,
      fatal: null,
    })
  })

  it('leaves an idle chat unmarked, so its composer is ready', () => {
    const state = viewState({ foreground_chat: 'chat-1', foreground_title: 'Idle one' })

    expect(chatOf(state).mark).toBeNull()
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
      dialog: null,
      fatal: null,
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
})
