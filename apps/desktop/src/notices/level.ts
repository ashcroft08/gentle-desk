// The level rule of a notice, as the states board (00-states-board.html) draws
// it: warning and error carry a mark whose SHAPE says which it is, an error is
// announced as an alert while the other levels are a status, and every level has
// its own word so the level is never carried by colour alone.
//
// The rule is drawn by this module and SAID by `copy.ts`: the word is one of the
// notice's own words, so it lives in the language dictionary with the rest of
// them (0.2.0, D4/D5) and `noticeLookOf` answers only what does not depend on the
// language.

import type { NoticeLevel } from '../contract/bindings'
import type { MarkKind } from '../icons/shapes'

export type NoticeLook = {
  /** How the card is announced: an error interrupts, the rest do not. */
  role: 'alert' | 'status'
  /** The board's mark for the level, `null` for info (the board draws none). */
  mark: MarkKind | null
  /** The board's class for the level's left edge. */
  tone: string
}

/**
 * How one level is drawn. The three cases are the contract's three levels, and
 * the level's own word - the one a screen reader hears before the text - is
 * `noticesCopy(language).levels[level]`.
 */
export function noticeLookOf(level: NoticeLevel): NoticeLook {
  switch (level) {
    case 'Error':
      return { role: 'alert', mark: 'Ended', tone: 'err' }
    case 'Warning':
      return { role: 'status', mark: 'NeedsAttention', tone: 'warn' }
    case 'Info':
      return { role: 'status', mark: null, tone: 'info' }
  }
}
