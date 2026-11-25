// The Settings rail's update dot (packages-status S8, T5; L2): the mark the
// "Actualizaciones" entry wears while something waits, so a user who opened
// Settings on General knows where to go. It is the footer gear's dot (0.1.14)
// in a second place - the same 6px of the same accent token - so the two
// promises read as one promise, and it is decoration: the entry's own
// `aria-description` (settings/copy.ts) says the same news in words.
//
// The look is pinned here as DECLARATIONS read out of app.css (comments
// stripped, selector lists understood), never as the sheet's raw text: the
// stylesheet explains itself in prose, which a substring search would let
// satisfy these tests with the declarations gone. The shape lives in ONE rule
// shared with the gear, and each place only says where the dot sits - so the
// two can never drift apart.
//
// What a stylesheet read cannot prove is how the dot LOOKS in a real window:
// that is reported as unverified and judged on the running app.
import '../test-support/dom-env'
import { afterEach, describe, expect, it } from 'vitest'
import { cleanup, render } from '@testing-library/svelte'
import APP_CSS from '../app.css?raw'
import RailDot from './RailDot.svelte'

/** The sheet without its comments: prose never satisfies a selector. */
const SHEET = APP_CSS.replace(/\/\*[\s\S]*?\*\//g, '')

/** The gear's dot, which is the look this one repeats. */
const GEAR = '.foot .ic .dot'
/** The rail entry's dot: the selector the dialog's tablist draws it under. */
const RAIL = ".settings-rail [role='tab'] .dot"

/**
 * EVERY rule whose comma-separated selector list names `selector`, in sheet
 * order: a shared rule answers for both places, and the place's own rule is the
 * second one that names it.
 */
function rules(selector: string): string[] {
  const pattern = /([^{}]+)\{([^{}]*)\}/g
  const found: string[] = []
  let rule = pattern.exec(SHEET)
  while (rule !== null) {
    const list = rule[1].split(',').map((part) => part.trim())
    if (list.includes(selector)) found.push(rule[2])
    rule = pattern.exec(SHEET)
  }
  return found
}

/** One declaration of a selector, from the first rule that makes it. */
function declared(selector: string, property: string): string | null {
  for (const body of rules(selector)) {
    const found = new RegExp(`(?:^|;)\\s*${property}:\\s*([^;]+);`).exec(body)
    if (found !== null) return found[1].trim()
  }
  return null
}

/** The shape of a dot as the sheet declares it: what makes it look like itself. */
function shape(selector: string): Record<string, string | null> {
  return {
    width: declared(selector, 'width'),
    height: declared(selector, 'height'),
    radius: declared(selector, 'border-radius'),
    background: declared(selector, 'background'),
  }
}

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

describe('the rail\'s update dot', () => {
  it('draws one mark the reader is not told about, since the entry says it in words', () => {
    const { container } = render(RailDot)

    const dots = container.querySelectorAll('.dot')
    expect(dots).toHaveLength(1)
    // Decoration: the promise is carried by the entry's description, never by
    // this element and never by colour alone.
    expect(dots[0].getAttribute('aria-hidden')).toBe('true')
    // It adds no word to the entry, so the entry's own name cannot move.
    expect(dots[0].textContent).toBe('')
  })

  it('wears exactly the gear\'s dot: the same shape, one rule, both places', () => {
    const gear = shape(GEAR)
    const rail = shape(RAIL)

    expect(rail).toEqual(gear)
    // The house's own token, so the six themes paint it without a line of ours.
    expect(gear.background).toBe('var(--accent)')
  })

  it('is never wider than 8px', () => {
    const width = shape(RAIL).width

    expect(width).toMatch(/^\d+px$/)
    expect(Number.parseInt(width ?? '', 10)).toBeLessThanOrEqual(8)
  })

  it('parks at the entry\'s right end without taking room from its word', () => {
    // The rail entry is a flex line, so `margin-left: auto` is what pushes the
    // dot to the end: it is the LAST item, its width is fixed and tiny
    // (`flex: none`), and it never sits over the word the way an absolutely
    // positioned dot would. That is the whole "it must not shift the text" rule.
    expect(declared(RAIL, 'margin-left')).toBe('auto')
    expect(declared(RAIL, 'flex')).toBe('none')
    expect(declared(RAIL, 'position')).toBeNull()
  })
})
