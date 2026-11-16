// The mark a state draws carries the class the stylesheet animates (S1): the
// pulse rides `.mark.m-working`, so the Working mark must DRAWN that class and
// the five still states must not borrow it. The glyph, the colour and the
// placement are `Mark`'s own and do not move.

import { describe, expect, it } from 'vitest'
import { render } from '@testing-library/svelte'
import Mark from '../icons/Mark.svelte'
import type { MarkKind } from '../icons/shapes'

/** Every state and the one class 01a names for it. */
const CLASSES: [MarkKind, string][] = [
  ['Working', 'm-working'],
  ['Queued', 'm-queued'],
  ['Opening', 'm-opening'],
  ['NewReply', 'm-reply'],
  ['NeedsAttention', 'm-attn'],
  ['Ended', 'm-ended'],
]

describe('a drawn mark', () => {
  it.each(CLASSES)('draws %s with its one state class (%s)', (kind, name) => {
    const { container } = render(Mark, { props: { kind } })
    const svg = container.querySelector('svg')

    expect(svg?.classList.contains('mark')).toBe(true)
    expect(svg?.classList.contains(name)).toBe(true)
  })

  it('gives no state another state to move with', () => {
    for (const [kind, name] of CLASSES) {
      const { container, unmount } = render(Mark, { props: { kind } })
      const drawn = [...(container.querySelector('svg')?.classList ?? [])].filter((one) =>
        one.startsWith('m-'),
      )
      expect(drawn, kind).toEqual([name])
      unmount()
    }
  })
})
