// Copy is an ICON that only exists under the pointer (maintainer, 2026-11-18: "el
// copiar que aparece en el chat, hazlo icono y tambien que solo aparezca cuando se
// pasa el mouse sobre el texto"). Two rules, both pinned here: no word is drawn,
// and the control is invisible until the message is hovered or it takes focus.
import { describe, expect, it } from 'vitest'
import { render } from '@testing-library/svelte'
import APP_CSS from '../app.css?raw'
import CopyButton from './CopyButton.svelte'

/** One rule's declarations, by its exact selector, comments removed. */
function declarations(selector: string): string {
  const start = APP_CSS.indexOf(`\n${selector} {`)
  if (start < 0) throw new Error(`no rule for ${selector}`)
  return APP_CSS.slice(start, APP_CSS.indexOf('}', start)).replace(/\/\*[\s\S]*?\*\//g, '')
}

/** The one button the component draws, scoped to its own container. */
function drawn() {
  const { container } = render(CopyButton, { props: { messageId: 'msg-0' } })
  const button = container.querySelector('button')
  if (button === null) throw new Error('the copy control drew no button')

  return button
}

describe('the copy control', () => {
  it('draws an icon and no word', () => {
    const button = drawn()

    expect(button.querySelector('svg')).not.toBeNull()
    expect(button.textContent?.trim()).toBe('')
  })

  it('keeps the word as its accessible name and its tooltip', () => {
    const button = drawn()

    expect(button.getAttribute('aria-label')).toBeTruthy()
    expect(button.getAttribute('title')).toBe(button.getAttribute('aria-label'))
  })

  it('is invisible until the message is hovered, and returns for the keyboard', () => {
    expect(declarations('.copy')).toContain('opacity: 0')
    // The reveal is its own rule: the message being hovered, or the button focused.
    expect(APP_CSS).toContain('.msg:hover .copy')
    expect(APP_CSS).toMatch(/\.copy:focus-visible\s*\{[^}]*opacity:\s*1/s)
  })
})
