// The Profiles pane's live sync (rpc-settings-sync T3b-2, S2): while the pane is
// mounted, the native watcher's `settings-files-changed` notice makes it read the
// file again through the SAME `RefreshProfiles` intent the focus listener and the
// buttons send - one read at a time, never two stacked, and nothing at all once
// the pane is gone. One test per rule: the notice that names its files, the one
// that names only the auth store, the burst, the unmount and a notice that cannot
// be subscribed to.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render } from '@testing-library/svelte'
import type { ProfilesView_Serialize as ProfilesView } from '../contract/bindings'
import {
  deferred,
  failListen,
  handlerArgs,
  listenerCount,
  pushEvent,
  resetTauriMock,
  setHandler,
} from '../test-support/tauri-mock'
import ProfilesPane from './ProfilesPane.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The native watcher's event, as T3b-1 emits it. */
const WATCHED = 'settings-files-changed'

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => null)
})

function profiles(): ProfilesView {
  return { status: { type: 'Ready' }, active: null, mode: 'organic', profiles: [] }
}

/** How many times the pane asked for a fresh read of the profiles files. */
function reads(): number {
  return handlerArgs('dispatch_intent').filter(
    (args) => (args.intent as { type: string }).type === 'RefreshProfiles',
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

describe('the Profiles pane and the native watcher', () => {
  it('reads again when the notice names the profiles files', async () => {
    render(ProfilesPane, { props: { profiles: profiles() } })
    await vi.waitFor(() => expect(reads()).toBe(1))
    await vi.waitFor(() => expect(listenerCount(WATCHED)).toBe(1))

    notice(true, false)

    await vi.waitFor(() => expect(reads()).toBe(2))
  })

  it('ignores a notice that names only the auth store', async () => {
    render(ProfilesPane, { props: { profiles: profiles() } })
    await vi.waitFor(() => expect(listenerCount(WATCHED)).toBe(1))
    expect(reads()).toBe(1)

    notice(false, true)
    await settle()

    expect(reads()).toBe(1)
  })

  it('does not stack reads: a burst of notices asks for one more read', async () => {
    const first = deferred<void>()
    let calls = 0
    setHandler('dispatch_intent', () => (calls++ === 0 ? first.promise : null))
    render(ProfilesPane, { props: { profiles: profiles() } })
    await vi.waitFor(() => expect(reads()).toBe(1))
    await vi.waitFor(() => expect(listenerCount(WATCHED)).toBe(1))

    // Three notices while the first read is still in flight start nothing new.
    notice(true, false)
    notice(true, false)
    notice(true, false)
    await settle()
    expect(reads()).toBe(1)

    // They do not vanish either: once the first read answers, one more runs.
    first.resolve()
    await vi.waitFor(() => expect(reads()).toBe(2))
    await settle()
    expect(reads()).toBe(2)
  })

  it('starts nothing after the pane goes away: a queued read is dropped', async () => {
    const first = deferred<void>()
    let calls = 0
    setHandler('dispatch_intent', () => (calls++ === 0 ? first.promise : null))
    const { unmount } = render(ProfilesPane, { props: { profiles: profiles() } })
    await vi.waitFor(() => expect(reads()).toBe(1))
    await vi.waitFor(() => expect(listenerCount(WATCHED)).toBe(1))

    // A notice while the first read is still in flight asks for a follow-up...
    notice(true, false)
    await settle()
    expect(reads()).toBe(1)

    // ...and the pane goes away before that first read answers.
    unmount()
    first.resolve()
    await settle()

    // The in-flight read may finish, but nothing starts after the teardown.
    expect(reads()).toBe(1)
  })

  it('stops listening when the pane goes away, and reads nothing after', async () => {
    const { unmount } = render(ProfilesPane, { props: { profiles: profiles() } })
    await vi.waitFor(() => expect(listenerCount(WATCHED)).toBe(1))
    const before = reads()

    unmount()
    expect(listenerCount(WATCHED)).toBe(0)

    notice(true, false)
    await settle()
    expect(reads()).toBe(before)
  })

  it('keeps working when the notice cannot be subscribed to', async () => {
    failListen(WATCHED)
    render(ProfilesPane, { props: { profiles: profiles() } })
    await vi.waitFor(() => expect(reads()).toBe(1))
    await settle()
    expect(listenerCount(WATCHED)).toBe(0)

    // The focus read is the fallback, and it is still there.
    window.dispatchEvent(new Event('focus'))
    await vi.waitFor(() => expect(reads()).toBe(2))
  })
})
