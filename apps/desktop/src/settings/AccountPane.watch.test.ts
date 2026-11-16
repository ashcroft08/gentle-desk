// The Account pane's live sync (rpc-settings-sync T3b-2, S2): while the pane is
// mounted, the native watcher's `settings-files-changed` notice makes it run the
// SAME `RefreshAuth` check the focus listener and "Check again" send - and only
// when the notice names the shell's auth store, never the profiles files.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render } from '@testing-library/svelte'
import type { AuthView_Serialize as AuthView } from '../contract/bindings'
import {
  deferred,
  handlerArgs,
  listenerCount,
  pushEvent,
  resetTauriMock,
  setHandler,
} from '../test-support/tauri-mock'
import AccountPane from './AccountPane.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The native watcher's event, as T3b-1 emits it. */
const WATCHED = 'settings-files-changed'

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => null)
})

function auth(): AuthView {
  return { status: { type: 'Ready' }, providers: [] }
}

/** How many times the pane asked for a check of the shell's auth store. */
function checks(): number {
  return handlerArgs('dispatch_intent').filter(
    (args) => (args.intent as { type: string }).type === 'RefreshAuth',
  ).length
}

/** One notice of the native watcher: which group of settings files changed. */
function notice(profiles: boolean, account: boolean): void {
  pushEvent(WATCHED, { profiles, account })
}

/** Lets every microtask and timer the pane scheduled settle. */
async function settle(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, 0))
}

describe('the Account pane and the native watcher', () => {
  it('checks again when the notice names the auth store', async () => {
    render(AccountPane, { props: { auth: auth() } })
    await vi.waitFor(() => expect(checks()).toBe(1))
    await vi.waitFor(() => expect(listenerCount(WATCHED)).toBe(1))

    notice(false, true)

    await vi.waitFor(() => expect(checks()).toBe(2))
  })

  it('ignores a notice that names only the profiles files', async () => {
    render(AccountPane, { props: { auth: auth() } })
    await vi.waitFor(() => expect(listenerCount(WATCHED)).toBe(1))
    expect(checks()).toBe(1)

    notice(true, false)
    await settle()

    expect(checks()).toBe(1)
  })

  it('runs nothing after the pane goes away: a queued check is dropped', async () => {
    const first = deferred<void>()
    let calls = 0
    setHandler('dispatch_intent', () => (calls++ === 0 ? first.promise : null))
    const { unmount } = render(AccountPane, { props: { auth: auth() } })
    await vi.waitFor(() => expect(checks()).toBe(1))
    await vi.waitFor(() => expect(listenerCount(WATCHED)).toBe(1))

    // A notice while the first check is still in flight asks for a follow-up...
    notice(false, true)
    await settle()
    expect(checks()).toBe(1)

    // ...and the pane goes away before that first check answers.
    unmount()
    first.resolve()
    await settle()

    // No subprocess starts after Settings' pane is gone.
    expect(checks()).toBe(1)
  })

  it('stops listening when the pane goes away', async () => {
    const { unmount } = render(AccountPane, { props: { auth: auth() } })
    await vi.waitFor(() => expect(listenerCount(WATCHED)).toBe(1))

    unmount()
    expect(listenerCount(WATCHED)).toBe(0)
  })
})
