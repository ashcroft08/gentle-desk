// The rose's size, chosen from the WINDOW rather than fixed (welcome-screen T2's
// follow-up): maximized, one modest number left the picture glued to the title and
// lost in an empty column, so the view asks the window how tall it is and the rose
// takes the step that window affords. Pure numbers plus one subscription: what a
// frame looks like is ./rose.ts's business and drawing it is Rose.svelte's.
//
// The thresholds are read through `matchMedia` and not through `innerHeight`,
// because a media query is the one thing that also tells the view when it CHANGES
// (the listener fires on a resize), which is how a grown window grows the rose.

/** A height step and the window height it starts at, tallest first. */
const TIERS = [
  { min: 760, height: 136 },
  { min: 600, height: 116 },
  { min: 480, height: 96 },
] as const

/** A short window still gets a rose, a small one (and the step under every tier). */
const SMALLEST = 72
/** In a narrow pane the rose takes no more than this, so it fits at the sides. */
const NARROW = 96
/** Under this window width the rose stops growing: the pane, not the window. */
const NARROW_MIN = 700
/**
 * View B's rose stops at the second step (welcome-screen T3): that pane shares its
 * height with the composer, which is the protagonist there, so the picture grows
 * no further than 116 whatever the window affords - 136 is view A's, alone on its
 * screen. The view applies the cap; what a frame looks like stays ./rose.ts's.
 */
export const NEW_SESSION_ROSE_MAX = 116
/** What the view starts with where no window can be asked (jsdom, an old webview). */
export const ROSE_FALLBACK = 96

/** The queries the view listens to: one per tier, plus the narrow-window cap. */
export const ROSE_QUERIES = [
  ...TIERS.map((tier) => `(min-height: ${tier.min}px)`),
  `(min-width: ${NARROW_MIN}px)`,
] as const

/** The rose's height in the window these media answers describe. */
export function roseHeightFor(matches: (query: string) => boolean): number {
  const tier = TIERS.find((step) => matches(`(min-height: ${step.min}px)`))?.height ?? SMALLEST
  return matches(`(min-width: ${NARROW_MIN}px)`) ? tier : Math.min(tier, NARROW)
}

/**
 * Follows the window: calls `apply` with the rose's height now and again whenever
 * that height actually changes, and returns the way to stop following.
 * A browser without `matchMedia` gets the fallback once and nothing else.
 */
export function watchRoseHeight(apply: (height: number) => void): () => void {
  const media = globalThis.matchMedia
  if (typeof media !== 'function') {
    apply(ROSE_FALLBACK)
    return () => {}
  }
  const lists = new Map<string, MediaQueryList>()
  for (const query of ROSE_QUERIES) lists.set(query, media.call(globalThis, query))
  let last: number | null = null
  const update = () => {
    const height = roseHeightFor((query) => lists.get(query)?.matches === true)
    // One resize fires every list; the rose only hears the size it did not have.
    if (height === last) return
    last = height
    apply(height)
  }
  update()
  for (const list of lists.values()) list.addEventListener('change', update)
  return () => {
    for (const list of lists.values()) list.removeEventListener('change', update)
  }
}
