// The Updates pane's words (0.1.12, S2, S3, S4, S7): one plain sentence per
// state, so the user never has to know which command runs underneath. The
// versions are the only numbers, and they are drawn quietly under the headline.
// Plain ASCII.

/** The three states that are not a card. */
export const CHECKING = 'Checking...'
export const NOT_CHECKED = 'Not checked yet.'

/** The one check button, and the one action. */
export const CHECK_AGAIN = 'Check again'
export const UPDATE = 'Update'

/** The card's two tags. */
export const UPDATE_AVAILABLE = 'Update available'
export const UP_TO_DATE = 'Everything is up to date'

/** How each piece of this machine is named: no file, no product's file name. */
export const APP_LABEL = 'This app'
export const PI_LABEL = 'pi'
export const SHELL_LABEL = 'shell'

/** What a version the check could not read is drawn as, never a guess. */
export const UNKNOWN_VERSION = 'unknown'

/** The quiet row's separator, the middle dot the house's small notes use. */
export const SEPARATOR = ' \u00b7 '

/** The headline of one row that is behind: what it runs, and what is published. */
export const HEADLINE = (label: string, current: string | null, latest: string | null): string =>
  `${label} ${current ?? UNKNOWN_VERSION} \u2192 ${latest ?? UNKNOWN_VERSION}`

/** The quiet row: the version each of the three pieces runs. */
export const VERSIONS = (lines: string[]): string => lines.join(SEPARATOR)

/** What Update does: it is the terminal's, so the pane says so. */
export const UPDATE_IN_TERMINAL =
  "Update opens a terminal window and runs the shell's own update; read what it says there."

/** The app's own release is not a terminal's job: the banner installs it. */
export const APP_UPDATES_ITSELF = 'This app updates itself: use the update banner at the top.'

/**
 * Why Update is dead while a chat is open (S4): the rule, the count that blocks
 * it, and what makes it come alive.
 */
export function gateHint(openChats: number): string {
  const chats = openChats === 1 ? '1 chat is' : `${openChats} chats are`
  const them = openChats === 1 ? 'it' : 'them'
  return `Updates need every chat closed. ${chats} open - close ${them} and Update comes alive.`
}
