// The two rules of the Chat | Helpers tablist and the tab's badge, one test per
// rule: the arrow keys walk the list and wrap, Home and End jump to its ends,
// and every other key keeps its own meaning.

import { describe, expect, it } from 'vitest'
import { TABS, tabAfter, workingBadgeName } from './tabs'

describe('tabAfter', () => {
  it('walks the list with the arrow keys, wrapping at both ends', () => {
    expect(tabAfter('ArrowRight', 'chat')).toBe('helpers')
    expect(tabAfter('ArrowRight', 'helpers')).toBe('chat')
    expect(tabAfter('ArrowLeft', 'helpers')).toBe('chat')
    expect(tabAfter('ArrowLeft', 'chat')).toBe('helpers')
    expect(tabAfter('ArrowDown', 'chat')).toBe('helpers')
    expect(tabAfter('ArrowUp', 'chat')).toBe('helpers')
  })

  it('jumps to the first and last tab with Home and End', () => {
    expect(tabAfter('Home', 'helpers')).toBe(TABS[0])
    expect(tabAfter('End', 'chat')).toBe(TABS[TABS.length - 1])
  })

  it('leaves every other key alone', () => {
    for (const key of ['Enter', 'Tab', 'Escape', 'ArrowRightish', 'a']) {
      expect(tabAfter(key, 'chat')).toBeNull()
    }
  })
})

describe('workingBadgeName', () => {
  it('names one working helper in the singular', () => {
    expect(workingBadgeName(1)).toBe('1 helper working')
  })

  it('names every other count in the plural', () => {
    expect(workingBadgeName(2)).toBe('2 helpers working')
    expect(workingBadgeName(11)).toBe('11 helpers working')
  })
})
