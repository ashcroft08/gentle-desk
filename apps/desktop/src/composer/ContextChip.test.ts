// The context chip as the composer draws it (0.1.12, S5): one quiet number
// beside the model pills, with the token detail on hover. The component only
// renders what Rust pushed - it asks the shell nothing - so the last test pins
// that it never reads on its own (the refresh is the push, not a poll).
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render } from '@testing-library/svelte'
import type { ContextUsageView_Serialize as ContextUsageView } from '../contract/bindings'
import { invokeCalls, resetTauriMock } from '../test-support/tauri-mock'
import ContextChip from './ContextChip.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

function usage(over: Partial<ContextUsageView> = {}): ContextUsageView {
  return { tokens: 70_000, percent: 35, window: 200_000, ...over }
}

/** The chip's own element, or null when the component drew nothing. */
function chip(usage: ContextUsageView | null): HTMLElement | null {
  const { container } = render(ContextChip, { props: { usage } })
  return container.querySelector('.ctx') as HTMLElement | null
}

describe('the context chip', () => {
  it('draws the percentage, the bar and the tones of the scale', () => {
    const quiet = chip(usage())!
    expect(quiet.textContent?.trim()).toBe('35%')
    expect(quiet.classList.contains('warning')).toBe(false)
    expect(quiet.querySelector('.bar i')).toHaveProperty('style')
    expect((quiet.querySelector('.bar i') as HTMLElement).style.width).toBe('35%')

    expect(chip(usage({ percent: 78 }))?.classList.contains('warning')).toBe(true)
    expect(chip(usage({ percent: 96 }))?.classList.contains('danger')).toBe(true)
  })

  it('tells a reader the same reading, and keeps the tokens on hover', () => {
    const drawn = chip(usage())!

    expect(drawn.getAttribute('role')).toBe('status')
    expect(drawn.getAttribute('aria-label')).toContain('35')
    expect(drawn.getAttribute('title')).toBe('70k / 200k tokens')
  })

  it('draws a dash for a chat that just compacted, with no bar', () => {
    const drawn = chip(usage({ tokens: null, percent: null }))!

    expect(drawn.textContent?.trim()).toEqual(expect.stringMatching(/^[\u2013-]$/))
    expect(drawn.querySelector('.bar')).toBeNull()
  })

  it('draws nothing at all while there is no reading', () => {
    expect(chip(null)).toBeNull()
    expect(chip(usage({ tokens: null, percent: null, window: null }))).toBeNull()
  })

  it('asks the shell nothing: the push is the only source of a reading', async () => {
    chip(usage())
    await Promise.resolve()
    expect(invokeCalls).toEqual([])
  })
})
