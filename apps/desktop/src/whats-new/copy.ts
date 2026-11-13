// The what's-new modal's words, in one place. The notes arrive from Rust as the
// release body's markdown and are drawn through the thread's sanitized pipeline.

/** The modal's heading; the version pill sits beside it in the same title. */
export const HEADING = "What's new"

/** The modal's one answer: it stores the version and clears the notes. */
export const DISMISS = 'Got it'

/** The keyboard hint beside the answer. */
export const ESC_HINT = 'Esc to close'

/** The numeric parts of a dotted version, or null when it is not one. */
function parts(version: string): number[] | null {
  if (!/^\d+(\.\d+)*$/.test(version)) return null
  return version.split('.').map(Number)
}

/**
 * "since X" for the header: only when `previous` is a version strictly older
 * than `version`, compared number by number (0.1.9 < 0.1.11). A missing,
 * equal, newer (downgrade) or unparsable previous version says nothing.
 */
export function sinceLabel(version: string, previous: string | null | undefined): string | null {
  const now = parts(version)
  const before = previous ? parts(previous) : null
  if (now === null || before === null) return null
  for (let i = 0; i < Math.max(now.length, before.length); i++) {
    const a = before[i] ?? 0
    const b = now[i] ?? 0
    if (a !== b) return a < b ? `since ${previous}` : null
  }
  return null
}
