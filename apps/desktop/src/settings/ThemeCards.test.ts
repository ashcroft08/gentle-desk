// The Appearance section's theme grid (the reference's theme cards,
// crates/app/src/features/settings/theme.rs:463-533): one card per theme, each
// drawn as a miniature terminal over THAT theme's own background - a prompt
// chevron in its accent, a letter pair in its foreground, an "ok" in its own
// green, a path in its muted grey, and the palette dots it can spare - then the
// theme's drawn name, the selected card outlined in the accent and checked.
//
// The preview is painted, not instantiated: the card carries a `data-theme`
// attribute, and `theme.css` already scopes every token to it, so the miniature
// takes the same variables the window would and this component never spells a
// colour. That is the whole reason the tests below read BOTH stylesheets: the
// rules in `app.css` say which token each part uses, and `theme.css` says what
// each theme's token is, so a known value pins the pair.
//
// What a stylesheet read cannot prove: that the result LOOKS like a terminal on
// each of the six themes. That needs a real window and is reported as unverified.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, within } from '@testing-library/svelte'
import type { ThemeView } from '../contract/bindings'
import { intents, openSettings, pane } from '../test-support/settings'
import { resetTauriMock } from '../test-support/tauri-mock'
import { setLanguage } from '../i18n/language.svelte'
import { THEMES, themeLabelsCopy } from '../theme/themes'
import APP_CSS from '../app.css?raw'
import THEME_CSS from '../theme.css?raw'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The six drawn names, in the order the cards are drawn. */
const NAMES = THEMES.map((theme) => themeLabelsCopy('en')[theme])

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setLanguage('en')
})

/** The theme the pushed state carries: the card that must wear the check. */
async function appearance(theme: ThemeView = 'system'): Promise<HTMLElement> {
  const dialog = await openSettings({}, '0.1.9', { theme })
  return pane(dialog)
}

/** The cards, in the order they are drawn. */
function cards(panel: HTMLElement): HTMLElement[] {
  return within(panel).getAllByRole('button')
}

/** The `SetTheme` intents the bridge received, oldest first. */
function themeIntents(): ThemeView[] {
  return intents()
    .map((intent) => intent as { type: string; data?: { theme: ThemeView } })
    .filter((intent) => intent.type === 'SetTheme')
    .map((intent) => intent.data!.theme)
}

