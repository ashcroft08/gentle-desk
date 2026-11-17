// When the Profile tab reads (profiles-editor T2c, L13): opening the tab asks for
// the foreground chat's view ONCE, the window taking the focus back while the tab
// is open asks again, and the unavailable state's Try again does too. The reads are
// coalesced like the Settings panes' (one in flight, one queued) and closing the
// tab starts nothing after it (the `files-changed.ts` precedent). A chat with no
// foreground id asks for nothing: there is no chat to ask about.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte'
import { tick } from 'svelte'
import type { ChatProfileView_Serialize as ChatProfile } from '../contract/bindings'
import { setLanguage } from '../i18n/language.svelte'
import { chatProfile } from '../test-support/chat-profile'
import {
  type Deferred,
  deferred,
  handlerArgs,
  resetTauriMock,
  setHandler,
} from '../test-support/tauri-mock'
import ProfilePanel from './ProfilePanel.svelte'
import { profileCopy } from './copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

const copy = profileCopy('en')

/** Every read the tab asked for, as the intent carries it. */
function reads(): unknown[] {
  return handlerArgs('dispatch_intent')
    .map((args) => args.intent)
    .filter((intent) => (intent as { type: string }).type === 'RefreshChatProfile')
}

/** The chat ids those reads named. */
function chatIds(): unknown[] {
  return reads().map((intent) => (intent as { data: { chat_id: string } }).data.chat_id)
}

afterEach(cleanup)
beforeEach(() => {
  resetTauriMock()
  // The language is module state: every file in this suite starts in English.
  setLanguage('en')
})

describe('opening the tab', () => {
  it('reads the foreground chat once', async () => {
    setHandler('dispatch_intent', () => null)
    render(ProfilePanel, { props: { chatId: 'foreground_chat', view: null, active: true } })

    await waitFor(() => expect(reads()).toHaveLength(1))
    expect(chatIds()).toEqual(['foreground_chat'])
  })

  it('reads nothing while it is not the open tab', async () => {
    setHandler('dispatch_intent', () => null)
    render(ProfilePanel, { props: { chatId: 'foreground_chat', view: null, active: false } })

    await tick()
    expect(reads()).toHaveLength(0)
  })

  it('reads nothing for a chat with no foreground id', async () => {
    setHandler('dispatch_intent', () => null)
    render(ProfilePanel, { props: { chatId: null, view: null, active: true } })

    await tick()
    expect(reads()).toHaveLength(0)
    // And nothing is claimed about a chat that is not there: it is still reading.
    expect(screen.getByText(copy.reading)).toBeTruthy()
  })

  it('reads again when the tab is opened a second time', async () => {
    setHandler('dispatch_intent', () => null)
    const drawn = render(ProfilePanel, {
      props: { chatId: 'foreground_chat', view: null, active: false },
    })
    expect(reads()).toHaveLength(0)

    await drawn.rerender({ chatId: 'foreground_chat', view: null, active: true })

    await waitFor(() => expect(reads()).toHaveLength(1))
  })
})

describe('the window taking the focus back', () => {
  it('reads once more while the tab is open', async () => {
    setHandler('dispatch_intent', () => null)
    render(ProfilePanel, { props: { chatId: 'foreground_chat', view: chatProfile(), active: true } })
    await waitFor(() => expect(reads()).toHaveLength(1))

    window.dispatchEvent(new Event('focus'))

    await waitFor(() => expect(reads()).toHaveLength(2))
  })

  it('reads nothing once the tab is closed', async () => {
    setHandler('dispatch_intent', () => null)
    const drawn = render(ProfilePanel, {
      props: { chatId: 'foreground_chat', view: chatProfile(), active: true },
    })
    await waitFor(() => expect(reads()).toHaveLength(1))
    await drawn.rerender({ chatId: 'foreground_chat', view: chatProfile(), active: false })

    window.dispatchEvent(new Event('focus'))

    await tick()
    expect(reads()).toHaveLength(1)
  })
})

describe('the reads of one open tab', () => {
  /** One dispatch, held open until the test answers it. */
  function serveSlowDispatch(): Deferred<unknown>[] {
    const pending: Deferred<unknown>[] = []
    setHandler('dispatch_intent', () => {
      const answer = deferred<unknown>()
      pending.push(answer)
      return answer.promise
    })
    return pending
  }

  it('never runs two at once: a second ask waits and runs once', async () => {
    const pending = serveSlowDispatch()
    render(ProfilePanel, { props: { chatId: 'foreground_chat', view: chatProfile(), active: true } })
    await waitFor(() => expect(reads()).toHaveLength(1))

    // Two asks while the first read is still in flight.
    window.dispatchEvent(new Event('focus'))
    window.dispatchEvent(new Event('focus'))
    await tick()
    expect(reads()).toHaveLength(1)

    pending[0].resolve(null)

    await waitFor(() => expect(reads()).toHaveLength(2))
    pending[1].resolve(null)
    await tick()
    // The burst cost one follow-up read, not one per ask.
    expect(reads()).toHaveLength(2)
  })

  it('starts nothing after the tab closes, not even a queued read', async () => {
    const pending = serveSlowDispatch()
    const drawn = render(ProfilePanel, {
      props: { chatId: 'foreground_chat', view: chatProfile(), active: true },
    })
    await waitFor(() => expect(reads()).toHaveLength(1))

    // A read is queued behind the one in flight, and then the tab closes.
    window.dispatchEvent(new Event('focus'))
    await tick()
    await drawn.rerender({ chatId: 'foreground_chat', view: chatProfile(), active: false })

    pending[0].resolve(null)
    await tick()
    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(reads()).toHaveLength(1)
  })
})

describe('the unavailable state', () => {
  it('says the read answered nothing, and Try again reads once more', async () => {
    setHandler('dispatch_intent', () => null)
    render(ProfilePanel, { props: { chatId: 'foreground_chat', view: null, active: true } })

    // `chat_profile` stayed null after the read answered: the tab says so.
    const again = await screen.findByRole('button', { name: copy.tryAgain })
    expect(screen.getByText(copy.unavailable)).toBeTruthy()
    expect(screen.getByText(copy.unavailableWhy)).toBeTruthy()

    await fireEvent.click(again)

    await waitFor(() => expect(reads()).toHaveLength(2))
  })

  it('draws the same state when the read answered a view no store filled', async () => {
    setHandler('dispatch_intent', () => null)
    const empty: ChatProfile = chatProfile({
      profiles: [],
      default_profile: null,
      pins: null,
      stores: { profiles: 'unavailable', fallbacks: 'unavailable', pins: 'unavailable' },
    })
    render(ProfilePanel, { props: { chatId: 'foreground_chat', view: empty, active: true } })

    expect(screen.getByText(copy.unavailable)).toBeTruthy()
    expect(screen.getByRole('button', { name: copy.tryAgain })).toBeTruthy()
  })
})
