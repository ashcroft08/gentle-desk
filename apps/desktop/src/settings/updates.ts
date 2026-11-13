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

import type {
  UpdatesView_Serialize as UpdatesView,
  VersionRowView_Serialize as VersionRow,
} from '../contract/bindings'
import { APP_LABEL, PI_LABEL, SHELL_LABEL } from './updates-copy'

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
}

/** The three rows of the check, with the labels the pane draws them under. */
function rows(updates: UpdatesView): { label: string; row: VersionRow }[] {
  return [
    { label: APP_LABEL, row: updates.desk },
    { label: PI_LABEL, row: updates.pi },
    { label: SHELL_LABEL, row: updates.shell },
  ]
}

/** The whole card, from the check Rust pushed. */
export function cardOf(updates: UpdatesView | null): UpdatesCard {
  const empty: UpdatesCard = {
    checked: false,
    checking: false,
    available: false,
    headlines: [],
    versions: [],
    terminal: false,
    appOnly: false,
  }
  if (updates === null) return empty

  const all = rows(updates)
  const versions = all.map(({ label, row }) => ({ label, version: row.current }))
  const behind = all.filter(({ row }) => row.update_available === true)
  const available = behind.length > 0
  const terminal = updates.pi.update_available === true || updates.shell.update_available === true

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
  }
}
