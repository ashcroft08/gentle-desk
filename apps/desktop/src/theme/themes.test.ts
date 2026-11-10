// The six theme names, checked against the two places that must agree with them:
// the stylesheet that paints them and the app's own rules. A drift here is a
// window that keeps its old colors after the user picks a theme, so the names
// are pinned as text instead of trusted. The stylesheets are read as files: a
// `?raw` import of a stylesheet is empty under Vitest, and an empty stylesheet
// would pass every check below.
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { THEMES } from './themes'

/** A stylesheet of `src/`, read from the app's own folder. */
function stylesheet(name: string): string {
  return readFileSync(resolve('src', name), 'utf8')
}

const THEME_CSS = stylesheet('theme.css')
const APP_CSS = stylesheet('app.css')

/** The `[data-theme='<name>']` blocks a stylesheet declares. */
function dataThemeNames(css: string): string[] {
  const names = new Set<string>()
  for (const match of css.matchAll(/\[data-theme='([^']+)'\]/g)) names.add(match[1])
  return [...names]
}

/** One rule's body, by its exact selector, from a stylesheet. */
function ruleBody(css: string, selector: string): string {
  const start = css.indexOf(`${selector} {`)
  if (start < 0) throw new Error(`no rule for ${selector}`)
  const end = css.indexOf('}', start)
  return css.slice(start, end)
}

describe('the six theme names', () => {
  it('are the picker order the artboard draws', () => {
    expect(THEMES).toEqual([
      'system',
      'dark',
      'Gentle',
      'Gentleman-Cute',
      'Gentleman-Sexy',
      'light',
    ])
  })

  it('are the six data-theme blocks the stylesheet paints', () => {
    expect(dataThemeNames(THEME_CSS).sort()).toEqual([...THEMES].sort())
  })

  it('are the values the picker list takes its colors from', () => {
    // The list is a real list of the page, so the chosen theme's variables color
    // it: no fixed palette, and no native popup that ignores the page.
    const list = ruleBody(APP_CSS, '.tpl')
    expect(list).toContain('background: var(--surface)')
    expect(list).toContain('border: 1px solid var(--border)')
    expect(ruleBody(APP_CSS, '.tpl li')).toContain('color: var(--text)')
  })
})
