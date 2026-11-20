// The Settings dialog's ONE read, as a small store (settings-snapshot): one
// `settings_snapshot` per open, the plain object every page draws from, and the
// signal that says a load the user asked for is on its way.
//
// It lives here rather than in the dialog for two reasons: the dialog already
// carries the rail, the search and the seven sections, and this is the one rule
// worth reading on its own - how often the machine is read, and what happens to
// the asks that arrive while a read is out.
//
// `coalesced` is the helper the Account pane used for its own check
// (rpc-settings-sync T3b-2), unchanged: a burst of asks - a watcher notice, a
// window focus and a click at the same moment - costs one extra read, never one
// per ask, and a stopped read starts nothing after itself, so no check runs with
// Settings closed.
import type { SettingsSnapshot_Serialize as SettingsSnapshot } from '../contract/bindings'
import { settingsSnapshot } from '../bridge'
import { coalesced, type CoalescedRead } from './files-changed'

/** What the dialog holds: the answer, whether a read is out, and the two ways to ask. */
export interface SettingsRead {
  /** The one read's answer, or `null` until it lands. */
  readonly snapshot: SettingsSnapshot | null
  /** Whether a read is on its way right now. */
  readonly busy: boolean
  /** Starts this open's one read; the answer is the function that stops it. */
  start: () => () => void
  /** Asks for one more read: what every section's "Check again" calls. */
  again: () => void
}

export function createSettingsRead(): SettingsRead {
  let snapshot = $state<SettingsSnapshot | null>(null)
  let busy = $state(false)
  // The read of this open, kept so every ask reaches the same queue.
  let load: CoalescedRead | null = null

  /**
   * One read, one object. A load that does not answer at all leaves the snapshot
   * as it was and the next ask tries again: each SECTION's own failure is inside
   * the object, never here.
   */
  async function read(): Promise<void> {
    busy = true
    try {
      const answered = await settingsSnapshot()
      if (answered !== null) snapshot = answered
    } finally {
      busy = false
    }
  }

  return {
    get snapshot(): SettingsSnapshot | null {
      return snapshot
    },
    get busy(): boolean {
      return busy
    },
    start(): () => void {
      const running = coalesced(read)
      load = running
      void running()
      return () => {
        running.cancel()
        load = null
        busy = false
      }
    },
    again(): void {
      void load?.()
    },
  }
}
