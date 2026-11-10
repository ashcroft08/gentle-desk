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

describe('the conversation width', () => {
  it('keeps the line in the readable band instead of filling any window', () => {
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
