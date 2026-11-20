// New session's folder, the line under it and the element that describes it
// (welcome-screen R1, S3): with a selected workspace nothing changed, and with
// none the button falls back to the workspace view A's primary action names -
// the one the user worked in last. A last workspace that is gone speaks for
// itself: Rust already worded and localized the refusal, so it is printed
// VERBATIM, and only a run with no last workspace at all keeps the sidebar's own
// "select a workspace" line. Split out of the component so the sidebar stays
// inside its line budget and the four cases are read in one place.
import type { WelcomeWorkspace_Serialize } from '../contract/bindings'
import { EMPTY_WHY_ID, NEW_SESSION_WHY_ID } from './copy'

/** What the sidebar's New session button is in this state. */
export type NewSessionTarget = {
  /** The folder a new chat starts in; `null` while the button is disabled. */
  path: string | null
  /** The line drawn under the button, `null` when nothing explains it. */
  reason: string | null
  /** The element a disabled button is described by; `null` when it has none. */
  describedBy: string | null
}

/**
 * The button's state from the selected workspace, the first-run empty state, the
 * view's last workspace and the sidebar's own refusal line.
 */
export function newSessionTarget(
  currentPath: string | null,
  empty: boolean,
  last: WelcomeWorkspace_Serialize | null,
  noWorkspaceReason: string,
): NewSessionTarget {
  // A selected workspace wins, and nothing else changes with one (R1).
  if (currentPath !== null) return { path: currentPath, reason: null, describedBy: null }
  // No sessions at all: the empty prompt says why, and it keeps the button's
  // description, exactly as it always did.
  if (empty) return { path: null, reason: null, describedBy: EMPTY_WHY_ID }
  // No last workspace to fall back to: today's disabled button and its line.
  if (last === null) {
    return { path: null, reason: noWorkspaceReason, describedBy: NEW_SESSION_WHY_ID }
  }
  // Available: the very intent a selected workspace sends, and no line to read.
  if (last.available) return { path: last.path, reason: null, describedBy: null }
  // Gone or not a folder: disabled, with Rust's own words verbatim.
  return {
    path: null,
    reason: last.reason ?? noWorkspaceReason,
    describedBy: NEW_SESSION_WHY_ID,
  }
}
