// The three rules of the Chat | Helpers | Profile tablist, one test per rule: the
// arrow keys walk the list and wrap, Home and End jump to its ends, and every other
// key keeps its own meaning. The tab words and the badge's name are the header's
// own copy, so they are pinned in `copy.test.ts` beside the dictionary that holds
// them (0.2.0, S3, D5).

import { describe, expect, it } from 'vitest'
import { TABS, tabAfter } from './tabs'

describe('tabAfter', () => {
  it('walks the list with the arrow keys, wrapping at both ends', () => {
    expect(tabAfter('ArrowRight', 'chat')).toBe('helpers')
    expect(tabAfter('ArrowRight', 'helpers')).toBe('profile')
    expect(tabAfter('ArrowRight', 'profile')).toBe('chat')
    expect(tabAfter('ArrowLeft', 'chat')).toBe('profile')
    expect(tabAfter('ArrowLeft', 'profile')).toBe('helpers')
    expect(tabAfter('ArrowDown', 'chat')).toBe('helpers')
    expect(tabAfter('ArrowUp', 'chat')).toBe('profile')
  })

  it('jumps to the first and last tab with Home and End', () => {
    expect(tabAfter('Home', 'profile')).toBe(TABS[0])
    expect(tabAfter('End', 'chat')).toBe(TABS[TABS.length - 1])
    expect(TABS[TABS.length - 1]).toBe('profile')
  })

  it('leaves every other key alone', () => {
    for (const key of ['Enter', 'Tab', 'Escape', 'ArrowRightish', 'a']) {
      expect(tabAfter(key, 'chat')).toBeNull()
    }
  })
})
