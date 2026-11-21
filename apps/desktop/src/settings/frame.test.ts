// The approved Settings frame (design/settings-polish/02-frame.html, P1/P3/P4/P5/P6
// and D3/D4): seven one-declaration fixes read off the real dialog and approved on
// the canvas. this file pins them where a stylesheet read can prove them - the
// declaration itself, never the rule's raw text, since app.css explains itself in
// comments and a substring search would let the prose satisfy the test.
//
// What a stylesheet read cannot prove: that the result LOOKS like the artboard in
// a real window. That is reported as unverified.
import { describe, expect, it } from 'vitest'
import APP_CSS from '../app.css?raw'

/** One rule's DECLARATIONS, by its exact selector: comments removed. */
function declarations(selector: string): string {
  const exact = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const pattern = new RegExp(`(?:^|\\n)\\s*${exact}\\s*\\{`)
  const start = pattern.exec(APP_CSS)
  if (start === null) throw new Error(`no rule for ${selector}`)
  const open = start.index + start[0].length
  const body = APP_CSS.slice(open, APP_CSS.indexOf('}', open))
  return body.replace(/\/\*[\s\S]*?\*\//g, '')
}

/** Every `@media` block's query and body, in the order the sheet declares them. */
function mediaBlocks(): { query: string; body: string }[] {
  const blocks: { query: string; body: string }[] = []
  const marker = /@media[^{]*/g
  let found = marker.exec(APP_CSS)
  while (found !== null) {
    const open = APP_CSS.indexOf('{', found.index)
    let depth = 0
    let end = open
    for (let index = open; index < APP_CSS.length; index += 1) {
      if (APP_CSS[index] === '{') depth += 1
      if (APP_CSS[index] === '}') {
        depth -= 1
        if (depth === 0) {
          end = index
          break
        }
      }
    }
    blocks.push({
      query: found[0].trim(),
      body: APP_CSS.slice(open + 1, end).replace(/\/\*[\s\S]*?\*\//g, ''),
    })
    marker.lastIndex = end
    found = marker.exec(APP_CSS)
  }
  return blocks
}

describe('the Settings frame the maintainer approved', () => {
  // P1 · the search field: the glyph's gap and the wrapper's padding move out to
  // the rail's own column, and the input stops inheriting `.dlg input`'s 7px 10px
  // - the 10px left padding put the placeholder's first letter 39px in and made a
  // 32.75px input overflow its 28px box.
  it('sets the search field on the rail glyph column and repads its input', () => {
    const find = declarations('.settings-find')
    expect(find).toMatch(/gap:\s*8px\s*;/)
    expect(find).toMatch(/padding:\s*0\s+9px\s*;/)

    expect(declarations('.settings-find input')).toMatch(/padding:\s*0\s*;/)
  })

  // P4 · one rail entry: the house's default 8 between a glyph and its word.
  it('puts the rail glyph and word 8px apart', () => {
    expect(declarations(".settings-rail [role='tab']")).toMatch(/gap:\s*8px\s*;/)
  })

  // D4/P5 · the rail column's gap, and the pane's symmetric padding plus the
  // scrollbar channel reserved in all eight sections. S7 raised both by the one
  // step the sheet already uses: 12 -> 14 between field and rail, 2 -> 4 at the
  // pane's two edges.
  it('separates the field from the rail and keeps the pane symmetric and gutter-stable', () => {
    expect(declarations('.settings-side')).toMatch(/gap:\s*14px\s*;/)

    const pane = declarations('.settings-pane')
    expect(pane).toMatch(/padding:\s*4px\s+0\s+0\s+4px\s*;/)
    expect(pane).toMatch(/scrollbar-gutter:\s*stable\s*;/)
  })

  // P6 · the Close row: the card's own 14px gap is the separation, not 14 + 16.
  it('drops the Close row margin in the Settings card', () => {
    expect(declarations('.dlg.settings .acts')).toMatch(/margin-top:\s*0\s*;/)
  })

  // P3 · the label class the Appearance grid's word is drawn in: `.mode-label`'s
  // ink without its bottom border.
  it('draws the pane label in the mode label ink, without a border', () => {
    const label = declarations('.pane-label')
    expect(label).toMatch(/color:\s*var\(--muted\)\s*;/)
    expect(label).toMatch(/font-size:\s*12px\s*;/)
    expect(label).toMatch(/font-weight:\s*600\s*;/)
    expect(label).not.toMatch(/border/)
    expect(label).toMatch(/margin:\s*0\s+0\s+8px\s*;/)
  })

  // D3/P3 · three columns whatever the window says: the card is at least 752px
  // wide at every window the app allows (860 from 1280 up, S7), so a viewport
  // media query only ever described a range the grid could not really be narrow
  // in.
  it('keeps the cards three to a row with no viewport query to override it', () => {
    expect(declarations('.tc-grid')).toMatch(
      /grid-template-columns:\s*repeat\(3,\s*minmax\(0,\s*1fr\)\)/,
    )
    for (const block of mediaBlocks()) {
      expect(block.body, block.query).not.toMatch(/\.tc-grid/)
    }
  })

  // D3 · the dead declaration: the rail is always taller than 240px, so this
  // never applied. The bound against the window stays.
  it("keeps the body's window bound and drops the dead min-height", () => {
    const body = declarations('.settings-body')
    expect(body).toMatch(/max-height:\s*calc\(100vh\s*-\s*var\(--settings-chrome\)\)/)
    expect(body).not.toMatch(/min-height/)
  })

  // PRESERVE: the Close-row margin is only dropped in the Settings card. Every
  // other dialog still sets its own footer off the content it closes.
  it("leaves the other dialogs' Close row margin alone", () => {
    expect(declarations('.dlg .acts')).toMatch(/margin-top:\s*16px\s*;/)
  })
})
