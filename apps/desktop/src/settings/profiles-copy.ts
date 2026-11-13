// The Profiles pane's words (0.1.12, S1, T2, L9). Read-only: every edit happens
// in the terminal, so the pane's own copy says what it shows and where the
// writing happens. Plain ASCII, except the middle dot the rest of the app's
// small notes already use.

/** The switch's own label, and the one line under it that keeps it honest. */
export const MODE_LABEL = 'Orchestrator mode'
export const MODE_ORGANIC = 'Organic'
export const MODE_FULL_DELEGATION = 'Full delegation'
export const MODE_NOTE = 'For chats opened from now on \u00b7 not stored'

/** The two routes the summary block draws, and the table's three columns. */
export const ORCHESTRATOR_LABEL = 'Orchestrator'
export const FALLBACK_LABEL = 'Fallback'
export const AGENT_COLUMN = 'Agent'
export const MODEL_COLUMN = 'Model'
export const EFFORT_COLUMN = 'Effort'

/** The status line of the active profile (L11: the active one only). */
export const ACTIVE_NOTE = 'Default for new sessions'

/** The pane's three states, in the order the pushes walk them. */
export const READING = 'Reading profiles...'
export const NO_PROFILES = 'No profiles found.'
export const FAILED = (reason: string): string => `The profiles could not be read: ${reason}`

/** What a retry means, and the one button that does it. */
export const RETRY_SAFE = 'Reading them again is safe.'
export const TRY_AGAIN = 'Try again'

/** The one action, and the note that says it opens a terminal. */
export const EDIT_IN_TERMINAL = 'Edit in terminal'
export const EDIT_NOTE =
  'Edits, creates and switches profiles - the panel refreshes when the terminal closes.'
