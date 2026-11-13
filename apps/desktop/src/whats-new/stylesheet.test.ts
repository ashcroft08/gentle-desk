// The what's-new modal follows the app's theme: its rules may only use the
// theme's CSS variables, never a fixed colour (the user's explicit ask).
import { describe, expect, it } from 'vitest'
import APP_CSS from '../app.css?raw'

/** Every rule whose selector mentions `.whatsnew`, as `selector { body }`. */
function whatsNewRules(): string[] {
  return [...APP_CSS.matchAll(/([^{}]*\.whatsnew[^{}]*)\{([^}]*)\}/g)].map((m) => m[0])
}

describe('the what\u2019s-new modal stylesheet', () => {
  it('has rules, and none carries a fixed colour', () => {
    const rules = whatsNewRules()
    expect(rules.length).toBeGreaterThan(5)
    for (const rule of rules) {
      expect(rule, rule).not.toMatch(/#[0-9a-fA-F]{3,8}\b/)
      expect(rule, rule).not.toMatch(/\b(rgb|rgba|hsl|hsla)\(/)
    }
  })

  it('uses theme variables for its surface, border and accent', () => {
    const css = whatsNewRules().join('\n')
    for (const name of ['--border', '--muted', '--faint', '--accent', '--tint', '--text']) {
      expect(css).toContain(`var(${name})`)
    }
  })
})
