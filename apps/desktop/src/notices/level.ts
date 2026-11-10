// The level rule of a notice, as the states board (00-states-board.html) draws
// it: warning and error carry a mark whose SHAPE says which it is, an error is
// announced as an alert while the other levels are a status, and every level has
// its own word so the level is never carried by colour alone.

import type { NoticeLevel } from '../contract/bindings'
import type { MarkKind } from '../icons/shapes'

export type NoticeLook = {
  /** How the card is announced: an error interrupts, the rest do not. */
  role: 'alert' | 'status'
  /** The board's mark for the level, `null` for info (the board draws none). */
  mark: MarkKind | null
  /** The level's own word, which screen readers hear before the text. */
  word: string
  /** The board's class for the level's left edge. */
  tone: string
}

/** How one level is drawn. The three cases are the contract's three levels. */
export function noticeLookOf(level: NoticeLevel): NoticeLook {
  switch (level) {
    case 'Error':
      return { role: 'alert', mark: 'Ended', word: 'Error', tone: 'err' }
    case 'Warning':
      return { role: 'status', mark: 'NeedsAttention', word: 'Warning', tone: 'warn' }
    case 'Info':
      return { role: 'status', mark: null, word: 'Info', tone: 'info' }
  }
}
