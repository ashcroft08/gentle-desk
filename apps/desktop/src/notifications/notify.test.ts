// The alerting path (notifications S1, S2, S3, S6), end to end over the detector:
// what a push says is announced ONLY while the window is not in front and the
// setting is on, one tone and one toast per alert, never twice for the same kind
// inside five seconds, and never at all for the first state after launch.
//
// The tone and the toast are mocked here (`./tone`, `./toast`): this file is
// about WHEN they are asked for and with what words, while their own files cover
// how they do it. The window's focus is the document's, exactly as the store
// reads it, so the test says "in front" the way the machine does.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { LanguageView, ViewState_Serialize as ViewState } from '../contract/bindings'
import {
  activity,
  confirmDialog,
  plainNotice,
  pushEvent,
  resetTauriMock,
  viewState,
} from '../test-support/tauri-mock'

const { playTone, sendToast } = vi.hoisted(() => ({
  playTone: vi.fn(async () => {}),
  sendToast: vi.fn<(title: string, body: string) => Promise<'sent' | 'denied'>>(
    async () => 'sent',
  ),
}))

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))
vi.mock('./tone', () => ({ playTone }))
vi.mock('./toast', () => ({ sendToast }))

import { notifyCopy } from './copy'
import { RATE_LIMIT_MS, startNotifications } from './notify'

/** Lets the wiring's own promise chain land before a test reads the mocks. */
async function settle(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, 0))
}

/** What the document says about being in front, as the store reads it. */
function inFront(focused: boolean): void {
  vi.spyOn(document, 'hasFocus').mockReturnValue(focused)
  Object.defineProperty(document, 'hidden', { configurable: true, get: () => false })
}

/** Starts the wiring the way the shell does, once the window has said its state. */
async function start(
  options: { language?: LanguageView; view?: () => ViewState | null } = {},
): Promise<() => void> {
  const stop = startNotifications({
    language: options.language ?? 'en',
    ...(options.view === undefined ? {} : { view: options.view }),
  })
  await settle()
  return stop
}

/** One pushed state, as `state-changed` carries it. */
function push(state: ViewState): void {
  pushEvent('state-changed', state)
}

/** A chat working on a reply, in the foreground and named by the view. */
function working(): ViewState {
  return viewState({
    foreground_chat: 'chat-1',
    foreground_title: 'Fix the build',
    activity: [activity('chat-1', 'Working')],
  })
}

/** The same chat with the turn finished: what S3a is about. */
function replied(): ViewState {
  return viewState({ foreground_chat: 'chat-1', foreground_title: 'Fix the build' })
}

/** Every alert one push can carry: a reply, an ask and a failure at once. */
function asking(): ViewState {
  return viewState({
    foreground_chat: 'chat-1',
    foreground_title: 'Fix the build',
    activity: [activity('chat-2', 'NeedsAttention')],
    dialog: confirmDialog('dialog-1'),
    notices: [plainNotice(7, 'Disk is full', 'Error')],
  })
}

/** Only the ask: a dialog appeared over the chat the view names. */
function waiting(): ViewState {
  return viewState({
    foreground_chat: 'chat-1',
    foreground_title: 'Fix the build',
    dialog: confirmDialog('dialog-1'),
  })
}

/** Stops the wiring after every case, so one test's listeners never leak. */
let stop: (() => void) | null = null

beforeEach(() => {
  resetTauriMock()
  playTone.mockClear()
  sendToast.mockClear()
  sendToast.mockResolvedValue('sent')
  inFront(false)
})

afterEach(() => {
  stop?.()
  stop = null
  vi.useRealTimers()
  vi.restoreAllMocks()
  Object.defineProperty(document, 'hidden', { configurable: true, get: () => false })
})

describe('an alert while the window is in front', () => {
  it('says nothing, whatever the push says (S1)', async () => {
    inFront(true)
    stop = await start()

    push(working())
    push(replied())
    await settle()

    expect(playTone).not.toHaveBeenCalled()
    expect(sendToast).not.toHaveBeenCalled()
  })
})

