// The composer's words: the artboard's own text where it draws one
// (design/gentle-desk-v2/01a-tree-chat.html for the box, 00-states-board.html for
// what each state says), so the design and the app cannot drift apart silently.
//
// The two reasons are this slice's own words: the states board names the
// composer's behaviour per state ("Send disabled until the chat is ready") but
// draws no line, so the text below says the same thing in the form the app can
// show, and nothing more. A queued chat needs none: it shows Stop, as iced does.

/** The prompt box's accessible name (the artboard's `aria-label`). */
export const MESSAGE_LABEL = 'Message'

/** The box's placeholder (the artboard's `placeholder`). */
export const MESSAGE_PLACEHOLDER = 'Tell Gentle what you need...'

/** The line under the box that lists the three keys the box answers to. */
export const KEYS_HINT = 'Enter to send \u00b7 Shift+Enter for a new line \u00b7 Esc to stop'

/** The two buttons, each its own accessible name (the artboard's). */
export const SEND = 'Send'
export const STOP = 'Stop'

/** The id of the line that says why Send is disabled; Send is described by it. */
export const SEND_WHY_ID = 'composer-send-why'

/** The tallest the box grows before it scrolls (the artboard's `max-height`). */
export const MAX_HEIGHT = 120

/** Opening: the chat's process is coming up and cannot take a prompt yet. */
export const OPENING_REASON = 'Send is available once the chat is ready.'

/** What the composer says while files are dragged over the window: the whole
 * window is the target, and the composer is where they land. */
export const DROP_HINT = 'Drop the images here to attach them.'

/** The accessible name of the button that takes one attached image away; the
 * drawn name adds the file it takes. */
export const REMOVE_IMAGE = 'Remove'

/** The custom dropdown's filter box and its empty result (S7). */
export const SEARCH_PLACEHOLDER = 'Search models...'
export const NOTHING_MATCHES = 'Nothing matches.'

/** The sidebar collapse toggle's accessible name (responsive-sidebar S1). */
export const SIDEBAR_TOGGLE = 'Toggle sidebar'

/** Ended: the chat's process is gone until it is restarted. */
export const ENDED_REASON = "This chat's process exited. Restart it to keep talking."

/** The model chip's two accessible names (0.1.11). */
export const MODEL_LABEL = 'Model'
export const LEVEL_LABEL = 'Thinking level'

/** The chip's states, in the words the child's own truth takes. */
export const MODELS_LOADING = 'Loading models...'
export const MODELS_EMPTY = 'No models are available for this chat.'
export const MODELS_UNAVAILABLE = 'Models unavailable:'

// --- the context chip (0.1.12, S5) ---

/**
 * The chip's accessible name: the number, and what the number is a share of.
 * The text is the state, so the colour never carries it alone.
 */
export const CONTEXT_LABEL = (percent: number): string =>
  `Context used: ${percent} percent of the model's window`

/** What the chip says when pi reported no percentage: right after a compaction. */
export const CONTEXT_DASH = '\u2013'

/** The chip's name when there is no percentage to read (the dash state). */
export const CONTEXT_HIDDEN = 'Context usage is not known right now'

/**
 * A token count as the hover reads it: `38k`, `200k`, `1.2m`. Rounded, because
 * the detail is a context, not a measurement.
 */
export function formatTokens(count: number): string {
  if (count >= 1_000_000) return `${Math.round(count / 100_000) / 10}m`
  if (count >= 1_000) return `${Math.round(count / 1_000)}k`
  return String(count)
}

/**
 * The chip's hover text: the tokens the context holds against the window it
 * fills. Either number can be missing (pi answers null right after a
 * compaction), and a missing one is simply not said.
 */
export function tokensTitle(tokens: number | null, window: number | null): string {
  if (tokens === null) return ''
  if (window === null) return `${formatTokens(tokens)} tokens`
  return `${formatTokens(tokens)} / ${formatTokens(window)} tokens`
}
