// The two pure composer rules, one test per rule: Enter sends only a plain
// Enter (never with Shift, never during an IME composition), and the box grows
// with what was typed up to the artboard's maximum height.

import { describe, expect, it } from 'vitest'
import { MAX_HEIGHT } from './copy'
import { fittedHeight, shouldSend, type KeyLike } from './keys'

function key(over: Partial<KeyLike> = {}): KeyLike {
  return { key: 'Enter', shiftKey: false, isComposing: false, keyCode: 13, ...over }
}

describe('shouldSend', () => {
  it('sends on a plain Enter', () => {
    expect(shouldSend(key())).toBe(true)
  })

  it('leaves Shift+Enter to the textarea, so it inserts a newline', () => {
    expect(shouldSend(key({ shiftKey: true }))).toBe(false)
  })

  it('leaves an Enter that commits an IME composition to the candidate list', () => {
    expect(shouldSend(key({ isComposing: true }))).toBe(false)
    // The legacy signal a browser sends instead of `isComposing`.
    expect(shouldSend(key({ keyCode: 229 }))).toBe(false)
  })

  it('ignores every other key', () => {
    expect(shouldSend(key({ key: 'a' }))).toBe(false)
    expect(shouldSend(key({ key: 'Escape' }))).toBe(false)
  })
})

describe('fittedHeight', () => {
  it('follows the content until the maximum', () => {
    expect(fittedHeight(24)).toBe(24)
    expect(fittedHeight(MAX_HEIGHT - 1)).toBe(MAX_HEIGHT - 1)
  })

  it('stops at the artboard maximum, where the box scrolls instead', () => {
    expect(fittedHeight(MAX_HEIGHT + 200)).toBe(MAX_HEIGHT)
  })

  it('never goes negative, whatever a browser reports', () => {
    expect(fittedHeight(-5)).toBe(0)
    expect(fittedHeight(Number.NaN)).toBe(0)
  })
})
