// The Updates section's check as plain data (settings-dialog S6): nothing asked
// yet, the ask on its way, or the answer. Pure, so the drawn states stay testable
// without a window or a network; the check itself is `update/plugin`'s seam - the
// same one the window's startup check runs - and its answer is taken as it came.

import type { CheckResult } from '../update/plugin'

export type CheckState = { kind: 'idle' } | { kind: 'checking' } | CheckResult

/** Nothing has been asked yet: the section draws no result line. */
export function notChecked(): CheckState {
  return { kind: 'idle' }
}

/** The button was pressed: the check is on its way, so a second press is not. */
export function checking(): CheckState {
  return { kind: 'checking' }
}
