// The what's-new modal follows the app's theme: its rules may only use the
// theme's CSS variables, never a fixed colour (the user's explicit ask). The
// language tabs' folder-tab shape is a rule-block pin here (design/whats-new-modal
// 04); what the view DRAWS is pinned in `language-tabs.test.ts`.
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

  // The folder tab (design/whats-new-modal, artboard D): the row keeps its
  // separator line, and the ACTIVE tab is a surface-coloured block with the
  // border on its top and sides, its bottom open and sitting 1px lower, so it
  // covers that line under itself and reads as one component with the notes.
  it('pins the folder-tab anatomy of the language tabs', () => {
    const css = whatsNewRules().join('\n')
    const rule = (selector: string): string =>
      css.match(new RegExp(`[^{}]*${selector}[^{}]*\\{[^}]*\\}`))?.[0] ?? ''

    const row = rule("\\.tabs\\s")
    expect(row).toContain('display: flex')
    expect(row).toContain('border-bottom: 1px solid var(--border)')

    const tab = rule("\\.tabs \\[role='tab'\\]\\s")
    expect(tab).toContain('border-radius: 8px 8px 0 0')
    expect(tab).toContain('margin-bottom: -1px')
    expect(tab).toContain('border-bottom: 0')
    expect(tab).toContain('color: var(--muted)')

    const selected = rule("\\.tabs \\[role='tab'\\]\\[aria-selected='true'\\]")
    expect(selected).toContain('background: var(--surface)')
    expect(selected).toContain('border-color: var(--border)')
    expect(selected).toContain('border-bottom: 0')
    expect(selected).toContain('color: var(--accent)')
    expect(selected).toContain('font-weight: 600')

    // Hover is the text and nothing else, so the row never lights up per tab.
    const hover = rule("\\.tabs \\[role='tab'\\]:hover")
    expect(hover).toContain('color: var(--text)')
    expect(hover).not.toContain('background')
  })
})
