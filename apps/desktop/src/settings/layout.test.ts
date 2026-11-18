// The Settings card grows with its section, and a section can be long: the Usage
// pane draws every provider, every limit and every window, and Updates draws its
// notes. With no bound the card pushed its own footer past the bottom of the
// window, so the section could not be read and the Close button could not be
// reached (ui-density T1, found by the 2026-11-18 audit).
//
// The contract is two rules that only work together: the body is bounded against
// the viewport, and the pane - a flex child - is the thing that scrolls, so it
// needs min-height: 0 to be allowed to shrink below its content.
//
// Both assertions read DECLARATIONS, never the rule's raw text: app.css explains
// itself in comments, so a substring search would let the prose satisfy the test
// with the declaration gone (native review R3-002), and would pass any `100vh`
// even one that leaves the dialog chrome outside the bound (R3-001).
import { describe, expect, it } from 'vitest'
import APP_CSS from '../app.css?raw'

/** One rule's DECLARATIONS, by its exact selector: comments and braces removed. */
function declarations(selector: string): string {
  // The selector is matched with any spacing around it: reformatting the
  // stylesheet must not turn this helper into a generic failure (review R2-1).
  const pattern = new RegExp(`(^|\\n)\\s*${selector.replace('.', '\\.')}\\s*\\{`)
  const start = pattern.exec(APP_CSS)
  if (start === null) throw new Error(`no rule for ${selector}`)
  const open = start.index + start[0].length
  const body = APP_CSS.slice(open, APP_CSS.indexOf('}', open))

  return body.replace(/\/\*[\s\S]*?\*\//g, '')
}

describe('the Settings dialog bounds its own height', () => {
  it('the body leaves room for the dialog chrome above and below', () => {
    // `100vh` alone is NOT the contract: the title and the buttons live outside
    // the body, so the bound has to subtract them - and subtract a NAMED amount,
    // so the coupling is traceable (review R2-2).
    expect(declarations('.settings-body')).toMatch(
      /max-height:\s*calc\(\s*100vh\s*-\s*var\(--settings-chrome\)\s*\)/,
    )
    expect(declarations('.dlg.settings')).toMatch(/--settings-chrome:\s*\d+px\s*;/)
  })

  it('the pane scrolls inside that bound and is allowed to shrink', () => {
    const pane = declarations('.settings-pane')

    expect(pane).toMatch(/overflow-y:\s*auto/)
    expect(pane).toMatch(/min-height:\s*0\s*;/)
  })
})
