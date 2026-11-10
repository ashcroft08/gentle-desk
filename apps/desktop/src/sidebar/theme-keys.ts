// The keyboard movement of the theme list, kept out of the component so the
// rule lives in one place: the arrow keys walk the options and stop at both
// ends (as 01a's script does), Home and End jump to the first and the last, and
// Tab stays inside the open list instead of reaching the page behind it.
//
// `null` means "this key moves nothing": the caller then decides what the key
// does, if anything at all.

/** The option index a key moves the focus to, `null` for a key that only acts. */
export function movedIndex(key: string, index: number, count: number): number | null {
  if (count === 0) return null
  const last = count - 1
  switch (key) {
    case 'ArrowDown':
      return Math.min(index + 1, last)
    case 'ArrowUp':
      return Math.max(index - 1, 0)
    case 'Home':
      return 0
    case 'End':
      return last
    default:
      return null
  }
}

/** The option a Tab moves the focus to: the same walk, never out of the list. */
export function trappedIndex(shift: boolean, index: number, count: number): number | null {
  if (count === 0) return null
  return shift ? Math.max(index - 1, 0) : Math.min(index + 1, count - 1)
}
