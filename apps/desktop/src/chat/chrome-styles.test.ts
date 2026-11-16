// The chat chrome's own rules (S1, S2), read as TEXT the way
// `whats-new/stylesheet.test.ts` reads them: jsdom lays nothing out, and a rule
// the app states once is a decision to pin, not a drawing to measure. What the
// components DREW is pinned next door (`mark-pulse.test.ts`, `Header.test.ts`).
//
// 01a is the source of every rule below: the Working mark's 1.2s pulse (killed
// by reduced motion), the spacer that pins the Chat | Helpers switch to the right
// edge, and the narrow-window steps the header takes at 900px.

import { describe, expect, it } from 'vitest'
import APP_CSS from '../app.css?raw'

/** The body of the first rule whose selector is exactly `selector`. */
function rule(selector: string): string {
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const match = APP_CSS.match(new RegExp(`(?:^|[}\\n])\\s*${escaped}\\s*\\{([^}]*)\\}`))
  return match?.[1] ?? ''
}

/** The body of a block that holds rules of its own, matched by its braces. */
function block(at: string): string {
  const start = APP_CSS.indexOf(at)
  if (start < 0) return ''
  const open = APP_CSS.indexOf('{', start)
  let depth = 0
  for (let index = open; index < APP_CSS.length; index += 1) {
    if (APP_CSS[index] === '{') depth += 1
    else if (APP_CSS[index] === '}') {
      depth -= 1
      if (depth === 0) return APP_CSS.slice(open + 1, index)
    }
  }
  return ''
}

describe('the Working pulse (S1)', () => {
  it('animates the Working mark on the class the components draw', () => {
    const working = rule('.mark.m-working')

    expect(working).toContain('animation: pulse 1.2s ease-in-out infinite')
  })

  it('breathes to 35% opacity at the halfway point', () => {
    const pulse = block('@keyframes pulse')

    expect(pulse).toContain('50%')
    expect(pulse).toContain('opacity: 0.35')
  })

  it('stops for a reader who asked for reduced motion', () => {
    const reduced = block('@media (prefers-reduced-motion: reduce)')

    expect(reduced).not.toBe('')
    expect(reduced).toMatch(/\.mark\.m-working\s*\{[^}]*animation:\s*none/)
  })
})

describe('the header row (S2)', () => {
  it('pins the switch to the right edge with the one flexible spacer', () => {
    const spacer = rule('.mh .sp')

    expect(spacer).toContain('flex: 1')
    // The switch itself never grows or shrinks: the title is what gives way.
    expect(rule('.mh .seg')).toContain('flex: none')
  })

  it('keeps the row on one line, with the title free to truncate', () => {
    expect(rule('.mh')).toContain('flex-wrap: nowrap')
    expect(rule('.mh h1')).toContain('text-overflow: ellipsis')
    expect(rule('.mh h1')).toContain('min-width: 0')
  })

  it('tightens the row on a narrow window instead of crushing the switch', () => {
    const narrow = block('@media (max-width: 900px)')

    expect(narrow).toMatch(/\.mh\s*\{[^}]*height:\s*48px/)
    expect(narrow).toMatch(/\.mh\s*\{[^}]*padding:\s*0 12px/)
    expect(narrow).toMatch(/\.mh h1\s*\{[^}]*font-size:\s*14px/)
    expect(narrow).toMatch(/\.mh \.seg \[role='tab'\]\s*\{[^}]*padding:\s*4px 10px/)
  })

  it('gives the inline rename the title slot', () => {
    expect(rule('.mh .rn')).toContain('flex: 1')
    expect(rule('.mh .rn')).toContain('min-width: 0')
  })
})
