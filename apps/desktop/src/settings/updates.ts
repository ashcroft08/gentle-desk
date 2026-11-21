// The Updates pane's rules as plain data (0.1.12, S2, S4, S7): which of the card's
// three shapes the pushed check calls for, the headline of every row that found
// something newer, and the quiet versions row under it.
//
// Three states matter, and none of them is a guess:
//
// - `checked` is false until the check the app runs at open has answered, so an
//   empty card is never drawn before it did (S3);
// - a row whose `update_available` is null is UNKNOWN, never "up to date": a
//   registry that did not answer and the shell's own channel both leave it null
//   (the Rust half says why);
// - only pi and the shell are updated from a terminal, so only they offer the
//   Update button; the app's own release installs itself from the update banner.
//
// "Skip this version" (0.8.0 settings-polish S1, S2) adds two more answers to the
// card, both read from `UpdatesView::skipped` beside the rows: the version the
// button would skip (one a terminal updates, since that is what the pane's own
// Update runs) and whether the version on offer is already the skipped one, which
// is what offers the way back. The check itself always reports the real state, so
// a skipped version is still drawn as published here - hiding it is the startup
// check's business, never this rule's.

import type {
  UpdatesView_Serialize as UpdatesView,
  VersionRowView_Serialize as VersionRow,
} from '../contract/bindings'
import { sameVersion } from '../update/version'
import type { UpdatesCopy } from './updates-copy'

/** One piece of this machine, as the quiet versions row names it. */
export type VersionLine = { label: string; version: string | null }

/** One row that found something newer: what to say above the versions. */
export type Headline = { label: string; current: string | null; latest: string | null }

/** What the pane draws, worked out from the pushed check. */
export type UpdatesCard = {
  /** The check answered at least once: the card is drawn. */
  checked: boolean
  /** The check is on its way: a quiet line, and the versions it already had. */
  checking: boolean
  /** At least one row is behind. */
  available: boolean
  /** The rows that are behind, in the order the pane names them. */
  headlines: Headline[]
  /** The three pieces and the version each runs. */
  versions: VersionLine[]
  /** pi or the shell is behind, which is what a terminal window updates. */
  terminal: boolean
  /** Only the app itself is behind: the update banner installs that one. */
  appOnly: boolean
  /**
   * The version "Skip this version" would skip, or `null` when there is nothing
   * to skip: no row a terminal updates is behind, or the one that is holds the
   * version the user already skipped.
   */
  skipTarget: string | null
  /**
   * Whether a row publishes the version the user skipped, which is what draws
   * "Stop skipping" instead of the skip itself. It is read beside the row and not
   * from `update_available`, so it holds for both halves of S2: the explicit check
   * that still reports the skipped version, and the startup pass that does not
   * offer it.
   */
  skipping: boolean
}

/** One row of the check, with the label it is drawn under and who updates it. */
type Row = { label: string; row: VersionRow; terminal: boolean }

/** The three rows of the check, with the labels the pane draws them under. */
function rows(updates: UpdatesView, copy: UpdatesCopy): Row[] {
  return [
    { label: copy.appLabel, row: updates.desk, terminal: false },
    { label: copy.piLabel, row: updates.pi, terminal: true },
    { label: copy.shellLabel, row: updates.shell, terminal: true },
  ]
}

/**
 * The whole card, from the check Rust pushed, named in the language the pane is
 * drawn in (0.2.0, D4): the labels are the app's own words, so they arrive as the
 * pane's dictionary answers them; the versions inside them are the check's (D7).
 */
export function cardOf(updates: UpdatesView | null, copy: UpdatesCopy): UpdatesCard {
  const empty: UpdatesCard = {
    checked: false,
    checking: false,
    available: false,
    headlines: [],
    versions: [],
    terminal: false,
    appOnly: false,
    skipTarget: null,
    skipping: false,
  }
  if (updates === null) return empty

  const all = rows(updates, copy)
  const versions = all.map(({ label, row }) => ({ label, version: row.current }))
  const behind = all.filter(({ row }) => row.update_available === true)
  const available = behind.length > 0
  const terminal = updates.pi.update_available === true || updates.shell.update_available === true
  // The skip is the check's own version string, never a mode: it names one
  // version, compared as a version (so `2.0.5` and `02.0.5` are one).
  const skipped = updates.skipped ?? null
  const skipping = all.some(({ row }) => sameVersion(row.latest, skipped))
  const target = behind.find(
    ({ row, terminal }) => terminal && row.latest !== null && !sameVersion(row.latest, skipped),
  )

  if (updates.status === 'Checking') {
    return { ...empty, checking: true, versions }
  }
  if (updates.status === 'Unknown') return empty
  return {
    checked: true,
    checking: false,
    available,
    headlines: behind.map(({ label, row }) => ({
      label,
      current: row.current,
      latest: row.latest,
    })),
    versions,
    terminal,
    appOnly: available && !terminal && updates.desk.update_available === true,
    skipTarget: target?.row.latest ?? null,
    skipping,
  }
}
