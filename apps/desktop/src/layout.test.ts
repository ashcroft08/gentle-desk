// The width of the conversation: the thread and the box under it share one
// maximum, so a wide window is used instead of leaving a narrow column in the
// middle with empty sides, and the box lines up with the messages. The
// stylesheet comes in as `?raw` text (vitest.config.ts lists it).
import { describe, expect, it } from 'vitest'
import APP_CSS from './app.css?raw'

/** One rule's body, by its exact selector. */
function ruleBody(selector: string): string {
  const start = APP_CSS.indexOf(`\n${selector} {`)
  if (start < 0) throw new Error(`no rule for ${selector}`)
  return APP_CSS.slice(start, APP_CSS.indexOf('}', start))
}

describe('the sidebar collapse (responsive-sidebar S4)', () => {
  // A grid column must be minmax(0, 1fr): a bare `1fr` cannot shrink below its
  // content's min-content, so one nowrap title or code line pushed the whole
  // chat past the window's right edge (the user's narrow-window screenshot).
  it('the conversation column can shrink below its content', () => {
    expect(ruleBody('.app')).toContain('280px minmax(0, 1fr)')
    expect(ruleBody('.main')).toContain('min-width: 0')
  })

  // With the sidebar `display: none` the main area is the grid's FIRST item, so
  // a leftover 0-width first column crushed it to nothing (the collapsed
  // screenshot). The collapsed grid is one column that takes the whole width.
  it('the collapsed layout is one full-width column and the sidebar leaves it', () => {
    const collapsed = ruleBody('.app.scollapsed')
    expect(collapsed).toContain('grid-template-columns: minmax(0, 1fr)')
    expect(collapsed).not.toMatch(/grid-template-columns:\s*0/)
    expect(ruleBody('.app.scollapsed .side')).toContain('display: none')
  })

  it('on a narrow window an open sidebar overlays the chat, one full-width column', () => {
    expect(ruleBody('.app.soverlay')).toContain('grid-template-columns: minmax(0, 1fr)')
    expect(ruleBody('.app.soverlay .side')).toContain('position: absolute')
    expect(ruleBody('.app.soverlay .scrim')).toContain('position: absolute')
  })

  it('the toggle control exists as a real button rule', () => {
    expect(ruleBody('.side-toggle')).toContain('cursor: pointer')
  })
})

describe('the conversation width', () => {  it('keeps the line in the readable band instead of filling any window', () => {
    const declared = /--chat-max:\s*(\d+)px/.exec(APP_CSS)

    expect(declared).not.toBeNull()
    // About 60 to 80 characters at this font size: the band reading research
    // agrees on. Below it the text is cramped, above it the eye loses its place.
    expect(Number(declared?.[1])).toBeGreaterThanOrEqual(700)
    expect(Number(declared?.[1])).toBeLessThanOrEqual(1000)
  })

  it('applies it to the thread and to the box, not a narrow number of their own', () => {
    expect(ruleBody('.thread-in')).toContain('max-width: var(--chat-max)')
    expect(ruleBody('.composer-dock')).toContain('var(--chat-max)')
    expect(ruleBody('.thread-in')).not.toMatch(/max-width:\s*7\d\dpx/)
    expect(ruleBody('.composer-dock')).not.toMatch(/7\d\dpx/)
  })
})
