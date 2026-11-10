// The composer's two pure rules, kept out of the component so each one is
// testable on its own and nothing here reaches for the DOM.
//
// - `shouldSend`: Enter sends, Shift+Enter is a newline, and an Enter that
//   commits an IME composition (a Japanese, Chinese or Korean candidate list)
//   belongs to the candidate list, never to the chat. A composition reports
//   itself two ways - `isComposing`, and the legacy `229` key code - and both
//   are honoured, because a browser that commits with the second still fires a
//   plain Enter.
// - `fittedHeight`: the box grows with what was typed, up to the artboard's
//   maximum, and scrolls past it.

import { MAX_HEIGHT } from './copy'

/** The part of a keyboard event the send rule reads. */
export type KeyLike = {
  key: string
  shiftKey: boolean
  isComposing?: boolean
  keyCode?: number
}

/** Whether this key press asks to send the draft. */
export function shouldSend(event: KeyLike): boolean {
  if (event.key !== 'Enter' || event.shiftKey) return false
  return event.isComposing !== true && event.keyCode !== 229
}

/** How tall the box is drawn for a content height of `scrollHeight`. */
export function fittedHeight(scrollHeight: number): number {
  const content = Math.max(0, Math.trunc(scrollHeight) || 0)
  return Math.min(content, MAX_HEIGHT)
}
