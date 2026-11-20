// The rose as the welcome screen draws it: a canvas named by the screen, blooming
// once from its heart outward and then STILL, so a finished rose asks for no more
// frames. Reduced motion gets the open rose at once and never a frame at all.
// jsdom has no canvas, so the drawing surface here is a stub that only counts.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import Rose from './Rose.svelte'
import { ROSE_HEIGHT, ROSE_WIDTH, WRITE_MS, frameAt } from './rose'

const LABEL = 'La rosa de Gentle Shell'

/** One `fillRect`: where a dot of the rose lands, in CSS pixels. */
type Rect = [number, number, number, number]

type Surface = {
  /** How many times the component set up the canvas: one per drawn frame. */
  frames: number
  /** Every rect of the last frame. */
  rects: Rect[]
  fillStyle: string
  setTransform(...args: number[]): void
  fillRect(...rect: Rect): void
}

function surface(): Surface {
  const drawn: Surface = {
    frames: 0,
    rects: [],
    fillStyle: '',
    setTransform: () => {
      drawn.frames += 1
      drawn.rects = []
    },
    fillRect: (...rect: Rect) => {
      drawn.rects.push(rect)
    },
  }
  return drawn
}

/** requestAnimationFrame under our hand, so a frame is only drawn when we say. */
function frames() {
  let next = 0
  const pending = new Map<number, FrameRequestCallback>()
  const request = vi.fn((callback: FrameRequestCallback) => {
    next += 1
    pending.set(next, callback)
    return next
  })
  const cancel = vi.fn((handle: number) => {
    pending.delete(handle)
  })
  const flush = (now: number) => {
    const ready = [...pending.entries()]
    pending.clear()
    ready.forEach(([, callback]) => callback(now))
  }
  return { request, cancel, flush, pending }
}

const realGetContext = HTMLCanvasElement.prototype.getContext

beforeEach(() => {
  // The rose times itself from `performance.now()` when it mounts.
  vi.spyOn(performance, 'now').mockReturnValue(0)
})

afterEach(() => {
  cleanup()
  vi.restoreAllMocks()
  vi.unstubAllGlobals()
  HTMLCanvasElement.prototype.getContext = realGetContext
})

/** Mounts the rose over the stubbed surface, drawing frames only when flushed. */
function mountRose(options: { reducedMotion?: boolean; height?: number; silent?: boolean } = {}) {
  const channel = frames()
  const drawn = surface()
  vi.stubGlobal('requestAnimationFrame', channel.request)
  vi.stubGlobal('cancelAnimationFrame', channel.cancel)
  vi.stubGlobal('matchMedia', (query: string) => ({
    matches: options.reducedMotion === true && query.includes('prefers-reduced-motion'),
    media: query,
    onchange: null,
    addListener: () => {},
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => false,
  }))
  // jsdom has no canvas and its own `getContext` answers null (and logs about it),
  // so the stub states the contract without the noise.
  HTMLCanvasElement.prototype.getContext = (options.silent
    ? () => null
    : () => drawn) as unknown as HTMLCanvasElement['getContext']

  const view = render(Rose, { props: { label: LABEL, height: options.height } })
  return { ...view, drawn, ...channel }
}

const canvasOf = (container: HTMLElement) => container.querySelector('canvas') as HTMLCanvasElement

describe('the rose', () => {
  it('is one image the caller names', () => {
    const { container } = mountRose()

    expect(screen.getByRole('img', { name: LABEL })).toBe(canvasOf(container))
  })

  it('blooms over 2.6 s and then asks for no more frames', () => {
    const { drawn, request, flush, pending } = mountRose()

    // The first frame is the rose at rest: nothing open, nothing painted yet.
    expect(drawn.frames).toBe(1)
    expect(drawn.rects).toEqual([])
    expect(request).toHaveBeenCalledTimes(1)

    flush(WRITE_MS / 4)
    expect(drawn.frames).toBe(2)
    expect(drawn.rects.length).toBeGreaterThan(0)
    expect(drawn.rects.length).toBeLessThan(frameAt(1, false).length)
    expect(request).toHaveBeenCalledTimes(2)

    flush(WRITE_MS)
    expect(drawn.frames).toBe(3)
    // Every dot of the finished rose, and the asking stops here for good.
    expect(drawn.rects).toHaveLength(frameAt(1, false).length)

    flush(WRITE_MS + 1000)
    flush(WRITE_MS + 2000)
    expect(drawn.frames).toBe(3)
    expect(request).toHaveBeenCalledTimes(2)
    expect(pending.size).toBe(0)
  })

  it('draws the open rose once, and never a frame, under reduced motion', () => {
    const { drawn, request, pending } = mountRose({ reducedMotion: true })

    expect(drawn.frames).toBe(1)
    expect(request).not.toHaveBeenCalled()
    expect(pending.size).toBe(0)

    // That one frame is the rose at REST: the pure still frame, dot for dot.
    const dot = 96 / ROSE_HEIGHT
    const rests = frameAt(1, true)
    expect(drawn.rects).toHaveLength(rests.length)
    drawn.rects.forEach((rect, index) => {
      expect(rect[0]).toBeCloseTo(8 + rests[index].x * dot, 6)
      expect(rect[1]).toBeCloseTo(8 + rests[index].y * dot, 6)
    })
  })

  it('cancels its waiting frame when it is taken away', () => {
    const { cancel, request, pending, unmount } = mountRose()

    expect(request).toHaveBeenCalledTimes(1)
    const waiting = [...pending.keys()]

    unmount()

    expect(cancel).toHaveBeenCalledWith(waiting[0])
    expect(pending.size).toBe(0)
  })

  it('draws at the device pixel ratio, and sizes the art by height', () => {
    vi.stubGlobal('devicePixelRatio', 2)
    const { container } = mountRose()
    const canvas = canvasOf(container)

    const cssWidth = Number.parseFloat(canvas.style.width)
    const cssHeight = Number.parseFloat(canvas.style.height)
    // 8 px of padding around the art, and the art 68 dots tall at 96 px.
    expect(cssHeight).toBeCloseTo(96 + 16, 6)
    expect(cssWidth).toBeCloseTo((ROSE_WIDTH * 96) / ROSE_HEIGHT + 16, 6)
    expect(canvas.width).toBe(Math.ceil(cssWidth * 2))
    expect(canvas.height).toBe(Math.ceil(cssHeight * 2))
  })

  it('draws the art taller when the screen asks for a taller rose', () => {
    const { container } = mountRose({ height: 200 })
    const canvas = canvasOf(container)

    expect(canvas.style.height).toBe('216px')
    expect(Number.parseFloat(canvas.style.width)).toBeCloseTo(
      (ROSE_WIDTH * 200) / ROSE_HEIGHT + 16,
      6,
    )
  })

  it('says nothing at all when the canvas silences it', () => {
    const { request, flush } = mountRose({ silent: true })

    expect(() => flush(WRITE_MS)).not.toThrow()
    expect(request).toHaveBeenCalledTimes(1)
  })

  it('keeps the rose out of the tab order and gives it no click to replay', () => {
    const { container, request } = mountRose()
    const canvas = canvasOf(container)

    expect(canvas.hasAttribute('tabindex')).toBe(false)
    canvas.click()
    expect(request).toHaveBeenCalledTimes(1)
  })
})
