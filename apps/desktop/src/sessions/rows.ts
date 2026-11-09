// The pure rules the sidebar tree needs: how tall a row is, which rows a scroll
// position shows, and which kind a row is. Row heights and the row math are
// display rules only; the rows themselves come from Rust one window at a time
// (S3), already flattened: a workspace header, then its chats.

import type { SessionTreeRow_Serialize } from '../contract/bindings'
import type { MarkKind } from '../icons/shapes'

/** The fixed height of one row (design/system.md, `--row-h`). */
export const ROW_HEIGHT = 34

/** One row of the tree as `sessions_window` answers it. */
export type TreeRow = SessionTreeRow_Serialize

/** A workspace header row: the folder and the state of its group. */
export type WorkspaceRow = Extract<TreeRow, { kind: 'Workspace' }>

/** A chat row: the same row the flat list served, under its folder. */
export type ChatRow = Extract<TreeRow, { kind: 'Chat' }>

/**
 * Whether `row` is a workspace header rather than a chat. Headers and chats
 * share {@link ROW_HEIGHT}, so the tree is one virtualized list of rows and
 * this guard is the only thing a caller needs to draw one.
 */
export function isWorkspaceRow(row: TreeRow): row is WorkspaceRow {
  return row.kind === 'Workspace'
}

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
 * heights that keep the scrollbar honest for a tree of 50 000 rows.
 *
 * `total` is the count Rust answers with: workspace headers included, so the
 * scrollbar measures the rows that really exist.
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

/**
 * The workspace each row of one page belongs to: a header to itself, a chat to
 * the closest header above it. A page that starts in the middle of a group has
 * chats before any header, which belong to `null`: their header is not drawn,
 * so their row has no new-chat icon to reveal.
 */
export function rowWorkspaceKeys(rows: TreeRow[]): (string | null)[] {
  const keys: (string | null)[] = []
  let current: string | null = null
  for (const row of rows) {
    if (isWorkspaceRow(row)) current = row.key
    keys.push(current)
  }
  return keys
}

/**
 * The folder a chat row sits under, as the path an intent names, or `null` when
 * the page does not carry that chat's header.
 */
export function workspacePathOfChat(rows: TreeRow[], chatId: string): string | null {
  let path: string | null = null
  for (const row of rows) {
    if (isWorkspaceRow(row)) path = row.path
    else if (row.id === chatId) return path
  }
  return null
}