describe('an alert while the window is not in front', () => {
  it('plays one tone and sends one toast per kind (S2, S3)', async () => {
    stop = await start()

    push(working())
    push(asking())
    await settle()

    expect(playTone).toHaveBeenCalledTimes(3)
    expect(sendToast.mock.calls).toEqual([
      // The reply and the ask are both about the chat the view names, so they
      // say its own title; the failure says the notice's own words.
      ['Fix the build', notifyCopy('en').replyBody],
      ['Fix the build', notifyCopy('en').needsYouBody],
      [notifyCopy('en').failureTitle, 'Disk is full'],
    ])
  })

  it('falls back to a generic line when the view cannot name the chat', async () => {
    stop = await start()

    push(working())
    // The chat that finished is not the one the view names any more.
    push(viewState({ foreground_chat: 'chat-3', foreground_title: 'Another chat' }))
    await settle()

    // And a chat waiting in the background is named by nobody, so the kind's own
    // line stands in for its title.
    push(
      viewState({
        foreground_chat: 'chat-3',
        foreground_title: 'Another chat',
        activity: [activity('chat-2', 'NeedsAttention')],
      }),
    )
    await settle()

    expect(sendToast.mock.calls).toEqual([
      [notifyCopy('en').replyTitle, notifyCopy('en').replyBody],
      [notifyCopy('en').needsYouTitle, notifyCopy('en').needsYouBody],
    ])
  })

  it('writes both the tone and the toast’s words in the language the app speaks', async () => {
    stop = await start({ language: 'es' })

    push(working())
    push(replied())
    await settle()

    expect(playTone).toHaveBeenCalledTimes(1)
    expect(sendToast.mock.calls).toEqual([
      ['Fix the build', notifyCopy('es').replyBody],
    ])
    expect(notifyCopy('es').replyBody).not.toBe(notifyCopy('en').replyBody)
  })
})

describe('an alert with the setting off', () => {
  it('says nothing, whatever the push says (S4)', async () => {
    stop = await start()

    push({ ...working(), notifications: 'off' })
    push({ ...asking(), notifications: 'off' })
    await settle()

    expect(playTone).not.toHaveBeenCalled()
    expect(sendToast).not.toHaveBeenCalled()
  })
})

describe('the first state after launch', () => {
  it('says nothing, and the next transition still does (S6)', async () => {
    stop = await start()

    // A state that is already holding a finished turn, an ask and a failure: the
    // alerts of things that happened before the window was watching.
    push(asking())
    await settle()

    expect(playTone).not.toHaveBeenCalled()
    expect(sendToast).not.toHaveBeenCalled()

    push(asking())
    await settle()

    // The same state again announces nothing either: only transitions do.
    expect(playTone).not.toHaveBeenCalled()

    push(working())
    push(replied())
    await settle()

    expect(playTone).toHaveBeenCalledTimes(1)
    expect(sendToast).toHaveBeenCalledTimes(1)
  })
})

describe('a burst of alerts', () => {
  it('sounds one kind once in five seconds and lets the others through (S6)', async () => {
    vi.useFakeTimers({ toFake: ['Date'] })
    vi.setSystemTime(0)
    stop = await start()

    push(working())
    push(replied())
    await settle()
    expect(playTone).toHaveBeenCalledTimes(1)

    // Three more replies, two seconds apart at the most: one sound, not a queue.
    for (const at of [1_000, 2_000, 4_000]) {
      vi.setSystemTime(at)
      push(working())
      push(replied())
      await settle()
    }
    expect(playTone).toHaveBeenCalledTimes(1)
    expect(sendToast.mock.calls).toHaveLength(1)

    // Another kind inside the same window is a different alert, and sounds.
    vi.setSystemTime(4_500)
    push(waiting())
    await settle()
    expect(playTone).toHaveBeenCalledTimes(2)

    // And the reply sounds again once its own window has passed.
    vi.setSystemTime(RATE_LIMIT_MS)
    push(working())
    push(replied())
    await settle()
    expect(playTone).toHaveBeenCalledTimes(3)
  })
})

describe('a toast Windows refused', () => {
  it('never throws out of the wiring', async () => {
    sendToast.mockResolvedValue('denied')
    stop = await start()

    push(working())
    push(replied())

    // The tone already played, and the refusal is the toast's own answer: the
    // wiring keeps going, and the next push announces what it says.
    await settle()
    expect(playTone).toHaveBeenCalledTimes(1)
    expect(sendToast).toHaveBeenCalledTimes(1)

    push(asking())
    await settle()
    expect(sendToast).toHaveBeenCalledTimes(3)
  })
})

describe('the state a caller hands over', () => {
  it('announces the state the getter holds, not the payload the push carries', async () => {
    // A caller that keeps its own newest state (a store) makes the push nothing
    // but the tick that says "look again".
    let held: ViewState | null = working()
    stop = await start({ view: () => held })

    push(viewState())
    await settle()
    expect(playTone).not.toHaveBeenCalled()

    held = replied()
    push(viewState())
    await settle()

    expect(playTone).toHaveBeenCalledTimes(1)
    expect(sendToast).toHaveBeenCalledTimes(1)
  })
})