/** One rule's DECLARATIONS, by its exact selector: comments removed. */
function declarations(css: string, selector: string): string {
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const match = css.match(new RegExp(`(?:^|[}\\n])\\s*${escaped}\\s*\\{([^}]*)\\}`))
  if (match === null) throw new Error(`no rule for ${selector}`)
  return match[1].replace(/\/\*[\s\S]*?\*\//g, '')
}

/** Every `@media (query)` block, concatenated: the app has more than one. */
function media(query: string): string {
  const blocks: string[] = []
  const marker = `@media (${query})`
  let at = APP_CSS.indexOf(marker)
  while (at >= 0) {
    const open = APP_CSS.indexOf('{', at)
    let depth = 0
    for (let index = open; index < APP_CSS.length; index += 1) {
      if (APP_CSS[index] === '{') depth += 1
      if (APP_CSS[index] === '}') {
        depth -= 1
        if (depth === 0) {
          blocks.push(APP_CSS.slice(open + 1, index))
          at = APP_CSS.indexOf(marker, index)
          break
        }
      }
    }
    if (depth !== 0) break
  }
  return blocks.join('\n')
}

/** One rule's body INSIDE every block of `query`. */
function scoped(query: string, selector: string): string {
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const match = media(query).match(new RegExp(`(?:^|[}\\n])\\s*${escaped}\\s*\\{([^}]*)\\}`))
  return match?.[1] ?? ''
}

/** The token one palette dot is painted with, read off its inline declaration. */
function dotToken(dot: Element): string | null {
  const inline = (dot as HTMLElement).style.getPropertyValue('--dot') || dot.getAttribute('style')
  return /var\(--([a-z0-9-]+)\)/.exec(inline ?? '')?.[1] ?? null
}

describe('the theme cards', () => {
  it('draws one card per theme, in the picker order, each named by its label', async () => {
    const panel = await appearance()

    expect(cards(panel)).toHaveLength(THEMES.length)
    // Each card scopes a theme to its own miniature: the attribute is what
    // `theme.css` selects, so it is the card's whole identity.
    expect(cards(panel).map((card) => card.querySelector('.tc-shot')?.getAttribute('data-theme'))).toEqual(
      [...THEMES],
    )
    for (const name of NAMES) {
      expect(within(panel).getByRole('button', { name }), name).toBeTruthy()
    }
  })

  it('marks the theme the state carries, and only that one', async () => {
    const panel = await appearance('Gentle')

    const pressed = cards(panel).map((card) => card.getAttribute('aria-pressed'))
    expect(pressed).toEqual(THEMES.map((theme) => (theme === 'Gentle' ? 'true' : 'false')))
    // The check is decoration beside the name, and the outlined one is the check.
    expect(panel.querySelectorAll('.tc-check')).toHaveLength(1)
    expect(cards(panel)[THEMES.indexOf('Gentle')].querySelector('.tc-check')).not.toBeNull()
  })

  it('switches through the intent the picker already sent', async () => {
    const panel = await appearance('system')

    await fireEvent.click(within(panel).getByRole('button', { name: 'Gentleman-Sexy' }))
    expect(themeIntents()).toEqual(['Gentleman-Sexy'])

    // The theme already chosen is not a change: nothing leaves for it.
    await fireEvent.click(within(panel).getByRole('button', { name: 'System' }))
    expect(themeIntents()).toEqual(['Gentleman-Sexy'])
  })

  // A card is a real button, so the browser's own Enter and Space activation
  // fires the same click this test fires by hand; jsdom does not synthesize that
  // activation, so what is pinned here is the button it is given and the focus
  // ring the stylesheet draws on it.
  it('is reachable and activatable from the keyboard', async () => {
    const panel = await appearance('system')
    const card = within(panel).getByRole('button', { name: 'Gentle' }) as HTMLButtonElement

    expect(card.tagName).toBe('BUTTON')
    expect(card.type).toBe('button')
    // Not a roving tab stop: every card is in the tab order, unlike a listbox
    // option, since each one is a whole preview and not a line of a list.
    expect(card.tabIndex).toBe(0)
    card.focus()
    expect(document.activeElement).toBe(card)

    await fireEvent.click(card)
    expect(themeIntents()).toEqual(['Gentle'])

    expect(declarations(APP_CSS, '.tc:focus-visible')).toMatch(/outline:\s*1px solid var\(--accent\)/)
  })

  it('paints the miniature from the theme each card carries, and the theme value is its own', async () => {
    const panel = await appearance('Gentle')
    const shot = within(panel)
      .getByRole('button', { name: 'Gentle' })
      .querySelector('.tc-shot')

    // The card scopes the theme's own block of `theme.css` to its miniature.
    expect(shot?.getAttribute('data-theme')).toBe('Gentle')

    // ...and every part of the miniature takes that block's variables.
    expect(declarations(APP_CSS, '.tc-shot')).toMatch(/background:\s*var\(--bg\)/)
    expect(declarations(APP_CSS, '.tc-prompt')).toMatch(/color:\s*var\(--accent\)/)
    expect(declarations(APP_CSS, '.tc-text')).toMatch(/color:\s*var\(--text\)/)
    expect(declarations(APP_CSS, '.tc-ok')).toMatch(/color:\s*var\(--success\)/)
    expect(declarations(APP_CSS, '.tc-path')).toMatch(/color:\s*var\(--muted\)/)

    // The stylesheet's own words, for two themes: dark's block is `:root`'s, and
    // Gentleman-Sexy is the theme whose accent nothing else shares. The pair of
    // values above and the variables the miniature takes are what make a card
    // the theme it names - and not one palette wearing six names.
    const dark = declarations(THEME_CSS, "[data-theme='dark']")
    expect(dark).toMatch(/--bg:\s*#0e1116;/)
    expect(dark).toMatch(/--accent:\s*#7aa2f7;/)

    const sexy = declarations(THEME_CSS, "[data-theme='Gentleman-Sexy']")
    expect(sexy).toMatch(/--bg:\s*#060407;/)
    expect(sexy).toMatch(/--accent:\s*#f43888;/)
  })

  it('draws the palette dots from tokens the miniature already takes, and no invented ones', async () => {
    const panel = await appearance()
    const dots = [...cards(panel)[THEMES.indexOf('Gentle')].querySelectorAll('.tc-dot')]
    const tokens = dots.map(dotToken)

    // Seven, in the reference's order: red, yellow, green, then the rest.
    expect(tokens).toEqual(['danger', 'warning', 'success', 'accent', 'muted', 'faint', 'text'])

    // Every theme the grid previews declares every token a dot draws, so no dot
    // falls back to the window's own colour (the rule: draw what exists).
    for (const theme of ['dark', 'light', 'Gentle', 'Gentleman-Cute', 'Gentleman-Sexy']) {
      const block = declarations(THEME_CSS, `[data-theme='${theme}']`)
      for (const token of tokens) {
        expect(block, `${theme} declares --${token}`).toMatch(new RegExp(`--${token}:`))
      }
    }
  })

  it('lays the cards three to a row, two on a medium window and one on a narrow one', () => {
    expect(declarations(APP_CSS, '.tc-grid')).toMatch(
      /grid-template-columns:\s*repeat\(3,\s*minmax\(0,\s*1fr\)\)/,
    )
    expect(scoped('max-width: 900px', '.tc-grid')).toMatch(
      /grid-template-columns:\s*repeat\(2,\s*minmax\(0,\s*1fr\)\)/,
    )
    expect(scoped('max-width: 640px', '.tc-grid')).toMatch(
      /grid-template-columns:\s*minmax\(0,\s*1fr\)/,
    )
  })
})
