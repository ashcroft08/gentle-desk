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

import { events } from '../contract/bindings'

/** Which group of pi's Settings files a pane reads. */
export type SettingsFilesGroup = 'profiles' | 'account'

/**
 * Wraps a pane's refresh so two reads never stack: a refresh asked for while one
 * is in flight does not start a second call, it is remembered and runs ONCE when
 * the in-flight read answers. A burst of notices therefore costs one extra read,
 * not one per notice - which matters for the Account pane, whose read is a
 * subprocess.
 */
export function coalesced(read: () => Promise<void>): () => Promise<void> {
  let inFlight = false
  let asked = false
  return async function run(): Promise<void> {
    if (inFlight) {
      asked = true
      return
    }
    inFlight = true
    try {
      do {
        asked = false
        await read()
      } while (asked)
    } finally {
      inFlight = false
    }
  }
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
