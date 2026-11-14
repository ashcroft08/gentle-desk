// The gear's dot (0.1.14): whether the sidebar footer promises an update. The
// rule is plain data, so it is testable without a window or a check.
//
// Only the rows a TERMINAL updates count. The app's own row is left out on
// purpose: its update installs itself from the banner at the top of the window,
// which is the louder place to say it, so a second promise on the gear would
// only repeat it. A row the check could not read (`null`) is unknown, never
// "behind", and a check on its way keeps the rows it already read, so the dot
// does not blink off while "Check again" runs.

import type { UpdatesView_Serialize as UpdatesView } from '../contract/bindings'

/** Whether a row the app updates from a terminal is behind. */
export function updateWaiting(updates: UpdatesView | null): boolean {
  if (updates === null) return false
  return updates.pi.update_available === true || updates.shell.update_available === true
}
