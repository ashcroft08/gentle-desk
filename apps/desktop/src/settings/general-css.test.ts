// The General pane's rows, as the approved artboard declares them (0.9.0 general
// S2; `design/settings-redesign/08-general.html`, G1/G2 and the five classes its
// own footer names): `.tool`, `.thead`, `.tname`, `.tver`, `.twhy` and `.tcheck`,
// scoped under `.settings-pane` the way every other pane's rules are.
//
// Every assertion reads a DECLARATION, never the rule's raw text: app.css
// explains itself in comments, and a substring search would let the prose satisfy
// the test with the declaration gone. Every colour is the house's token, so the
// six themes paint these rows without a line of their own.
//
// What a stylesheet read cannot prove is how the rows LOOK in a real window: that
// is reported as unverified and judged on the running app.
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

/** Every selector this file pins, as the sheet must spell them. */
const SCOPED = [
  '.settings-pane .tool',
  '.settings-pane .tool + .tool',
  '.settings-pane .tool:first-of-type',
  '.settings-pane .thead',
  '.settings-pane .tname',
  '.settings-pane .tver',
  '.settings-pane .twhy',
  '.settings-pane .twhy code',
  '.settings-pane .tcheck',
  '.settings-pane .tcheck .btn',
]

describe('the General pane rows', () => {
  it('scopes every new class under the pane, so no other page inherits it', () => {
    for (const selector of SCOPED) {
      expect(declarations(selector), selector).not.toBe('')
    }
  })

  // G1: one block per tool, set off by a hairline, and the first one carries no
  // rule above it - the pane's hint already separates it from the heading.
  it('separates the tool rows with one hairline and spares the first', () => {
    expect(declarations('.settings-pane .tool')).toMatch(/width:\s*100%\s*;/)
    expect(declarations('.settings-pane .tool + .tool')).toMatch(
      /border-top:\s*1px\s+solid\s+var\(--border\)\s*;/,
    )
    const first = declarations('.settings-pane .tool:first-of-type')
    expect(first).toMatch(/border-top:\s*0\s*;/)
    expect(first).toMatch(/padding-top:\s*2px\s*;/)
  })

  // G1: the name and the state on one line, the name on the left and the tag on
  // the right whatever the name's length.
  it('draws the name and the state on one row, the tag on its own column', () => {
    expect(declarations('.settings-pane .thead')).toMatch(
      /grid-template-columns:\s*minmax\(0,\s*1fr\)\s+auto\s*;/,
    )
    expect(declarations('.settings-pane .thead')).toMatch(/align-items:\s*baseline\s*;/)
  })

  it('draws the name and the version mono, and the version in the faint ink', () => {
    for (const selector of ['.settings-pane .tname', '.settings-pane .tver']) {
      expect(declarations(selector), selector).toMatch(
        /font-family:\s*ui-monospace,\s*'Cascadia Code',\s*Consolas,\s*monospace\s*;/,
      )
    }
    expect(declarations('.settings-pane .tname')).toMatch(/font-weight:\s*600\s*;/)
    expect(declarations('.settings-pane .tver')).toMatch(/color:\s*var\(--faint\)\s*;/)
  })

  // G2: the fix hint is a sentence of the muted ink, and the command inside it is
  // drawn in `<code>`, so the two never read as the same kind of text.
  it('draws the fix hint quiet, with its command in the code ink', () => {
    expect(declarations('.settings-pane .twhy')).toMatch(/color:\s*var\(--muted\)\s*;/)
    expect(declarations('.settings-pane .twhy')).toMatch(/line-height:\s*1\.55\s*;/)
    const code = declarations('.settings-pane .twhy code')
    expect(code).toMatch(/background:\s*var\(--surface2\)\s*;/)
    expect(code).toMatch(/border-radius:\s*4px\s*;/)
  })

  // G1: the last check and its button close the pane, the button pushed right.
  it('sets the last check off with a hairline and pushes the button right', () => {
    const check = declarations('.settings-pane .tcheck')
    expect(check).toMatch(/border-top:\s*1px\s+solid\s+var\(--border\)\s*;/)
    expect(check).toMatch(/color:\s*var\(--muted\)\s*;/)
    expect(declarations('.settings-pane .tcheck .btn')).toMatch(/margin-left:\s*auto\s*;/)
  })

  // The house rule the whole sheet keeps: a token, never a colour of its own.
  it('writes no colour of its own', () => {
    for (const selector of SCOPED) {
      expect(declarations(selector), selector).not.toMatch(/#[0-9a-fA-F]{3,8}\b/)
    }
  })
})
