// The native watcher's notice, as the Settings panes consume it
// (rpc-settings-sync T3b-2, S2; the engine is T3b-1's settings/watch.rs).
//
// The watcher only SAYS that pi's config files changed on disk: it runs no
// subprocess and no refresh, so nothing re-reads while Settings is closed. The
// panes own the files they read, so the notice names GROUPS (`profiles`,
// `account`) and never a path, and each pane decides whether the change is its
// own. A pane listens while it is mounted and hands the payload to its own
// refresh - the very same function its window-focus listener and its buttons
// call, so there is one read path and not two.
//
// The `profiles` group has no listener since profiles-editor T2d removed
// Settings > Profiles: it still names the files the native watcher sees, and the
// coalesced read below is shared with the Profile tab, which reads through its
// chat's child instead.

import { events } from '../contract/bindings'

/** Which group of pi's Settings files a pane reads. */
export type SettingsFilesGroup = 'profiles' | 'account'

/**
 * A pane's read, plus the way to stop it: `cancel()` is the pane's teardown,
 * after which the read starts nothing - not the queued follow-up and not a late
 * event. A read already in flight is left to finish (an IPC call is not
 * aborted), it simply starts nothing after itself.
 */
export type CoalescedRead = (() => Promise<void>) & { cancel: () => void }

/**
 * Wraps a pane's refresh so two reads never stack: a refresh asked for while one
 * is in flight does not start a second call, it is remembered and runs ONCE when
 * the in-flight read answers. A burst of notices therefore costs one extra read,
 * not one per notice - which matters for the Account pane, whose read is a
 * subprocess. `cancel()` ends that: nothing starts after the pane's teardown,
 * so no refresh runs with Settings closed.
 */
export function coalesced(read: () => Promise<void>): CoalescedRead {
  let inFlight = false
  let asked = false
  let stopped = false

  async function run(): Promise<void> {
    if (stopped) return
    if (inFlight) {
      asked = true
      return
    }
    inFlight = true
    try {
      do {
        asked = false
        await read()
      } while (asked && !stopped)
    } finally {
      inFlight = false
    }
  }

  return Object.assign(run, {
    cancel(): void {
      stopped = true
      asked = false
    },
  })
}

/**
 * Subscribes the caller's refresh to the native watcher while it is mounted, and
 * returns the function that unsubscribes it: a dialog that opens and closes must
 * not leave a listener behind. A subscription that fails - a build without the
 * event - is not fatal: the pane's window-focus read is still the fallback.
 */
export function followSettingsFiles(
  group: SettingsFilesGroup,
  refresh: () => Promise<void>,
): () => void {
  let stopped = false
  let stop: (() => void) | null = null

  void (async () => {
    try {
      const unlisten = await events.settingsFilesChanged.listen((event) => {
        if (stopped || !event.payload[group]) return
        void refresh()
      })
      if (stopped) {
        unlisten()
        return
      }
      stop = unlisten
    } catch {
      // No notice on this run: the focus read below/around still re-reads.
    }
  })()

  return () => {
    stopped = true
    stop?.()
    stop = null
  }
}
