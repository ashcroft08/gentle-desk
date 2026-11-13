// The context chip's rules as plain data (0.1.12, S5): what one reading of
// pi's `contextUsage` draws, decided once here so the states are testable
// without a window.
//
// The one rule that matters most: a null is NOT a zero. pi answers null for the
// tokens and the percentage right after a compaction, and it answers nothing at
// all before its first reading, so:
//
// - no field at all is no chip (never an empty stub that would claim 0%);
// - a known window with no percentage is a dash, which says "just compacted,
//   no reading yet" without inventing a number;
// - a percentage is the number pi computed, drawn with the bar under it.
//
// The colour is the reader's own scale, not pi's: quiet until 75%, the warning
// token from 75% (plan ahead), the danger token from 95% (about to compact).

import type { ContextUsageView_Serialize as ContextUsageView } from '../contract/bindings'
import { CONTEXT_DASH, CONTEXT_HIDDEN, CONTEXT_LABEL, tokensTitle } from './copy'

/** Which of the three drawn states the chip is in. */
export type ContextChipState =
  /** Nothing is read: the chip is not drawn at all. */
  | { kind: 'hidden' }
  /** A window is known but no percentage: the chip draws a dash. */
  | { kind: 'unknown'; label: string; title: string }
  /** pi gave a percentage: the number and the bar, in the tone of the scale. */
  | { kind: 'reading'; percent: number; tone: ContextTone; label: string; title: string }

/** The three tones the scale picks, each its own CSS class. */
export type ContextTone = 'normal' | 'warning' | 'danger'

/** The tone a percentage falls in: the reader's own scale, not pi's. */
export function toneOf(percent: number): ContextTone {
  if (percent >= 95) return 'danger'
  if (percent >= 75) return 'warning'
  return 'normal'
}

/** The chip's whole state, from the reading Rust pushed. */
export function chipOf(usage: ContextUsageView | null): ContextChipState {
  if (usage === null) return { kind: 'hidden' }
  const { tokens, percent, window } = usage
  if (tokens === null && percent === null && window === null) return { kind: 'hidden' }
  const title = tokensTitle(tokens, window)
  if (percent === null) return { kind: 'unknown', label: CONTEXT_HIDDEN, title }
  const rounded = Math.round(percent)
  return {
    kind: 'reading',
    percent: rounded,
    tone: toneOf(rounded),
    label: CONTEXT_LABEL(rounded),
    title,
  }
}

/** The dash the chip draws when the percentage is not known. */
export const DASH = CONTEXT_DASH

/**
 * How full the bar is, as a share of its own width: the percentage clamped to
 * the ends, so a reading pi rounded above 100 cannot overflow the chip.
 */
export function barWidth(percent: number): string {
  return `${Math.min(100, Math.max(0, percent))}%`
}
