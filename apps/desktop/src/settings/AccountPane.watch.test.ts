// The Account pane's live sync (rpc-settings-sync T3b-2, S2): while the pane is
// mounted, the native watcher's `settings-files-changed` notice makes it ask for
// the SAME dialog load the focus listener and "Check again" ask for
// (settings-snapshot) - and only when the notice names the shell's auth store,
// never the profiles files.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render } from '@testing-library/svelte'
import type { AuthView_Serialize as AuthView } from '../contract/bindings'
import { listenerCount, pushEvent, resetTauriMock, setHandler } from '../test-support/tauri-mock'
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

/** One ask for the dialog's load, as the pane sends it. */
function ask(): () => void {
  return vi.fn()
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
  it('asks for a load when the notice names the auth store', async () => {
    const again = ask()
    render(AccountPane, { props: { auth: auth(), onRefresh: again } })
    await vi.waitFor(() => expect(listenerCount(WATCHED)).toBe(1))
    // Appearing read nothing: the dialog's one load is what filled this pane in.
    expect(again).not.toHaveBeenCalled()

    notice(false, true)

    await vi.waitFor(() => expect(again).toHaveBeenCalledTimes(1))
  })

  it('ignores a notice that names only the profiles files', async () => {
    const again = ask()
    render(AccountPane, { props: { auth: auth(), onRefresh: again } })
    await vi.waitFor(() => expect(listenerCount(WATCHED)).toBe(1))

    notice(true, false)
    await settle()

    expect(again).not.toHaveBeenCalled()
  })

  it('asks nothing after the pane goes away', async () => {
    const again = ask()
    const { unmount } = render(AccountPane, { props: { auth: auth(), onRefresh: again } })
    await vi.waitFor(() => expect(listenerCount(WATCHED)).toBe(1))

    unmount()
    notice(false, true)
    await settle()

    // No check runs with Settings' pane gone (rpc-settings-sync T3b-2).
    expect(again).not.toHaveBeenCalled()
  })

  it('stops listening when the pane goes away', async () => {
    const { unmount } = render(AccountPane, { props: { auth: auth(), onRefresh: ask() } })
    await vi.waitFor(() => expect(listenerCount(WATCHED)).toBe(1))

    unmount()
    expect(listenerCount(WATCHED)).toBe(0)
  })
})
