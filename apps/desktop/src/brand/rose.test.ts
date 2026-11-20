// The rose of gentle-shell-gpui as the welcome screen draws it: braille art plus
// a shade map, decoded to dots that bloom from the heart outward, and the numbers
// the canvas draws each frame with. Pure, so every rule is checked without a
// window; the canvas component only runs these numbers over time.

import { describe, expect, it } from 'vitest'
import { ROSE, SHADE } from './rose-art'
import {
  BASE_COLOUR,
  ROSE_DOTS,
  ROSE_HEIGHT,
  ROSE_WIDTH,
  WRITE_MS,
  decodeRose,
  frameAt,
  smoothstep,
} from './rose'

const HEART_X = ROSE_WIDTH * 0.5
const HEART_Y = ROSE_HEIGHT * 0.32

/** How far a dot sits from the heart, in dots. */
const distanceOf = (x: number, y: number) => Math.hypot(x - HEART_X, y - HEART_Y)

/** The lightness a dot rests at: its shade, at the rose's own colour. */
const restLightness = (shade: number) => Math.min(0.95, BASE_COLOUR.lightness * shade)

describe('the rose art', () => {
  it('is 17 rows of braille, each with a shade row', () => {
    expect(ROSE).toHaveLength(17)
    expect(SHADE).toHaveLength(ROSE.length)

    SHADE.forEach((row, index) => {
      expect(row, `shade row ${index}`).toBeTypeOf('string')
      expect(row.length, `shade row ${index}`).toBeGreaterThan(0)
      expect(row.replace(/[0-9 ]/g, '')).toBe('')
    })
  })

  it('decodes to the 1,779 dots the banner draws, 80 wide and 68 tall', () => {
    const art = decodeRose(ROSE, SHADE)

    expect(art.dots).toHaveLength(1779)
    expect(art.width).toBe(80)
    expect(art.height).toBe(68)
    expect(ROSE_DOTS).toHaveLength(1779)
    expect(ROSE_WIDTH).toBe(80)
    expect(ROSE_HEIGHT).toBe(ROSE.length * 4)
  })

  it('puts a braille cell on the 2x4 grid of dots, in braille order', () => {
    // 0x2801 is the top-left dot only; 0x28ff is all eight.
    expect(decodeRose(['⠁'], ['7']).dots.map((dot) => [dot.x, dot.y])).toEqual([[0, 0]])
    expect(decodeRose(['⣿'], ['7']).dots.map((dot) => [dot.x, dot.y])).toEqual([
      [0, 0],
      [0, 1],
      [0, 2],
      [1, 0],
      [1, 1],
      [1, 2],
      [0, 3],
      [1, 3],
    ])
    expect(decodeRose([' ⠁'], [' 7']).dots.map((dot) => [dot.x, dot.y])).toEqual([[2, 0]])
  })

  it('takes a dot brightness from the shade digit, and a mid shade when there is none', () => {
    // 0.55 + 0.075 * digit: 6 is the rose's mid shade and 9 its brightest.
    expect(decodeRose(['⠁'], ['6']).dots[0].shade).toBeCloseTo(1, 10)
    expect(decodeRose(['⠁'], ['9']).dots[0].shade).toBeCloseTo(0.55 + 0.075 * 9, 10)
    expect(decodeRose(['⠁'], ['0']).dots[0].shade).toBeCloseTo(0.55, 10)
    // A shade row shorter than its art gives the cells beyond it the mid shade.
    expect(decodeRose(['⠁⠁'], ['9']).dots.map((dot) => dot.shade)).toEqual([0.55 + 0.075 * 9, 1])
  })

  it('blooms from its heart outward: the farther a dot, the later it opens', () => {
    const byDistance = [...ROSE_DOTS].sort((a, b) => distanceOf(a.x, a.y) - distanceOf(b.x, b.y))
    const farthest = byDistance[byDistance.length - 1]

    byDistance.forEach((dot, index) => {
      expect(dot.bloom, `bloom of dot ${index}`).toBeGreaterThanOrEqual(0)
      expect(dot.bloom).toBeLessThanOrEqual(1)
      if (index > 0) {
        expect(dot.bloom).toBeGreaterThanOrEqual(byDistance[index - 1].bloom)
      }
    })
    // The farthest dot is the last to open, and the heart's neighbours open first.
    expect(farthest.bloom).toBeCloseTo(1, 10)
    expect(distanceOf(farthest.x, farthest.y)).toBeCloseTo(
      Math.max(...ROSE_DOTS.map((dot) => distanceOf(dot.x, dot.y))),
      10,
    )
    expect(byDistance[0].bloom).toBeLessThan(0.25)
    expect(byDistance[0].bloom).toBeGreaterThan(0)
  })
})

