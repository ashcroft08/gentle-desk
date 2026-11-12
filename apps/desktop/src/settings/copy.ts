// The Settings dialog's words (settings-dialog S2-S4). The two sections hold the
// controls the footer used to carry, so their labels are the ones those controls
// already had, and the Shell home section says what the welcome screen says about
// the two homes (shell-choice S2). Everything here is plain ASCII.

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

/** The third section's title: its About and Updates fields are T2's. */
export const ABOUT_TITLE = 'About and updates'

export const CLOSE = 'Close'
