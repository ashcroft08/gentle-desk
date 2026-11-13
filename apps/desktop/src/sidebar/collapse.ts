// The sidebar collapse rule (responsive-sidebar S2, S3): the window's width
// decides unless the user has toggled the sidebar by hand, and a manual toggle
// wins for the whole run — a session the user expanded on a narrow window is
// never re-collapsed by a resize, and one collapsed on a wide window stays so.

/** Below this window width the sidebar starts collapsed (S2): 280px of sidebar
 * plus a conversation in the readable band have to fit side by side. */
export const SIDEBAR_AUTO_COLLAPSE_BELOW = 800

/**
 * Whether the sidebar is collapsed now.
 *
 * `manual` is the user's own toggle: `null` while untouched, and once touched
 * it wins for the whole run (S3). `width` is the window's inner width, `null`
 * before the first measurement — which draws expanded rather than guessing.
 */
export function collapsedOf(manual: boolean | null, width: number | null): boolean {
  if (manual !== null) return manual
  return width !== null && width < SIDEBAR_AUTO_COLLAPSE_BELOW
}
