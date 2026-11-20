// The rose's size, as the window dictates (welcome-screen T2's follow-up): the
// maintainer looked at the screen maximized and saw a modest rose glued to the
// title, so the picture now grows with the window instead of being one number.
// The tiers are the decision, and they are checked here without a window: the
// component test in Home.test.ts is what proves the view obeys them.
import { afterEach, describe, expect, it, vi } from 'vitest'
import { ROSE_FALLBACK, roseHeightFor, watchRoseHeight } from './rose-height'
import { mediaWindow } from '../test-support/media'

/** The height the view would draw the rose at in a `width` x `height` window. */
function roseIn(width: number, height: number): number {
  return roseHeightFor((query) => {
    const min = /\(min-(width|height):\s*(\d+)px\)/.exec(query)
    if (min === null) return false
    return (min[1] === 'width' ? width : height) >= Number(min[2])
  })
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('the rose’s height in a window', () => {
  it('takes a step per window height', () => {
    expect(roseIn(1600, 1000)).toBe(136)
    expect(roseIn(1600, 760)).toBe(136)
    expect(roseIn(1600, 759)).toBe(116)
    expect(roseIn(1600, 600)).toBe(116)
    expect(roseIn(1600, 599)).toBe(96)
    expect(roseIn(1600, 480)).toBe(96)
    expect(roseIn(1600, 479)).toBe(72)
  })

  it('never gets bigger than the smaller step in a narrow window', () => {
    expect(roseIn(700, 1000)).toBe(136)
    expect(roseIn(699, 1000)).toBe(96)
    expect(roseIn(699, 400)).toBe(72)
    expect(roseIn(400, 900)).toBe(96)
  })

  it('keeps the modest rose when no window can be asked', () => {
    expect(ROSE_FALLBACK).toBe(96)
    const sizes: number[] = []
    // jsdom and a webview without `matchMedia` both land here.
    vi.stubGlobal('matchMedia', undefined)
    const stop = watchRoseHeight((px) => sizes.push(px))
    stop()
    expect(sizes).toEqual([ROSE_FALLBACK])
  })

  it('follows the window once per real change, and stops when it is told to', () => {
    const window = mediaWindow(1600, 1000)
    const sizes: number[] = []
    const stop = watchRoseHeight((px) => sizes.push(px))

    expect(sizes).toEqual([136])
    window.resize(1000, 560)
    expect(sizes).toEqual([136, 96])
    // Narrow, and 96 either way: the rose is not redrawn for a size it has.
    window.resize(640, 1000)
    expect(sizes).toEqual([136, 96])
    window.resize(1600, 1000)
    expect(sizes).toEqual([136, 96, 136])

    stop()
    window.resize(1000, 560)
    expect(sizes).toEqual([136, 96, 136])
  })
})
