// The context rail (design/composer-dock S1, S5): the 2px line ABOVE the
// composer, outside its border, that fills with the context percentage. The
// percent sits at the left with a clock glyph and the model's window at the
// right; there is no chip and no box. A null reading is never a zero: no reading
// is no rail, and a chat that just compacted draws a dash with an empty rail.
//
// The tone is the reader's own scale, the one the chip already used
// (`./context`): quiet until 75%, the warning token from there. The component
// asks the shell nothing - the push is the only source of a reading.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render } from '@testing-library/svelte'
import type { ContextUsageView_Serialize as ContextUsageView } from '../contract/bindings'
import { invokeCalls, resetTauriMock } from '../test-support/tauri-mock'
import ContextRail from './ContextRail.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

function usage(over: Partial<ContextUsageView> = {}): ContextUsageView {
  return { tokens: 70_000, percent: 35, window: 200_000, ...over }
}

/** The rail's own element, or null when the component drew nothing. */
function rail(reading: ContextUsageView | null): HTMLElement | null {
  const { container } = render(ContextRail, { props: { usage: reading } })
  return container.querySelector('.ctx-rail') as HTMLElement | null
}

/** The fill of the rail, or null when it draws no fill at all. */
function fill(reading: ContextUsageView | null): HTMLElement | null {
  return rail(reading)?.querySelector('.rail i') as HTMLElement | null
}

describe('the context rail', () => {
  it('fills the rail with the percentage pi reported', () => {
    expect(fill(usage())?.style.width).toBe('35%')
    expect(fill(usage({ percent: 6 }))?.style.width).toBe('6%')
  })

  it('turns to the warning tone past 75 %, and keeps the fill in step', () => {
    expect(rail(usage({ percent: 74 }))?.classList.contains('warning')).toBe(false)

    const near = rail(usage({ percent: 82 }))!
    expect(near.classList.contains('warning')).toBe(true)
    expect((near.querySelector('.rail i') as HTMLElement).style.width).toBe('82%')
  })

  it('puts the percent on the left with a clock, and the window on the right', () => {
    const drawn = rail(usage())!
    const label = drawn.querySelector('.lab') as HTMLElement
    const [left, right] = Array.from(label.children) as HTMLElement[]

    expect(left.textContent?.trim()).toBe('35%')
    // The clock: one ring and its two hands, the app's own icon.
    expect(left.querySelector('svg circle')).toBeTruthy()
    expect(left.querySelector('svg path')).toBeTruthy()
    expect(right.textContent?.trim()).toBe('200k')
  })

  it('tells a reader the same reading and keeps the tokens on hover', () => {
    const drawn = rail(usage())!
    const status = drawn.querySelector('[role="status"]') as HTMLElement

    expect(status.getAttribute('aria-label')).toContain('35')
    expect(status.getAttribute('title')).toBe('70k / 200k tokens')
  })

  it('draws a dash and an empty rail for a chat that just compacted', () => {
    const drawn = rail(usage({ tokens: null, percent: null }))!

    expect(drawn.querySelector('.lab')?.textContent).toContain('\u2013')
    expect(drawn.querySelector('.rail i')).toBeNull()
    expect(drawn.classList.contains('warning')).toBe(false)
  })

  it('draws nothing at all while there is no reading', () => {
    expect(rail(null)).toBeNull()
    expect(rail(usage({ tokens: null, percent: null, window: null }))).toBeNull()
  })

  it('asks the shell nothing: the push is the only source of a reading', async () => {
    rail(usage())
    await Promise.resolve()

    expect(invokeCalls).toEqual([])
  })
})
