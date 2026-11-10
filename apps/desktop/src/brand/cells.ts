// The rules that turn the stored pixel rows into drawable cells and the order
// they are typed in. Pure, so the order is testable without a window.

/** How long each column waits behind the one before it. */
export const STEP_MS = 14

export type Cell = {
  /** Column, 0 at the left. */
  x: number
  /** Row, 0 at the top. */
  y: number
  /** A pixel of the letters, or of the dimmer shadow the lettering casts. */
  kind: 'letter' | 'shadow'
  /** How long after mount this cell appears, in milliseconds. */
  delay: number
}

/**
 * One cell per '#' (letter) or 's' (shadow), typed left to right: a column
 * appears one step after the column before it, and the rows inside a column are
 * staggered by less than a step so no cell ever comes before a cell of an
 * earlier column.
 */
export function cellsOf(mask: string[]): Cell[] {
  const cells: Cell[] = []
  mask.forEach((row, y) => {
    for (let x = 0; x < row.length; x += 1) {
      const mark = row[x]
      if (mark !== '#' && mark !== 's') continue
      cells.push({
        x,
        y,
        kind: mark === '#' ? 'letter' : 'shadow',
        delay: x * STEP_MS + (y % 4) * (STEP_MS / 4),
      })
    }
  })
  return cells
}
