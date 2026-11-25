// The two rules at the end of every scrolling Settings pane (settings-redesign
// 05-usage.html / 07-gap-and-rules.html, R2; approved as D5 and D9): 24px of air
// under the content and a 24px veil over the cut, on `.settings-pane` itself, so
// Cuenta, Actualizaciones and Apariencia get them too and not just Uso.
//
// The 24 is the SAME number in the two declarations on purpose: the veil can only
// fall on padding when the scroll is at the end, so nothing is ever left grey -
// the arithmetic below reads both numbers out of the sheet and compares them
// instead of hardcoding either.
//
// Every assertion reads a DECLARATION, never the rule's raw text: app.css explains
// itself in comments, and a substring search would let the prose satisfy the test
// with the declaration gone. What a stylesheet read cannot prove - how the veil
// looks in a real WebView2 window, and whether its scrollbar thumb fading there
// bothers the eye - is reported as unverified.
import { describe, expect, it } from 'vitest'
import APP_CSS from '../app.css?raw'

/** The sheet without its comments: prose never satisfies a selector. */
const SHEET = APP_CSS.replace(/\/\*[\s\S]*?\*\//g, '')

/** One rule's DECLARATIONS, by its exact selector (spacing normalized). */
function declarations(selector: string): string {
  const exact = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&').replace(/ /g, '\\s+')
  const pattern = new RegExp(`(?:^|\\n)\\s*${exact}\\s*\\{`)
  const start = pattern.exec(SHEET)
  if (start === null) throw new Error(`no rule for ${selector}`)
  const open = start.index + start[0].length
  return SHEET.slice(open, SHEET.indexOf('}', open))
}

/** The pane's bottom padding, as its own shorthand declares it. */
function bottomPadding(): number {
  const shorthand = /padding:\s*([^;]+);/.exec(declarations('.settings-pane'))
  expect(shorthand, 'the pane declares its padding as one shorthand').not.toBeNull()
  const parts = shorthand![1].trim().split(/\s+/)
  // `4px 12px 24px 4px`: four values are top / right / bottom / left. The RIGHT
  // edge is real: Windows 11's overlay scrollbar collapses the
  // `scrollbar-gutter` to zero, so without it the content ran under the floating
  // bar (the maintainer's 0.8.2 test: the plan glued to the edge).
  expect(parts, 'the pane names all four edges').toHaveLength(4)
  const bottom = /^(\d+)px$/.exec(parts[2])
  expect(bottom, 'the bottom edge is a px length').not.toBeNull()
  return Number(bottom![1])
}

/** The veil's size, as the gradient's own stop declares it. */
function veilHeight(property: 'mask-image' | '-webkit-mask-image'): number {
  const rule = declarations('.settings-pane')
  const declared = new RegExp(`${property}:\\s*linear-gradient\\(([^;]+)\\)\\s*;`).exec(rule)
  expect(declared, `the pane declares ${property}`).not.toBeNull()
  const stop = /var\(--text\)\s+calc\(100%\s*-\s*(\d+)px\)/.exec(declared![1])
  expect(stop, `${property} fades over the last px of the pane`).not.toBeNull()
  return Number(stop![1])
}

describe('the end of a scrolling Settings pane', () => {
  it('keeps 24px of air under the content, and nothing against the footer', () => {
    expect(bottomPadding()).toBe(24)
    // PRESERVE: the two edges the rail does not already give are unchanged.
    const shorthand = /padding:\s*([^;]+);/.exec(declarations('.settings-pane'))!
    expect(shorthand[1].trim()).toBe('4px 12px 24px 4px')
    // The right edge is a px length the test can read, not a gutter it cannot:
    // an overlay scrollbar makes `scrollbar-gutter` reserve nothing.
    expect(/^12px$/.test(shorthand[1].trim().split(/\s+/)[1] ?? ''), 'the right edge is real air').toBe(true)
  })

  it('fades the last 24px, with the prefixed property WebView2 older than 120 needs', () => {
    expect(veilHeight('mask-image')).toBe(24)
    expect(veilHeight('-webkit-mask-image')).toBe(24)
  })

  it('uses the two numbers as one: the veil is exactly the air under it', () => {
    expect(veilHeight('mask-image')).toBe(bottomPadding())
    expect(veilHeight('-webkit-mask-image')).toBe(bottomPadding())
  })

  it('writes no colour of its own: the veil is a mask over an opaque token', () => {
    const pane = declarations('.settings-pane')
    expect(pane).not.toMatch(/#[0-9a-fA-F]{3,8}\b/)
    expect(pane).toMatch(/var\(--text\)/)
    // A veil only at the bottom: one gradient, fading to transparent at 100%.
    expect(pane).toMatch(/linear-gradient\(\s*to bottom\s*,/)
    expect(pane.match(/linear-gradient\(/g)).toHaveLength(2)
  })
})
