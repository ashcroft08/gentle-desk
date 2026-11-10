// Where the thread box has to be after a delta, as one pure rule. jsdom cannot
// lay a box out, so this rule is what the tests exercise; the component only
// feeds it the live numbers.
//
// The two questions are separate on purpose: `nearBottom` decides what the
// READER means (a scroll event, a small slack so a pixel of rounding does not
// unstick the thread), `nextScrollTop` decides what a RENDER should do about it
// (follow the new text, or leave the reader where they are).

/** How close to the bottom still counts as being at the bottom, in pixels. */
export const NEAR_BOTTOM = 48

/** Whether a reader who is at `top` counts as being at the bottom of the box. */
export function nearBottom(top: number, height: number, view: number): boolean {
  // A box shorter than its view is not scrolled away, whatever the numbers say.
  if (height <= view) return true
  return top + view >= height - NEAR_BOTTOM
}

/**
 * The `scrollTop` the box wants after a delta: the bottom while the reader is
 * stuck to it, and `null` (leave it alone) while the reader has scrolled up.
 */
export function nextScrollTop(
  box: { scrollTop: number; scrollHeight: number; clientHeight: number },
  stick: boolean,
): number | null {
  if (!stick) return null
  return Math.max(0, box.scrollHeight - box.clientHeight)
}
