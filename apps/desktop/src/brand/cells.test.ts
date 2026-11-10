// The pixel lettering is stored as rows of '#' and '.'; these are the rules
// that turn it into the cells the wordmark draws and the order they appear in.

import { describe, expect, it } from 'vitest'
import { cellsOf, STEP_MS } from './cells'
import { MASK } from './mask'

describe('the wordmark cells', () => {
  it('makes one cell per # and none for a dot, with its column and row', () => {
    expect(cellsOf(['#.', '.#']).map((cell) => [cell.x, cell.y])).toEqual([
      [0, 0],
      [1, 1],
    ])
  })

  it('tells a pixel of the letters from a pixel of the shadow', () => {
    expect(cellsOf(['#s.']).map((cell) => cell.kind)).toEqual(['letter', 'shadow'])
  })

  it('reveals left to right: a cell never appears before one in an earlier column', () => {
    const cells = cellsOf(['#.#', '#.#'])

    const delayOf = (x: number) => Math.min(...cells.filter((cell) => cell.x === x).map((cell) => cell.delay))
    expect(delayOf(0)).toBe(0)
    expect(delayOf(2)).toBeGreaterThan(delayOf(0))
    expect(Math.max(...cells.filter((cell) => cell.x === 0).map((cell) => cell.delay))).toBeLessThan(
      delayOf(2),
    )
  })

  it('takes under two seconds to type the whole lettering', () => {
    const last = Math.max(...cellsOf(MASK).map((cell) => cell.delay))

    expect(last).toBeGreaterThan(0)
    expect(last).toBeLessThan(2000)
    expect(STEP_MS).toBeGreaterThan(0)
  })

  it('is a rectangle of the same width in every row, with letters and shadow in it', () => {
    expect(new Set(MASK.map((row) => row.length)).size).toBe(1)
    expect(MASK.some((row) => row.includes('#'))).toBe(true)
    expect(MASK.some((row) => row.includes('s'))).toBe(true)
  })
})
