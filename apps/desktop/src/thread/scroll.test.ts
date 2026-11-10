// The scroll rule of the thread, as one pure helper: where the box wants to be
// after a delta, and whether the reader still counts as being at the bottom.
// jsdom cannot lay a box out, so the rule is tested here rather than through a
// component.

import { describe, expect, it } from 'vitest'
import { NEAR_BOTTOM, nearBottom, nextScrollTop } from './scroll'

describe('thread scrolling', () => {
  it('sticks to the bottom while the reader is at the bottom', () => {
    // 500 - 100 = 400 is the bottom of this box.
    expect(nextScrollTop({ scrollTop: 120, scrollHeight: 500, clientHeight: 100 }, true)).toBe(400)
  })

  it('does not jump while the reader has scrolled up', () => {
    expect(
      nextScrollTop({ scrollTop: 120, scrollHeight: 500, clientHeight: 100 }, false),
    ).toBeNull()
  })

  it('counts a reader within the slack as being at the bottom, and past it as not', () => {
    // The bottom is 800; the slack is 48 px.
    expect(nearBottom(800, 900, 100)).toBe(true)
    expect(nearBottom(800 - NEAR_BOTTOM, 900, 100)).toBe(true)
    expect(nearBottom(800 - NEAR_BOTTOM - 1, 900, 100)).toBe(false)
  })

  it('never treats a box shorter than its view as scrolled away', () => {
    expect(nearBottom(0, 40, 100)).toBe(true)
  })
})
