// The helpers panel's drawn words and the two rules the card needs: which chat
// mark stands for a helper status, and the one line that sums up what a helper
// is doing. The status label, the task title and the counters line all come
// from Rust (`gentle_runtime::helper_text`), so nothing here re-spells them.

import type { HelperStatusView } from '../contract/bindings'
import type { MarkKind } from '../icons/shapes'

/** The listbox's accessible name, as 01b writes it. */
export const HELPERS_LIST_LABEL = 'Helpers'

/** What an open chat with no delegated helper says (T7b-1 decision). */
export const NO_HELPERS = 'No helpers for this chat'

/**
 * The separator between the parts of a card's line: the one iced's card uses
 * (U+00B7). Written as an escape so the source stays ASCII, and drawn, never an
 * icon.
 */
export const DOT = '\u00b7'

/**
 * The chat mark that stands for a helper status (approved T7b-1 mapping). The
 * shape is only the picture: the card's accessible name for it is the status
 * label Rust sent, so the text, not the colour, carries the state.
 */
export function markOfStatus(status: HelperStatusView): MarkKind {
  switch (status) {
    case 'Running':
      return 'Working'
    case 'Queued':
    case 'Waiting':
      return 'Queued'
    case 'Done':
      return 'NewReply'
    case 'Failed':
    case 'Cancelled':
      return 'Ended'
  }
}

/** "2 turns <dot> 1 tool calls", the counts iced's card spells out. */
export function cardMeta(turns: number, toolCalls: number): string {
  return `${turns} turns ${DOT} ${toolCalls} tool calls`
}
