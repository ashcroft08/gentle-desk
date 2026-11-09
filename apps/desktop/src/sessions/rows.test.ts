import { describe, expect, it } from 'vitest'
import { markKindOf, ROW_HEIGHT, virtualWindow } from './rows'

describe('ROW_HEIGHT', () => {
  it('is the fixed row height of the design system', () => {
    expect(ROW_HEIGHT).toBe(34)
  })
})

describe('virtualWindow', () => {
  it('draws nothing for an empty list', () => {
    expect(virtualWindow(0, 0, 340)).toEqual({ first: 0, count: 0, top: 0, bottom: 0 })
  })

  it('draws the one row of a one-row list and nothing on the last page', () => {
    expect(virtualWindow(1, 0, 340)).toEqual({ first: 0, count: 1, top: 0, bottom: 0 })
  })

  it('draws every row when the list is shorter than the viewport', () => {
    expect(virtualWindow(3, 0, 340)).toEqual({ first: 0, count: 3, top: 0, bottom: 0 })
  })

  it('reserves the spacers around the viewport of a long list', () => {
    expect(virtualWindow(100, 0, 340)).toEqual({ first: 0, count: 10, top: 0, bottom: 3060 })
  })

  it('stops at the last full page instead of scrolling past it', () => {
    expect(virtualWindow(100, 5000, 340)).toEqual({ first: 90, count: 10, top: 3060, bottom: 0 })
  })

  it('rounds a partly visible row into the viewport', () => {
    expect(virtualWindow(100, 40, 35)).toEqual({ first: 1, count: 2, top: 34, bottom: 3298 })
  })

  it('draws nothing when the viewport has no height', () => {
    expect(virtualWindow(10, 0, 0)).toEqual({ first: 0, count: 0, top: 0, bottom: 0 })
  })
})

describe('markKindOf', () => {
  it('names every mark the contract carries', () => {
    for (const kind of ['Working', 'Queued', 'Opening', 'NewReply', 'NeedsAttention', 'Ended']) {
      expect(markKindOf(kind)).toBe(kind)
    }
  })

  it('draws no mark for a name the contract does not carry', () => {
    expect(markKindOf('Idle')).toBeNull()
  })
})
