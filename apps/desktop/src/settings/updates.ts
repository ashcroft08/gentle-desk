// The Updates pane's rules as plain data (0.1.12, S2, S4, S7; 0.8.1 S6, S8): which
// of the card's shapes the check calls for, what each of its three rows draws, and
// which row carries the one "Skip this version".
//
// Three states matter, and none of them is a guess:
//
// - `checked` is false until the check the app runs at open has answered, so an
//   empty card is never drawn before it did (S3);
// - a row whose `update_available` is null is UNKNOWN, never "up to date": a
//   registry that did not answer, the shell's own channel, and the installed
//   version's own read all leave it null (the Rust half says why), and a row that
//   has no answer says which read failed in one short sentence (S8);
// - only pi and the shell are updated from a terminal, so only they ever offer an
//   action, and the shell's is its own `upgrade` (S7). The app's own release
//   installs itself from the update banner, and its row says so.
//
// "Skip this version" (S1, S2) adds two more answers to the card, both read from
// `UpdatesView::skipped` beside the rows: the one version the button would skip and
// the row that publishes the skipped one, which is what offers the way back. The
// contract stores ONE version, so D9 puts the skip on the row whose `latest` is the
// greatest version - the newest thing on offer - and each row is compared with it.
// one stored version can only name one of them.
//
// packages-status T2 (S1, S3) adds one function beside the card, `packagesOf`,
// with the Extensions block's honest states, and lets the one last-checked
// line take the newest instant either read reported.
// The check itself always reports the real state, so a skipped version is still
// drawn as published here: hiding it is the startup check's business.
//
// packages-status T6 (S6, S7) adds two things to that: a FIFTH state, `unchecked`
// - the listing exists and pi's own check has not run in that child, so "al día"
// would be a claim nobody made - and `updatesDrawn`, which is which packages
// slice the dialog draws when the load's answer and the pushed check disagree.

import type {
  PackagesView_Serialize as PackagesView,
  UpdatesView_Serialize as UpdatesView,
  VersionRowView_Serialize as VersionRow,
} from '../contract/bindings'
import { parseVersion, sameVersion } from '../update/version'
import type { ReasonRow, UpdatesCopy } from './updates-copy'

/** The one terminal action a row offers, if any (0.8.1 S7). */
export type RowAction = 'update' | 'upgrade'

/** The single skip button a row carries: the offer, or the way back. */
export type RowSkip = 'skip' | 'unskip'

/** One row of the card: one piece of this machine, its versions and its action. */
export type UpdateRow = {
  key: ReasonRow
  /** The piece's name, as the pane's dictionary spells it (D10). */
  label: string
  /** The version column: `current -> latest` when it is behind, else current. */
  version: string
  current: string | null
  latest: string | null
  /** This row has something newer published. */
  behind: boolean
  /** The comparison could be made: `false` is unknown, never "up to date". */
  known: boolean
  /** The one sentence under the row - why a read failed, or what a skip means. */
  note: string | null
  /** The terminal window this row opens, or `null` when it offers none. */
  action: RowAction | null
  /** Whether this row carries the skip button, and which half. */
  skip: RowSkip | null
}

/** What the pane draws, worked out from the pushed check. */
export type UpdatesCard = {
  /** The check answered at least once: the card is drawn. */
  checked: boolean
  /** The check is on its way: a quiet line, and the last time it was read. */
  checking: boolean
  /** At least one row is behind. */
  available: boolean
  /** Every row was read and none is behind: the only honest "up to date". */
  upToDate: boolean
  /** A row nobody could read: the tag says "not checked" instead of "up to date". */
  partial: boolean
  /** A row a terminal window updates is behind, which is what the note says. */
  terminal: boolean
  /** The three rows, in the order the pane names them. */
  rows: UpdateRow[]
  /** When the last finished check read the machine, or `null` before any. */
  checkedMs: number | null
}

