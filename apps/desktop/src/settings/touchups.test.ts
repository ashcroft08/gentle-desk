// The Settings dialog's touch-ups (odd/tasks/settings-redesign.md S7, T3; L4):
// the maintainer saw three full redesigns and asked for the one that exists with
// a few improvements - "mira el que tenemos y dale unos toques para mejorarlo no
// mas". So the dialog, the rail, the search and the panes keep their shape, and
// what changed is measured here: a card that follows the window, a body with a
// height of its own, more air, and no box where the box was not a unit.
//
// Every assertion reads a DECLARATION out of app.css - the durable record of the
// touch-ups - never the sheet's raw text, which explains itself in comments and
// would satisfy a substring search with the declaration gone (native review
// R3-002). The no-overflow case is arithmetic over the numbers those declarations
// name, never a hardcoded result: change one and the fit is recomputed with it.
//
// What a stylesheet read cannot prove is how the result LOOKS in a real window:
// the parent builds the app and the maintainer judges it there, so it is reported
// as unverified rather than pinned here.
import { describe, expect, it } from 'vitest'
import APP_CSS from '../app.css?raw'

/** The sheet without its comments: a selector list is never prose. */
const SHEET = APP_CSS.replace(/\/\*[\s\S]*?\*\//g, '')

/**
 * One rule's DECLARATIONS, by one selector of its comma-separated list, so a
 * grouped rule (`.card.updates, .card.ext`) answers for each of its selectors.
 */
function declarations(selector: string): string {
  const pattern = /([^{}]+)\{([^{}]*)\}/g
  let found = pattern.exec(SHEET)
  while (found !== null) {
    const list = found[1].split(',').map((part) => part.trim())
    if (list.includes(selector)) return found[2]
    found = pattern.exec(SHEET)
  }
  throw new Error(`no rule for ${selector}`)
}

/** The window the app can be shrunk to (`src-tauri/tauri.conf.json`). */
const MIN_WINDOW = { width: 800, height: 560 }
/** The margin the card keeps at each side of the window. */
const MIN_MARGIN = 48

/** The card's width at a window width, from the rule's own two numbers. */
function cardWidth(viewWidth: number): number {
  const pinned = /width:\s*min\((\d+)px,\s*calc\(100vw\s*-\s*(\d+)px\)\)\s*;/.exec(
    declarations('.dlg.settings'),
  )
  expect(pinned, 'the card names the widest it goes and the margin it keeps').not.toBeNull()
  return Math.min(Number(pinned![1]), viewWidth - Number(pinned![2]))
}

/** What the card's chrome takes out of the window, as the card itself names it. */
function chrome(): number {
  const pinned = /--settings-chrome:\s*(\d+)px\s*;/.exec(declarations('.dlg.settings'))
  expect(pinned, 'the card names the chrome it subtracts').not.toBeNull()
  return Number(pinned![1])
}

/** The body's height at a window height: the clamp, then the window's own bound. */
function bodyHeight(viewHeight: number): number {
  const pinned = /height:\s*clamp\((\d+)px,\s*([\d.]+)vh,\s*(\d+)px\)\s*;/.exec(
    declarations('.settings-body'),
  )
  expect(pinned, 'the body names its floor, its share of the window and its ceiling').not.toBeNull()
  const floor = Number(pinned![1])
  const share = Number(pinned![2]) / 100
  const ceiling = Number(pinned![3])
  const asked = Math.min(ceiling, Math.max(floor, share * viewHeight))
  return Math.min(asked, viewHeight - chrome())
}

describe('the Settings dialog’s touch-ups', () => {
  // S7 · "lo veo pequeño": the card gives up the fixed 680 and takes the window,
  // keeping 24px at each side - 752 wide at the 800x560 minimum, 860 from 1280 up.
  it('sizes the card after the window, up to 860px and never past it', () => {
    const card = declarations('.dlg.settings')

    expect(card).toMatch(/width:\s*min\(860px,\s*calc\(100vw\s*-\s*48px\)\)\s*;/)
    expect(cardWidth(1280)).toBe(860)
    expect(cardWidth(1920)).toBe(860)
    expect(cardWidth(MIN_WINDOW.width)).toBe(MIN_WINDOW.width - MIN_MARGIN)
  })

  // S7 · the body used to be whatever the rail made it (about 310). It follows the
  // window now: 380 in the shortest one the app allows, 480 at 800 tall, 560 from
  // 933 up, with the pane - not the body - as what scrolls inside.
  it('gives the body a height of its own that follows the window', () => {
    const body = declarations('.settings-body')

    expect(body).toMatch(/height:\s*clamp\(380px,\s*60vh,\s*560px\)\s*;/)
    expect(bodyHeight(MIN_WINDOW.height)).toBe(380)
    expect(bodyHeight(800)).toBe(480)
    expect(bodyHeight(1080)).toBe(560)
    expect(declarations('.settings-pane')).toMatch(/overflow-y:\s*auto\s*;/)
  })

  // THE no-overflow case: at the window the app can be shrunk to, the card these
  // declarations describe - the body plus the chrome it subtracts - is inside 560,
  // and its width is inside 800 with the margins it names. Computed from the
  // numbers in the sheet, so a bigger clamp fails here before the app does.
  it('fits the 800x560 minimum window without overflowing it', () => {
    expect(MIN_WINDOW.width - cardWidth(MIN_WINDOW.width)).toBeGreaterThanOrEqual(MIN_MARGIN)
    expect(bodyHeight(MIN_WINDOW.height) + chrome()).toBeLessThanOrEqual(MIN_WINDOW.height)
  })

  it('fits the larger windows with room to spare', () => {
    for (const [width, height] of [
      [1280, 800],
      [1920, 1080],
    ] as const) {
      expect(cardWidth(width)).toBeLessThanOrEqual(width - MIN_MARGIN)
      expect(bodyHeight(height) + chrome()).toBeLessThanOrEqual(height)
    }
  })

  // S7 · "todo esta apretado": one rhythm, in the steps the sheet already uses.
  it('adds air in one rhythm the sheet already uses', () => {
    const card = declarations('.dlg.settings')

    expect(card).toMatch(/padding:\s*20px 24px\s*;/)
    expect(card).toMatch(/gap:\s*16px\s*;/)
    expect(declarations('.settings-body')).toMatch(/gap:\s*20px\s*;/)
    expect(declarations('.settings-side')).toMatch(/gap:\s*14px\s*;/)
    // The rail's rows are a touch taller than the 32 they were.
    expect(declarations(".settings-rail [role='tab']")).toMatch(/height:\s*36px\s*;/)
    // heading -> hint -> content, one step apart.
    expect(declarations('.settings-pane h3')).toMatch(/margin:\s*0 0 6px\s*;/)
    expect(declarations('.settings-pane .hint')).toMatch(/margin:\s*0 0 18px\s*;/)
  })

  // S7 · "mucho uso de cards": the box goes wherever the box was not a unit. The
  // Updates groups and the Account providers become runs of the pane; the hairline
  // and the air say where one thing ends and the next begins.
  it('draws the Updates groups and the Account providers with no box at all', () => {
    for (const selector of [
      '.settings-pane .card.updates',
      '.settings-pane .card.ext',
      '.settings-pane .providers',
    ]) {
      const rule = declarations(selector)
      // `box-sizing: border-box` is not a border: only a real border declaration is.
      expect(rule, selector).not.toMatch(/(^|[\s;])border(-[a-z]+)?\s*:/)
      expect(rule, selector).not.toMatch(/(^|[\s;])background(-[a-z]+)?\s*:/)
    }
  })

  it('separates those rows with one hairline instead of a box each', () => {
    for (const selector of [
      '.settings-pane .urow + .urow',
      '.settings-pane .provider + .provider',
      '.settings-pane .card.updates + .card.ext',
    ]) {
      expect(declarations(selector), selector).toMatch(
        /border-top:\s*1px solid var\(--border\)\s*;/,
      )
    }
  })

  // PRESERVE: the theme previews are the card S2 keeps - six of them, still three
  // to a row. The OTHER card this test used to keep - one Usage provider - is gone
  // by the maintainer's own ruling on that pane (settings-redesign L5, S8): a
  // provider is a section of the whole column now, separated by a hairline, and
  // `UsagePane.test.ts` pins that instead. The theme previews are not touched.
  it('keeps a card where a card is a unit, the theme previews', () => {
    expect(declarations('.tc')).toMatch(/border:\s*1px solid var\(--border\)\s*;/)
    expect(declarations('.tc-grid')).toMatch(
      /grid-template-columns:\s*repeat\(3,\s*minmax\(0,\s*1fr\)\)\s*;/,
    )
  })

  // S7 · the tiles take the card's new width: the miniature is a step bigger, so a
  // preview reads as a picture of a window and not as a thumbnail.
  it('grows the theme miniature with the wider card', () => {
    const shot = declarations('.tc-shot')

    expect(shot).toMatch(/padding:\s*8px\s*;/)
    expect(shot).toMatch(/font-size:\s*10px\s*;/)
    // PRESERVE: the miniature is still painted on the theme's own background.
    expect(shot).toMatch(/background:\s*var\(--bg\)\s*;/)
  })
})
