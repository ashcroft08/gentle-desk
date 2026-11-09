// The pure rules a session row needs: how tall a row is, which rows a scroll
// position shows, and which mark a chat activity name draws. All three are
// display rules only; the list itself comes from Rust one window at a time
// (S3) and the mark's colour comes from the design system's CSS variables.

import type { MarkKind } from '../icons/shapes'

/** The fixed height of one row (design/system.md, `--row-h`). */
export const ROW_HEIGHT = 34

export type VirtualWindow = {
  /** Index of the first row to draw. */
  first: number
  /** How many rows to draw below it. */
  count: number
  /** Height of the spacer above the drawn rows, in pixels. */
  top: number
  /** Height of the spacer below the drawn rows, in pixels. */
  bottom: number
}

/**
 * Which rows a viewport shows: the first one, how many fit, and the two spacer
 * heights that keep the scrollbar honest for a list of 50 000 rows.
 *
 * The window stops at the last full page, so the last rows are never scrolled
 * past and the bottom spacer reaches exactly zero there.
 */
export function virtualWindow(
  total: number,
  scrollTop: number,
  viewportHeight: number,
): VirtualWindow {
  const rows = Math.max(0, Math.trunc(total) || 0)
  const fits = Math.max(0, Math.ceil(viewportHeight / ROW_HEIGHT))
  if (rows === 0 || fits === 0) return { first: 0, count: 0, top: 0, bottom: 0 }
  const maxFirst = Math.max(0, rows - fits)
  const scrolled = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) || 0)
  const first = Math.min(maxFirst, scrolled)
  const count = Math.min(fits, rows - first)
  return {
    first,
    count,
    top: first * ROW_HEIGHT,
    bottom: (rows - first - count) * ROW_HEIGHT,
  }
}

/**
 * The names `ViewState.activity` carries, as the generated contract spells them
 * (`ChatActivityMark` in `src/contract/bindings.ts`). Never invented here.
 */
export const MARK_KINDS: readonly MarkKind[] = [
  'Working',
  'Queued',
  'Opening',
  'NewReply',
  'NeedsAttention',
  'Ended',
]

/**
 * The mark a chat activity name draws, or `null` for a name the contract does
 * not carry (a drifted value draws no mark rather than a wrong one).
 */
export function markKindOf(activity: string): MarkKind | null {
  return (MARK_KINDS as readonly string[]).includes(activity) ? (activity as MarkKind) : null
}
