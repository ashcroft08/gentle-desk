// The rose of gentle-shell-gpui over time, as pure numbers: the braille art
// decoded to dots, the order they open in, and what one frame looks like. The
// canvas in ./Rose.svelte only turns these into squares, so every rule the
// artboard showed - the bloom from the heart outward, the float, the sway, the
// glint, and the rest - is checked without a window.

import { ROSE, SHADE } from './rose-art'

/** How long the bloom takes, end to end. */
export const WRITE_MS = 2600
/** The reveal the bloom is done at: it happens in the first half of the write. */
const BLOOM_SHARE = 0.5
/** From this reveal the float and the sway ease out, so the rose comes to rest. */
const REST_FROM = 0.6
/** The colour every dot is shaded from: hsl(326, 100%, 73.1%). */
export const BASE_COLOUR = { hue: 326, saturation: 1, lightness: 0.731 }
/** How far the rose bobs, in dots, and how long one bob lasts. */
const BOB_AMPLITUDE = 1.8
const BOB_PERIOD_S = 5.2
/** How far it sways, in radians, and how long one sway lasts. */
const SWAY_AMPLITUDE = 0.022
const SWAY_PERIOD_S = 7.3
/** The glint: where its band is at a reveal, how broad and how strong it is. */
const GLINT_SLOPE = 1.8
const GLINT_OFFSET = -0.4
const GLINT_BREADTH = 0.12
const GLINT_STRENGTH = 0.35
/** A glinted dot keeps 65% of its colour and goes this bright. */
const GLINT_WASH = 0.35
const GLINT_LIGHTNESS = 0.97
/** No dot is drawn brighter than this, whatever its shade. */
const LIGHTNESS_CEILING = 0.95
/** A dot waits this share of its bloom time, and then takes this long to open. */
const OPEN_WAIT = 0.75
const OPEN_WIDTH = 0.25
/** A dot starts folded this far in towards the heart, and opens the rest. */
const UNFOLD_FLOOR = 0.82
const UNFOLD_SPAN = 0.18
/** The side of one dot's square, in dots: the canvas keeps it a pixel at least. */
export const DOT_SIDE = 0.55
/** Padding around the art, in CSS pixels. */
export const ROSE_PADDING = 8

/** The 2x4 dot grid of a braille cell, in braille's own bit order. */
const BITS = [
  [0, 0],
  [0, 1],
  [0, 2],
  [1, 0],
  [1, 1],
  [1, 2],
  [0, 3],
  [1, 3],
] as const
const BRAILLE_FIRST = 0x2800
const BRAILLE_LAST = 0x28ff
/** A dot is lit by its cell's 0-9 shade digit, and a cell with none is a mid shade. */
const SHADE_BASE = 0.55
const SHADE_STEP = 0.075
const SHADE_MID_DIGIT = 6

/** One dot of the art, in dots: its column and row, its brightness and its turn. */
export type RoseDot = {
  x: number
  y: number
  /** Brightness factor from the shade map: 0.55 + 0.075 * digit. */
  shade: number
  /** 0 next to the heart, 1 at the farthest dot: the share of the bloom it waits. */
  bloom: number
}

export type RoseArt = { dots: RoseDot[]; width: number; height: number }

/** The heart the rose blooms from, in dots: the middle, a third of the way down. */
function heartOf(width: number, height: number): { x: number; y: number } {
  return { x: width * 0.5, y: height * 0.32 }
}

function shadeOf(digit: string | undefined): number {
  const known = digit !== undefined && digit >= '0' && digit <= '9'
  return SHADE_BASE + SHADE_STEP * (known ? Number(digit) : SHADE_MID_DIGIT)
}

/**
 * Turns the braille rows and their shade rows into dots. A braille cell is 2
 * columns by 4 rows, so a row of the art is four rows of dots; a cell's digit
 * lights all of its dots alike. Each dot also learns when it opens: its distance
 * from the heart over the farthest distance.
 */
