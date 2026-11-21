// The gear's dot (0.1.14): whether the sidebar footer promises an update. The
// rule is plain data, so it is testable without a window or a check.
//
// Only the rows a TERMINAL updates count. The app's own row is left out on
// purpose: its update installs itself from the banner at the top of the window,
// which is the louder place to say it, so a second promise on the gear would
// only repeat it. A row the check could not read (`null`) is unknown, never
// "behind", and a check on its way keeps the rows it already read, so the dot
// does not blink off while "Check again" runs.
//
// The version the user skipped is no promise either (0.8.0 settings-polish S2):
// they already said no to it, and the check the app runs when it opens does not
// offer it again. A version NEWER than the skipped one is a promise as usual, and
// the skip is read as a version, so a value this app cannot compare hides nothing.

import type {
  UpdatesView_Serialize as UpdatesView,
  VersionRowView_Serialize as VersionRow,
} from '../contract/bindings'
import { sameVersion } from './version'

/** Whether `row` is behind and is not the version the user skipped. */
function waiting(row: VersionRow, skipped: string | null): boolean {
  return row.update_available === true && !sameVersion(row.latest, skipped)
}

/** Whether a row the app updates from a terminal is behind. */
export function updateWaiting(updates: UpdatesView | null): boolean {
  if (updates === null) return false
  const skipped = updates.skipped ?? null
  return waiting(updates.pi, skipped) || waiting(updates.shell, skipped)
}
