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
 * The two items the New workspace menu draws (01a, and the states board's
 * "New workspace: the menu" card).
 */
export const MENU_CHOOSE = 'Choose an existing folder'
export const MENU_CREATE = 'Create a new folder'

// The create-folder dialog. No artboard draws it, so these words are this
// slice's own; the board's delete sample gives the shape (title, one line,
// Cancel and the action).
export const NEW_FOLDER_TITLE = MENU_CREATE
export const NEW_FOLDER_HINT = 'Enter a name, then choose where the folder goes.'
export const NEW_FOLDER_LABEL = 'Folder name'
export const CREATE_IDLE = 'Create'
export const CREATE_BUSY = 'Creating...'
export const CANCEL = 'Cancel'

/**
 * The first-pass line and its counter, mirroring the rule the iced sidebar
 * tests (`crates/ui/src/view/sidebar/status.rs`): no counter before a file has
 * been found, so a zero total never reads like a settled answer.
 */
export function readingLine(done: number, total: number): string {
  return total === 0 ? 'Reading chats...' : `Reading chats... ${done} / ${total}`
}