export function decodeRose(rows: string[], shadeRows: string[]): RoseArt {
  const dots: RoseDot[] = []
  let width = 0

  rows.forEach((row, rowIndex) => {
    const shades = shadeRows[rowIndex] ?? ''
    Array.from(row).forEach((cell, column) => {
      const code = cell.codePointAt(0) ?? 0
      if (code < BRAILLE_FIRST || code > BRAILLE_LAST) return
      const bits = code - BRAILLE_FIRST
      const shade = shadeOf(shades[column])
      BITS.forEach(([dx, dy], bit) => {
        if ((bits & (1 << bit)) === 0) return
        const x = column * 2 + dx
        dots.push({ x, y: rowIndex * 4 + dy, shade, bloom: 0 })
        width = Math.max(width, x + 1)
      })
    })
  })

  const height = rows.length * 4
  const heart = heartOf(width, height)
  const farthest = Math.max(
    1,
    ...dots.map((dot) => Math.hypot(dot.x - heart.x, dot.y - heart.y)),
  )
  dots.forEach((dot) => {
    dot.bloom = Math.hypot(dot.x - heart.x, dot.y - heart.y) / farthest
  })
  return { dots, width, height }
}

export const ROSE_ART = decodeRose(ROSE, SHADE)
export const ROSE_DOTS = ROSE_ART.dots
export const ROSE_WIDTH = ROSE_ART.width
export const ROSE_HEIGHT = ROSE_ART.height

/** The smooth 0..1 ramp the artboard eases everything with. */
export function smoothstep(t: number): number {
  const clamped = Math.min(1, Math.max(0, t))
  return clamped * clamped * (3 - 2 * clamped)
}

/** What one dot looks like in one frame: where, how open, and in what colour. */
export type RoseDotDraw = {
  /** Position in dots, after the unfold, the sway and the bob. */
  x: number
  y: number
  /** 0 is folded away and not drawn; 1 is the fully open petal. */
  alpha: number
  lightness: number
  saturation: number
}

/** How much of the glint a dot catches: a band that sweeps across the petals. */
function glintAt(dot: RoseDot, glint: number): number {
  const along = (dot.x / ROSE_WIDTH + dot.y / ROSE_HEIGHT) * 0.5
  const closeness = Math.max(0, 1 - Math.abs((along - glint) / GLINT_BREADTH))
  return closeness ** 2 * GLINT_STRENGTH
}

/**
 * One frame of the rose, dot for dot. `reveal` runs 0 (nothing yet) to 1 (open
 * and at rest); `still` takes the float, the sway and the glint away, which is
 * how reduced motion sees it and how it stays once it has bloomed.
 */
export function frameAt(reveal: number, still: boolean): RoseDotDraw[] {
  const seconds = (reveal * WRITE_MS) / 1000
  const float = still ? 0 : 1 - smoothstep((reveal - REST_FROM) / (1 - REST_FROM))
  const bob = Math.sin((seconds * Math.PI * 2) / BOB_PERIOD_S) * BOB_AMPLITUDE * float
  const sway = Math.sin((seconds * Math.PI * 2) / SWAY_PERIOD_S) * SWAY_AMPLITUDE * float
  const swing = Math.sin(sway)
  const lift = Math.cos(sway)
  const glint = reveal * GLINT_SLOPE + GLINT_OFFSET
  const bloom = reveal / BLOOM_SHARE
  const heart = heartOf(ROSE_WIDTH, ROSE_HEIGHT)

  return ROSE_DOTS.map((dot) => {
    const alpha = smoothstep((bloom - dot.bloom * OPEN_WAIT) / OPEN_WIDTH)
    const unfold = UNFOLD_FLOOR + UNFOLD_SPAN * alpha
    const dx = (dot.x - heart.x) * unfold
    const dy = (dot.y - heart.y) * unfold
    const shine = still ? 0 : glintAt(dot, glint)
    const rest = Math.min(LIGHTNESS_CEILING, BASE_COLOUR.lightness * dot.shade)
    return {
      x: heart.x + dx * lift - dy * swing,
      y: heart.y + dx * swing + dy * lift + bob,
      alpha,
      lightness: rest + (GLINT_LIGHTNESS - rest) * shine,
      saturation: BASE_COLOUR.saturation * (1 - GLINT_WASH * shine),
    }
  })
}
