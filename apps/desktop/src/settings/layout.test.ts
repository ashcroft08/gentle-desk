// The Settings card grows with its section, and a section can be long: the Usage
// pane draws every provider, every limit and every window, and Updates draws its
// notes. With no bound the card pushed its own footer past the bottom of the
// window, so the section could not be read and the Close button could not be
// reached (ui-density T1, found by the 2026-11-18 audit).
//
// The contract is two rules that only work together: the body is bounded against
// the viewport, and the pane - a flex child - is the thing that scrolls, so it
// needs min-height: 0 to be allowed to shrink below its content.
import { describe, expect, it } from 'vitest'
import APP_CSS from '../app.css?raw'

/** One rule's body, by its exact selector. */
function ruleBody(selector: string): string {
  const start = APP_CSS.indexOf(`\n${selector} {`)
  if (start < 0) throw new Error(`no rule for ${selector}`)
  return APP_CSS.slice(start, APP_CSS.indexOf('}', start))
}

describe('the Settings dialog bounds its own height', () => {
  it('the body cannot grow past the window', () => {
    expect(ruleBody('.settings-body')).toMatch(/max-height:.*100vh/)
  })

  it('the pane scrolls inside that bound and is allowed to shrink', () => {
    const pane = ruleBody('.settings-pane')

    expect(pane).toContain('overflow-y: auto')
    expect(pane).toContain('min-height: 0')
  })
})