/** One extension pi's own check reports as behind, as the block draws it. */
export type PackageRow = {
  /** pi's own display name for it: never a source, a path or a scope (S4). */
  name: string
  /** The version it runs when pi reports one, or `null` to draw no version. */
  version: string | null
}

/**
 * The Extensions block's one honest state (packages-status T2, S1, S3; T6, S6):
 * the listing that named the behind ones, the listing pi has not checked yet, the
 * answer that named none, the read that could not run, and the one nothing asked -
 * never a sixth, and never a guess. A `behind` block whose `unchecked` is set keeps
 * drawing its line under the names: the list is not the whole story then.
 */
export type PackagesCard =
  | { state: 'behind'; count: number; rows: PackageRow[]; unchecked: boolean }
  | { state: 'unchecked' }
  | { state: 'upToDate' }
  | { state: 'unreadable' }
  | { state: 'unasked' }

/** The three rows of the check, with the action each one offers. */
function rowsOf(updates: UpdatesView): { key: ReasonRow; row: VersionRow; action: RowAction | null }[] {
  return [
    // The app's own release is the banner's job, so its row has no button.
    { key: 'app', row: updates.desk, action: null },
    { key: 'pi', row: updates.pi, action: 'update' },
    { key: 'shell', row: updates.shell, action: 'upgrade' },
  ]
}

/** Which of the three the row is, in the dictionary's own words. */
function labelOf(key: ReasonRow, copy: UpdatesCopy): string {
  if (key === 'app') return copy.appLabel
  return key === 'pi' ? copy.piLabel : copy.shellLabel
}

/** Three versions compared as numbers, never as text. */
function compare(a: [number, number, number], b: [number, number, number]): number {
  return a[0] - b[0] || a[1] - b[1] || a[2] - b[2]
}

/**
 * The row the one skip belongs to (D9): the greatest `latest` among the rows a
 * terminal updates, compared as `x.y.z`, and never the version already skipped -
 * that one is the row offering the way back instead.
 */
function skipRow(
  rows: { key: ReasonRow; row: VersionRow; action: RowAction | null }[],
  skipped: string | null,
): ReasonRow | null {
  let best: { key: ReasonRow; version: [number, number, number] } | null = null
  for (const { key, row, action } of rows) {
    if (action === null || row.update_available !== true) continue
    if (sameVersion(row.latest, skipped)) continue
    const version = parseVersion(row.latest)
    if (version === null) continue
    if (best === null || compare(version, best.version) > 0) best = { key, version }
  }
  return best?.key ?? null
}

/** The newer of the two instants the card knows, or the one it has. */
function newest(rows: number | null, packages: number | null): number | null {
  if (rows === null) return packages
  if (packages === null) return rows
  return Math.max(rows, packages)
}

/**
 * Whether pi's own check names the extensions that are behind (packages-status
 * T2, S1, S3; T6, S6): only a listing that answered can be `behind`,
 * `unchecked` or `upToDate`, a read that ran and could not answer is `unreadable`,
 * and no answer at all - no chat open, or an older shell that does not advertise
 * the read (S5) - is `unasked`.
 *
 * "Up to date" is only ever the answered listing pi's own check ran for and that
 * has nothing behind it: a listing with an item still `unknown` is `unchecked`,
 * and one with names behind AND items unknown carries both, because a behind list
 * nobody finished checking must never read as complete.
 */
export function packagesOf(packages: PackagesView | null): PackagesCard {
  if (packages === null) return { state: 'unasked' }
  if (!packages.available) return { state: packages.unknown ? 'unreadable' : 'unasked' }
  // A listing pi's own check cut short is drawn as the names it carried: the count
  // is those, and the block names no extension it did not read.
  const rows: PackageRow[] = packages.behind.map((each) => ({
    name: each.name,
    version: each.current ?? null,
  }))
  if (rows.length > 0) {
    return { state: 'behind', count: rows.length, rows, unchecked: packages.unchecked }
  }
  return packages.unchecked ? { state: 'unchecked' } : { state: 'upToDate' }
}

