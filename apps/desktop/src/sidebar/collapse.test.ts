// The sidebar collapse rule (responsive-sidebar S2, S3) as plain data: the
// window's width decides unless the user has toggled the sidebar by hand, and
// a manual toggle wins for the whole run.
import { describe, expect, it } from 'vitest'
import { SIDEBAR_AUTO_COLLAPSE_BELOW, collapsedOf, overlayOf } from './collapse'

describe('the sidebar collapse rule', () => {
  it('collapses a narrow window by default', () => {
    expect(collapsedOf(null, 690)).toBe(true)
    expect(collapsedOf(null, 100)).toBe(true)
  })

  it('expands a wide window by default', () => {
    expect(collapsedOf(null, 1024)).toBe(false)
    expect(collapsedOf(null, SIDEBAR_AUTO_COLLAPSE_BELOW)).toBe(false)
  })

  it('leaves the width unknown expanded rather than guessing', () => {
    expect(collapsedOf(null, null)).toBe(false)
  })

  it('a manual toggle wins over the width for the whole run', () => {
    // Expanded by hand on a narrow window: the resize never re-collapses it.
    expect(collapsedOf(false, 690)).toBe(false)
    // Collapsed by hand on a wide window: it stays collapsed.
    expect(collapsedOf(true, 1024)).toBe(true)
  })
})

describe('the sidebar overlay rule (narrow windows)', () => {
  it('an expanded sidebar on a narrow window overlays the chat instead of squeezing it', () => {
    expect(overlayOf(false, 690)).toBe(true)
    expect(overlayOf(false, SIDEBAR_AUTO_COLLAPSE_BELOW - 1)).toBe(true)
  })

  it('on a wide window the sidebar pushes the chat as a column, never overlays', () => {
    expect(overlayOf(false, 1024)).toBe(false)
    expect(overlayOf(false, SIDEBAR_AUTO_COLLAPSE_BELOW)).toBe(false)
  })

  it('a collapsed sidebar overlays nothing, at any width', () => {
    expect(overlayOf(true, 690)).toBe(false)
    expect(overlayOf(true, 1024)).toBe(false)
  })

  it('an unmeasured width never overlays', () => {
    expect(overlayOf(false, null)).toBe(false)
  })
})
