// What one chat's mark means for its composer: the parity rule of the migration
// (S2, T6) as plain data, so the component draws a decision instead of making
// one.
//
// One turn at a time. While a turn runs the button is Stop; while the chat is
// opening, waiting for a slot, or gone, Send is disabled and the line below the
// box says why. The text itself is never touched by any of it: a disabled Send
// keeps the draft, and the reducer refuses a prompt for exactly these states, so
// the view and the state tell the user the same story.

import type { ChatActivityMark } from '../contract/bindings'
import { ENDED_REASON, OPENING_REASON, QUEUED_REASON } from './copy'

/** Which of the two actions the composer shows. */
export type ComposerMode = 'send' | 'stop'

export type ComposerState = {
  mode: ComposerMode
  /** Whether Send is disabled (drawn with `aria-disabled`, never taken away). */
  disabled: boolean
  /** Why Send is disabled, as the visible line Send is described by. */
  reason: string | null
}

/**
 * The composer for a chat with `mark` (its activity, `null` when idle).
 *
 * Working turns the button into Stop, because the one thing a running turn takes
 * is a stop. Queued and opening take no prompt yet, and an ended chat takes none
 * until it is restarted; each says so. A finished reply and a pending dialog
 * change nothing here: both leave the composer ready for the next prompt.
 */
export function composerStateOf(mark: ChatActivityMark | null): ComposerState {
  switch (mark) {
    case 'Working':
      return { mode: 'stop', disabled: false, reason: null }
    case 'Queued':
      return { mode: 'send', disabled: true, reason: QUEUED_REASON }
    case 'Opening':
      return { mode: 'send', disabled: true, reason: OPENING_REASON }
    case 'Ended':
      return { mode: 'send', disabled: true, reason: ENDED_REASON }
    default:
      return { mode: 'send', disabled: false, reason: null }
  }
}
