// The sidebar's words, in one place: the design's own text, quoted from
// design/gentle-desk-v2/00-states-board.html, so the states board and the app
// cannot drift apart silently. Everything here is plain text.

/** The id of the `Select a workspace to start a session.` line. */
export const NEW_SESSION_WHY_ID = 'new-session-why'

/** The id of the empty first-run reason, drawn by the empty state. */
export const EMPTY_WHY_ID = 'workspace-empty-why'

/** Drawn above the tree when no workspace exists yet. */
export const EMPTY_TITLE = 'No workspaces yet'

/** The empty first run's one line: what to do, and why nothing else works. */
export const EMPTY_REASON =
  'Use New workspace to choose a folder or create a new one. A new session needs a workspace.'

/** New session's reason while a workspace exists but none is current (D4). */
export const NO_WORKSPACE_REASON = 'Select a workspace to start a session.'

/** The sidebar header: the section name of the tree. */
export const WORKSPACE_TITLE = 'Workspace'

/**
 * The first-pass line and its counter, mirroring the rule the iced sidebar
 * tests (`crates/ui/src/view/sidebar/status.rs`): no counter before a file has
 * been found, so a zero total never reads like a settled answer.
 */
export function readingLine(done: number, total: number): string {
  return total === 0 ? 'Reading chats...' : `Reading chats... ${done} / ${total}`
}
