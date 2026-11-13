// The Account pane's words (0.1.12, S1, T3, L11): provider NAMES and what the
// shell's own check said about each one. No account identity is shown, because
// the shell does not expose one (L11), and no credential ever reaches the page.
// Plain ASCII.

/** The two states the pane draws before it has an answer. */
export const READING = 'Reading the sign-ins...'
export const FAILED = (reason: string): string => `Could not read the sign-ins: ${reason}`
export const RETRY_SAFE = 'Checking again is safe.'
export const NO_PROVIDERS = 'No providers are signed in.'

/** The one check button, and the one terminal hand-off. */
export const CHECK_AGAIN = 'Check again'
export const OPEN_SHELL = 'Open shell to sign in or out'
export const NOTE =
  'The shell opens in a terminal window: type /login to sign in or /logout to sign out, then press Check again here.'

/** One provider's check, in the words the pane draws (the state, then its reason). */
export const CONNECTED = 'Connected'
export const UNVERIFIED = 'Signed in (not checkable)'
export const NOT_READY = 'Not ready'
export const INVALID = 'Sign-in is not valid'
export const ERROR = 'Could not check'
