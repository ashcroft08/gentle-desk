// The helpers view's pure rules, one test per rule: every helper status draws a
// chat mark (the approved mapping), the counts line is the one iced's card
// spells, and the thread's words are iced's own.

import { describe, expect, it } from 'vitest'
import type { HelperStatusView } from '../contract/bindings'
import type { MarkKind } from '../icons/shapes'
import {
  cardMeta,
  DOT,
  droppedLine,
  markOfStatus,
  threadHeading,
  toolStatusWord,
} from './copy'

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

describe('the thread words', () => {
  it('spells the dropped line the way iced does, singular or not', () => {
    expect(droppedLine(3)).toBe('3 earlier items were dropped')
    expect(droppedLine(1)).toBe('1 earlier items were dropped')
  })

  it('names the helper by its agent and task, or by the task alone', () => {
    expect(threadHeading('explore', 'Inventory of sidebar modules')).toBe(
      'explore - Inventory of sidebar modules',
    )
    expect(threadHeading('', 'Inventory of sidebar modules')).toBe('Inventory of sidebar modules')
    expect(threadHeading('  ', 'Only a task')).toBe('Only a task')
  })

  it('draws each tool status with iced own word', () => {
    expect(toolStatusWord('Running')).toBe('running')
    expect(toolStatusWord('Failed')).toBe('failed')
    expect(toolStatusWord('Ok')).toBe('ok')
  })
})
