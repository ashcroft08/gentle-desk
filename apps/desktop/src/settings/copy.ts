// The Settings dialog's words (settings-dialog S2-S6; 0.1.12 S1-S7). The two
// sections hold the controls the footer used to carry, so their labels are the
// ones those controls already had, and the Shell home section says what the
// welcome screen says about the two homes (shell-choice S2). The other four
// sections are the 0.1.12 panes, each with its own copy file beside it.
// Everything here is plain ASCII.

/** The gear's name and the dialog's title: the same word for both (S5, S2). */
export const SETTINGS = 'Settings'

/**
 * Each section's one-line hint (L4): the stacked dialog said what the two
 * sections were about once, above all of them, and the rail replaces that. Each
 * line now stands over the section it is about, so nothing has to be guessed
 * from a shared line.
 */
export const APPEARANCE_HINT = 'The look of this window.'
export const HOME_HINT = 'Which pi this shell runs on.'
export const UPDATES_HINT = 'What this machine runs, and whether something newer is out.'
export const PROFILES_HINT = 'Your agent profiles, as the terminal keeps them.'
export const ACCOUNT_HINT = 'Which providers are signed in, and where to sign in or out.'
export const ABOUT_HINT = 'Which build this is.'

export const APPEARANCE_TITLE = 'Appearance'
export const HOME_TITLE = 'Shell home'
export const UPDATES_TITLE = 'Updates'
export const PROFILES_TITLE = 'Profiles'
export const ACCOUNT_TITLE = 'Account'
export const ABOUT_TITLE = 'About'

/**
 * The full two-home explanation, the same two answers the welcome screen gives
 * (shell-choice S2) now that the control lives where a restart-needing choice is
 * not clicked by accident. The restart line itself is the home setting's own,
 * drawn while a change waits.
 */
export const HOME_EXPLAIN =
  'My pi uses the pi installation already on the machine: sign-ins, local models and chats appear in the terminal. Separate creates a clean space at ~/.gentle-shell/agent and touches nothing else.'

/** The app's name, as the About section says it (S5): the same name the window's
 * title and the installer carry. */
export const APP_NAME = 'Gentle Shell Desktop'

/** The running version, from the bridge: the one this run runs as (S5). */
export const VERSION = (version: string): string => `Version ${version}`

/** What the version line says while the bridge has named none: never a guess. */
export const VERSION_UNKNOWN = 'Version unknown'

export const CLOSE = 'Close'
