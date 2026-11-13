// The Account pane's words (0.1.12, S1, T3, L11): provider NAMES and what the
// shell's own check said about each one. No account identity is shown, because
// the shell does not expose one (L11), and no credential ever reaches the page.
// Plain ASCII.

/** The two states the pane draws before it has an answer. */
export const READING = 'Reading the sign-ins...'
export const FAILED = (reason: string): string => `Could not read the sign-ins: ${reason}`
export const RETRY_SAFE = 'Checking again is safe.'
export const NO_PROVIDERS = 'No providers are signed in.'

/** The one check button, and the two terminal hand-offs. */
export const CHECK_AGAIN = 'Check again'
export const LOGIN = 'Open login in terminal'
export const LOGOUT = 'Sign out in terminal'
export const NOTE = 'Login and logout run in a terminal window; this panel shows where you stand.'

/** One provider's check, in the words the pane draws (the state, then its reason). */
export const CONNECTED = 'Connected'
export const UNVERIFIED = 'Signed in (not checkable)'
export const NOT_READY = 'Not ready'
export const INVALID = 'Sign-in is not valid'
export const ERROR = 'Could not check'