describe('the rose over time', () => {
  it('takes 2.6 s to bloom, which is what a frame asks of', () => {
    expect(WRITE_MS).toBe(2600)
  })

  it('is a rose at hsl(326, 100%, 73.1%)', () => {
    expect(BASE_COLOUR).toEqual({ hue: 326, saturation: 1, lightness: 0.731 })
  })

  it('has no dot open yet at the first frame', () => {
    const first = frameAt(0, false)

    expect(first).toHaveLength(ROSE_DOTS.length)
    first.forEach((drawn) => expect(drawn.alpha).toBe(0))
  })

  it('has every dot open at the last frame', () => {
    const last = frameAt(1, false)

    expect(last).toHaveLength(ROSE_DOTS.length)
    last.forEach((drawn) => expect(drawn.alpha).toBeCloseTo(1, 10))
  })

  it('opens a dot further the later its turn comes', () => {
    const middle = frameAt(0.2, false)
    const openEarly = middle.filter((_, index) => ROSE_DOTS[index].bloom <= 0.5)
    const openLate = middle.filter((_, index) => ROSE_DOTS[index].bloom > 0.5)

    expect(openEarly.length).toBeGreaterThan(0)
    expect(openLate.length).toBeGreaterThan(0)
    expect(Math.max(...openLate.map((drawn) => drawn.alpha))).toBeGreaterThan(0)
    const mean = (dots: typeof middle) =>
      dots.reduce((sum, dot) => sum + dot.alpha, 0) / dots.length
    expect(mean(openEarly)).toBeGreaterThan(mean(openLate))
  })

  it('rests at the last frame: no float, no sway, no glint', () => {
    frameAt(1, false).forEach((drawn, index) => {
      expect(drawn.x).toBeCloseTo(ROSE_DOTS[index].x, 6)
      expect(drawn.y).toBeCloseTo(ROSE_DOTS[index].y, 6)
      expect(drawn.lightness).toBeCloseTo(restLightness(ROSE_DOTS[index].shade), 10)
      expect(drawn.saturation).toBeCloseTo(BASE_COLOUR.saturation, 10)
    })
  })

  it('a still frame never floats, sways or shines, at any reveal', () => {
    // Floating would lift every dot and swaying would turn it about the heart;
    // both show as a dot leaving its own radius, which is this cross product:
    // zero means the dot moved in or out only, never across.
    const across = (drawn: { x: number; y: number }, art: { x: number; y: number }) =>
      (drawn.x - HEART_X) * (art.y - HEART_Y) - (drawn.y - HEART_Y) * (art.x - HEART_X)

    for (const reveal of [0.1, 0.2, 0.5, 0.6, 0.8, 1]) {
      const still = frameAt(reveal, true)

      expect(still).toHaveLength(ROSE_DOTS.length)
      still.forEach((drawn, index) => {
        const art = ROSE_DOTS[index]
        expect(across(drawn, art), `across at ${reveal}`).toBeCloseTo(0, 9)
        expect(drawn.lightness, `lightness at ${reveal}`).toBeCloseTo(
          restLightness(art.shade),
          10,
        )
        expect(drawn.saturation, `saturation at ${reveal}`).toBeCloseTo(
          BASE_COLOUR.saturation,
          10,
        )
      })
    }

    // Once the bloom is done the unfold is done too, so a still frame sits on the art.
    for (const reveal of [0.5, 0.6, 0.8, 1]) {
      frameAt(reveal, true).forEach((drawn, index) => {
        expect(drawn.x, `x at ${reveal}`).toBeCloseTo(ROSE_DOTS[index].x, 6)
        expect(drawn.y, `y at ${reveal}`).toBeCloseTo(ROSE_DOTS[index].y, 6)
      })
    }
  })

  it('floats and shines while it opens, and stops by the last frame', () => {
    const moving = frameAt(0.5, false)

    const lifted = moving.filter((dot, index) => Math.abs(dot.y - ROSE_DOTS[index].y) > 0.01)
    const brighter = moving.filter(
      (dot, index) => dot.lightness > restLightness(ROSE_DOTS[index].shade) + 1e-9,
    )
    const washed = moving.filter(
      (dot) => dot.saturation < BASE_COLOUR.saturation - 1e-9,
    )

    expect(lifted.length).toBeGreaterThan(0)
    expect(brighter.length).toBeGreaterThan(0)
    expect(washed.length).toBeGreaterThan(0)
    // The glint lightens a dot towards white and drains its colour together.
    brighter.forEach((dot) => expect(dot.lightness).toBeLessThanOrEqual(0.97 + 1e-9))
  })

  it('smoothsteps between 0 and 1, and holds them outside', () => {
    expect(smoothstep(0)).toBe(0)
    expect(smoothstep(1)).toBe(1)
    expect(smoothstep(0.5)).toBeCloseTo(0.5, 12)
    expect(smoothstep(0.25)).toBeCloseTo(0.15625, 12)
    expect(smoothstep(-2)).toBe(0)
    expect(smoothstep(3)).toBe(1)
  })
})
