// The Usage pane's words (settings-0.1.12 S6's deferred half). English, plain,
// and no file name of anyone's: the user is told what the numbers are, where
// they come from, and what the one button does. Plain ASCII.
//
// The rule the pane repeats wherever it can: a provider without credentials is
// drawn as "no data", never as a used-up plan.

/** Before the first answer, and while the read runs (the shell's own sources). */
export const READING = 'Reading the usage...'

/**
 * The one reason line the pane draws, for both the read that could not happen
 * (something was unreadable) and the answer it could not use (a run that failed,
 * a document of another shape): the shell's own words follow the colon, so the
 * sentence never repeats them.
 */
export const CANNOT_READ = (reason: string): string => `Could not read the usage: ${reason}`

/** Nothing asked the sources anything yet (the section was never opened). */
export const NOT_READ = 'The usage has not been read yet.'

/** The package is not there: one line, the install button and the retry. */
export const MISSING = 'The usage sources are not installed.'
export const MISSING_WHY =
  'They are the shell\'s own sources: the same ones its subscription panel reads. This app never installs anything silently, so the button opens a terminal window with the install command and you decide there.'
export const INSTALL = 'Install in terminal'
export const INSTALL_NOTE =
  "Install opens a terminal window and runs the shell's own installer; read what it says there."

/** The two actions every state offers, when they apply. */
export const CHECK_AGAIN = 'Check again'
export const RETRY_SAFE = 'Checking again is safe.'

/** One provider's own words: its plan, a no-data answer, and a reached limit. */
export const PLAN = (plan: string): string => `Plan: ${plan}`
export const NO_DATA = 'No data'
export const NO_DATA_WHY = 'A provider without credentials reports no data, never 0%.'
export const LIMIT_REACHED = 'Limit reached'
export const PROVIDER_FAILED = 'Could not read this provider'

/** The sources answered, and named no provider at all. */
export const NO_PROVIDERS = 'The usage sources are not reporting any provider right now.'

/** The quiet line under the cards: what the numbers are and are not. */
export const NOTE =
  "These numbers come from the shell's own sources, the same ones its subscription panel reads: one row per window, with the share used and when it resets."