/** Whether pi's own check has finished for the whole listing (S6). */
function checkFinished(packages: PackagesView | null): packages is PackagesView {
  return packages !== null && packages.available && !packages.unchecked
}

/**
 * The updates the dialog draws (packages-status T6, S7): the push until the one
 * load answers, and the load's answer afterwards - the expression the rail and the
 * pane already shared - EXCEPT for the packages slice.
 *
 * The explicit check's result arrives by PUSH, so a check that finishes while
 * Settings is open lives only in the live slice: the pushed packages are drawn
 * when they are a FINISHED check (`available` and nothing left `unknown`) and the
 * load's are not. A pushed slice pi has not checked never replaces one that was,
 * and neither does anything a push carries about the three rows: only the
 * packages move.
 */
export function updatesDrawn(
  pushed: UpdatesView | null,
  snapshot: UpdatesView | null,
): UpdatesView | null {
  const base = snapshot ?? pushed
  const packages = pushed?.packages ?? null
  const better = checkFinished(packages) && !checkFinished(snapshot?.packages ?? null)
  return base === null || !better ? base : { ...base, packages }
}

/**
 * The whole card, from the check Rust pushed, named in the language the pane is
 * drawn in (0.2.0, D4): the labels and the reason sentences are the app's own
 * words, so they arrive as the pane's dictionary answers them; the versions inside
 * them are the check's (D7).
 */
export function cardOf(updates: UpdatesView | null, copy: UpdatesCopy): UpdatesCard {
  const empty: UpdatesCard = {
    checked: false,
    checking: false,
    available: false,
    upToDate: false,
    partial: false,
    terminal: false,
    rows: [],
    checkedMs: null,
  }
  if (updates === null) return empty

  // The last read is a datum of its own: a check on its way does not erase it.
  // packages-status T2: the packages slice carries pi's own check instant, so the
  // one line takes the NEWEST of the two - pi's read when it just ran, the rows'
  // own read when that is the newer one. An older instant adds nothing, and the
  // card never draws a second check line.
  const checkedMs = newest(updates.checked_ms ?? null, updates.packages.checked ?? null)
  if (updates.status === 'Checking') return { ...empty, checking: true, checkedMs }
  if (updates.status === 'Unknown') return { ...empty, checkedMs }

  const all = rowsOf(updates)
  const skipped = updates.skipped ?? null
  const available = all.some(({ row }) => row.update_available === true)
  const partial = all.some(({ row }) => row.update_available === null)
  const terminal = all.some(({ row, action }) => action !== null && row.update_available === true)
  const target = skipRow(all, skipped)

  const rows: UpdateRow[] = all.map(({ key, row, action }) => {
    const behind = row.update_available === true
    const skipping = sameVersion(row.latest, skipped)
    // A row that could not be read says which read failed and why (S8); the app's
    // own row says where its update comes from; the skipped one says what a skip
    // means. One sentence at most, and never two.
    const note =
      row.reason != null
        ? copy.rowReason(key, row.reason)
        : key === 'app' && behind
          ? copy.appUpdatesItself
          : skipping && row.latest !== null
            ? copy.skippedNote(row.latest)
            : null
    return {
      key,
      label: labelOf(key, copy),
      version: behind
        ? `${row.current ?? copy.unknownVersion} \u2192 ${row.latest ?? copy.unknownVersion}`
        : (row.current ?? copy.unknownVersion),
      current: row.current,
      latest: row.latest,
      behind,
      known: row.update_available !== null,
      note,
      // A terminal window is offered for a row that IS behind: a button that runs
      // one for nothing newer would run it for nothing.
      action: behind ? action : null,
      skip: target === key ? 'skip' : skipping ? 'unskip' : null,
    }
  })

  return {
    checked: true,
    checking: false,
    available,
    upToDate: !available && !partial,
    partial,
    terminal,
    rows,
    checkedMs,
  }
}
