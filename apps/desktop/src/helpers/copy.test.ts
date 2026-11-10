// The helpers card's two pure rules, one test per rule: every helper status
// draws a chat mark (the approved mapping), and the counts line is the one
// iced's card spells.

import { describe, expect, it } from 'vitest'
import type { HelperStatusView } from '../contract/bindings'
import type { MarkKind } from '../icons/shapes'
import { cardMeta, DOT, markOfStatus } from './copy'

const MARKS: [HelperStatusView, MarkKind][] = [
  ['Running', 'Working'],
  ['Queued', 'Queued'],
  ['Waiting', 'Queued'],
  ['Done', 'NewReply'],
  ['Failed', 'Ended'],
  ['Cancelled', 'Ended'],
]

describe('markOfStatus', () => {
  it.each(MARKS)('draws %s with the %s chat mark', (status, mark) => {
    expect(markOfStatus(status)).toBe(mark)
  })
})

describe('cardMeta', () => {
  it('spells the turns and the tool calls the way iced does', () => {
    expect(cardMeta(2, 1)).toBe(`2 turns ${DOT} 1 tool calls`)
  })

  it('separates with the middle dot iced uses, never an icon', () => {
    expect(DOT).toBe('\u00b7')
  })
})
