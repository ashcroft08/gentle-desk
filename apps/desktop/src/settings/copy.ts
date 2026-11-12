// The Settings dialog's words (settings-dialog S2-S6). The two sections hold the
// controls the footer used to carry, so their labels are the ones those controls
// already had, and the Shell home section says what the welcome screen says about
// the two homes (shell-choice S2). The third section carries the app's name and
// version and the manual update check. Everything here is plain ASCII.

/** The gear's name and the dialog's title: the same word for both (S5, S2). */
export const SETTINGS = 'Settings'

/** The dialog's one line under its title: what the two sections are about. */
export const HINT = 'The look of this window, and the home gentle shell runs on.'

export const APPEARANCE_TITLE = 'Appearance'
export const HOME_TITLE = 'Shell home'

/**
 * The full two-home explanation, the same two answers the welcome screen gives
 * (shell-choice S2) now that the control lives where a restart-needing choice is
 * not clicked by accident. The restart line itself is the home setting's own,
 * drawn while a change waits.
 */
export const HOME_EXPLAIN =
  'My pi uses the pi installation already on the machine: sign-ins, local models and chats appear in the terminal. Separate creates a clean space at ~/.gentle-shell/agent and touches nothing else.'

/** The third section's title: it carries the app's own information and the
 * manual update check (settings-dialog S5, S6). */
export const ABOUT_TITLE = 'About and updates'

/** The app's name, as the About section says it (S5): the same name the window's
 * title and the installer carry. */
export const APP_NAME = 'Gentle Shell Desktop'

/** The running version, from the bridge: the one this run runs as (S5). */
export const VERSION = (version: string): string => `Version ${version}`

/** What the version line says while the bridge has named none: never a guess. */
export const VERSION_UNKNOWN = 'Version unknown'

/** The check and what it answered (S6): the section draws the answer in place. */
export const CHECK_NOW = 'Check now'
export const CHECKING = 'Checking for updates...'
export const UP_TO_DATE = 'You are up to date.'
export const CHECK_AVAILABLE = (version: string): string => `Version ${version} is available.`
export const CHECK_FAILED = (message: string): string =>
  `The update check failed: ${message}`

export const CLOSE = 'Close'
