// View A's geometry, pinned where the app already keeps it: in the stylesheet
// (welcome-screen T2's follow-up). The maintainer looked at the screen in a
// MAXIMIZED window and asked for spacing and size that hold at every window size,
// so the rules below are read as text - the way Composer.dock.test.ts pins the
// dock - because what they decide is layout, which jsdom does not compute.
//
// What a text assertion can prove: the declarations exist, in the rule that owns
// them, for the window each one is meant for. What it CANNOT prove: that the
// result looks right - that the block lands at 40-45% of a maximized window, that
// nothing overlaps at the minimum size, that a real rose at 136 px breathes. Those
// need a real window, and they are reported as unverified.
import { describe, expect, it } from 'vitest'
import APP_CSS from '../app.css?raw'

/** The body of the first rule whose selector is exactly `selector`. */
function rule(selector: string): string {
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const match = APP_CSS.match(new RegExp(`(?:^|[}\\n])\\s*${escaped}\\s*\\{([^}]*)\\}`))
  return match?.[1] ?? ''
}

/** The rules inside `@media (query) { ... }`, with their braces balanced. */
function media(query: string): string {
  const at = APP_CSS.indexOf(`@media (${query})`)
  if (at < 0) return ''
  const open = APP_CSS.indexOf('{', at)
  let depth = 0
  for (let index = open; index < APP_CSS.length; index += 1) {
    if (APP_CSS[index] === '{') depth += 1
    if (APP_CSS[index] === '}') {
      depth -= 1
      if (depth === 0) return APP_CSS.slice(open + 1, index)
    }
  }
  return ''
}

/** One rule's body INSIDE a media query, read the way `rule` reads a loose one. */
function scoped(query: string, selector: string): string {
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const match = media(query).match(new RegExp(`(?:^|[}\\n])\\s*${escaped}\\s*\\{([^}]*)\\}`))
  return match?.[1] ?? ''
}

/** Does `body` declare `property`, whatever its value? */
function declares(body: string, property: string): boolean {
  return new RegExp(`(?:^|[;{\\s])${property}\\s*:`).test(body)
}

describe('the welcome fills its pane and sits above the middle of a tall one', () => {
  it('is a centred column that scrolls its own overflow', () => {
    const home = rule('.home')

    expect(home).toContain('display: flex')
    expect(home).toContain('flex-direction: column')
    expect(home).toContain('align-items: center')
    // `safe` keeps the centre for the ordinary case and falls back to the start
    // when the content is taller than the pane, so the top is never unreachable.
    expect(home).toContain('justify-content: safe center')
    expect(home).toContain('overflow-y: auto')
  })

  it('keeps a bigger bottom padding than its top one, both scaled to the height', () => {
    const home = rule('.home')

    // 24 px at the smallest window, 8vh while the window can afford it, 96 px
    // capped: the block's centre lands a little above the pane's own.
    expect(home).toContain('clamp(24px, 8vh, 96px)')
    expect(home).toMatch(/padding:[^;]*clamp\(24px, 8vh, 96px\)/)
  })

  it('gives the rose air before the title', () => {
    expect(rule('.home-rose')).toMatch(/margin:[^;]*30px/)
    expect(rule('.home-title')).toMatch(/margin:\s*0\s+0\s+6px/)
  })

  it('sizes the title with the window, between the two steps', () => {
    expect(rule('.home-title')).toContain('clamp(18px, 2.2vh + 10px, 24px)')
  })

  it('never lets the rose paint outside the pane', () => {
    expect(rule('.home-rose canvas')).toContain('max-width: 100%')
  })
})

describe('a short window does not centre', () => {
  it('starts the block at the top, with tighter paddings', () => {
    const home = scoped('max-height: 600px', '.home')

    expect(home).toContain('justify-content: flex-start')
    expect(declares(home, 'padding')).toBe(true)
  })

  it('still scrolls what does not fit, so nothing is hidden', () => {
    expect(scoped('max-height: 600px', '.home')).toContain('overflow-y: auto')
  })
})

describe('the recents column and its left edge', () => {
  it('is 400 px wide at most, and the pane at its narrowest', () => {
    expect(rule('.home-recents')).toContain('width: min(400px, 100%)')
  })

  it('starts its heading on the same left edge as its rows', () => {
    // The row's text sits 10 px inside it; the heading therefore does too.
    expect(rule('.home-chat')).toMatch(/padding:\s*7px\s+10px/)
    expect(rule('.home-kicker')).toContain('padding: 0 10px')
  })

  it('keeps the hero centred while the column is left-aligned', () => {
    expect(rule('.home')).toContain('align-items: center')
    expect(rule('.home-recents')).toContain('text-align: left')
  })
})

describe('a narrow window', () => {
  it('gives the column the whole width, with 16 px at the sides', () => {
    expect(scoped('max-width: 640px', '.home')).toContain('padding-inline: 16px')
    expect(scoped('max-width: 640px', '.home-recents')).toContain('width: 100%')
  })

  it('stretches the one action to the column', () => {
    const action = scoped('max-width: 640px', '.home-go')

    expect(action).toContain('width: 100%')
    expect(action).toContain('justify-content: center')
  })
})

describe('a very long chat title', () => {
  it('ellipsizes inside its cell instead of growing the row', () => {
    const cell = rule('.home-chat-title')

    expect(cell).toContain('min-width: 0')
    expect(cell).toContain('overflow: hidden')
    expect(cell).toContain('text-overflow: ellipsis')
    expect(cell).toContain('white-space: nowrap')
  })

  it('never pushes the age out of the row', () => {
    const age = rule('.home-chat-age')

    expect(age).toContain('flex: none')
    expect(age).toContain('white-space: nowrap')
  })
})
