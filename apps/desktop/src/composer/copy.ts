// The composer's words: the artboard's own text where it draws one
// (design/gentle-desk-v2/01a-tree-chat.html for the box, 00-states-board.html for
// what each state says), so the design and the app cannot drift apart silently.
//
// The two reasons are this slice's own words: the states board names the
// composer's behaviour per state ("Send disabled until the chat is ready") but
// draws no line, so the text below says the same thing in the form the app can
// show, and nothing more. A queued chat needs none: it shows Stop, as iced does.
//
// 0.2.0 (S3, D4): those words are written in BOTH languages here - one dictionary
// per language and a builder that answers the whole object, so a parametrized line
// (the context reading) stays a function and no string-keyed lookup is introduced.
// The English column is what the composer has drawn since the artboard, unchanged;
// the Spanish one is neutral - no voseo, no regionalisms - and keeps its accents.
//
// What does NOT move: the model names and ids, the thinking level names and the
// reason the model list failed, which are the child's own truth and arrive exactly
// as written (D7). Four values stay outside the dictionary because none of them is
// a word: the DOM id the reason line is named by, the tallest the box grows, the
// dash the context chip draws and the token detail (`tokens` is the same word in
// both languages, so it is a format, not copy).
import type { LanguageView } from '../contract/bindings'

/** The composer's own words for one language. */
export interface ComposerCopy {
  /** The prompt box's accessible name and its placeholder (the artboard's). */
  messageLabel: string
  messagePlaceholder: string
  /** The line under the box that lists the three keys it answers to. */
  keysHint: string
  /**
   * The same line while the chat works or waits for a slot: Enter sends
   * nothing there (one turn at a time), so the hint says it is off instead of
   * promising a send the composer will not do.
   */
  keysHintWorking: string
  /** The two buttons, each its own accessible name. */
  send: string
  stop: string
  /** Why Send is disabled: opening, and ended. */
  openingReason: string
  endedReason: string
  /** What the box says while files are dragged over the window. */
  dropHint: string
  /** The accessible name of the button that takes one attached image away. */
  removeImage: string
  /** The custom dropdown's filter box and its empty result (S7). */
  searchPlaceholder: string
  nothingMatches: string
  /** The sidebar collapse toggle's accessible name (responsive-sidebar S1). */
  sidebarToggle: string
  /** The model chip's two accessible names (0.1.11). */
  modelLabel: string
  levelLabel: string
  /** The chip's states, in the words the child's own truth takes. */
  modelsLoading: string
  modelsEmpty: string
  modelsUnavailable: string
  /**
   * The context chip's accessible name (0.1.12, S5): the number, and what the
   * number is a share of. The text is the state, so colour never carries it.
   */
  contextLabel: (percent: number) => string
  /** The chip's name when there is no percentage to read (the dash state). */
  contextHidden: string
}

/** English: the words the composer already drew. */
const ENGLISH: ComposerCopy = {
  messageLabel: 'Message',
  messagePlaceholder: 'Tell Gentle what you need...',
  keysHint: 'Enter to send \u00b7 Shift+Enter for a new line \u00b7 Esc to stop',
  keysHintWorking:
    'Enter is off while Gentle works \u00b7 Shift+Enter for a new line \u00b7 Esc to stop',
  send: 'Send',
  stop: 'Stop',
  openingReason: 'Send is available once the chat is ready.',
  endedReason: "This chat's process exited. Restart it to keep talking.",
  dropHint: 'Drop the images here to attach them.',
  removeImage: 'Remove',
  searchPlaceholder: 'Search models...',
  nothingMatches: 'Nothing matches.',
  sidebarToggle: 'Toggle sidebar',
  modelLabel: 'Model',
  levelLabel: 'Thinking level',
  modelsLoading: 'Loading models...',
  modelsEmpty: 'No models are available for this chat.',
  modelsUnavailable: 'Models unavailable:',
  contextLabel: (percent) => `Context used: ${percent} percent of the model's window`,
  contextHidden: 'Context usage is not known right now',
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH: typeof ENGLISH = {
  messageLabel: 'Mensaje',
  messagePlaceholder: 'Cuéntale a Gentle lo que necesitas...',
  // The three key names are the keyboard's own, so they are not translated.
  keysHint: 'Enter para enviar \u00b7 Shift+Enter para una línea nueva \u00b7 Esc para detener',
  keysHintWorking:
    'Enter no envía mientras Gentle trabaja \u00b7 Shift+Enter para una línea nueva \u00b7 Esc para detener',
  send: 'Enviar',
  stop: 'Detener',
  openingReason: 'Enviar estará disponible cuando el chat esté listo.',
  endedReason: 'El proceso de este chat terminó. Reinícialo para seguir hablando.',
  dropHint: 'Suelta las imágenes aquí para adjuntarlas.',
  removeImage: 'Quitar',
  searchPlaceholder: 'Buscar modelos...',
  nothingMatches: 'No hay coincidencias.',
  sidebarToggle: 'Mostrar u ocultar la barra lateral',
  modelLabel: 'Modelo',
  levelLabel: 'Nivel de pensamiento',
  modelsLoading: 'Cargando modelos...',
  modelsEmpty: 'No hay modelos disponibles para este chat.',
  modelsUnavailable: 'Modelos no disponibles:',
  contextLabel: (percent) => `Contexto usado: ${percent} por ciento de la ventana del modelo`,
  contextHidden: 'El uso del contexto no se conoce ahora mismo',
}

/** The composer's own words in `language`, the whole dictionary at once (D4). */
export function composerCopy(language: LanguageView): typeof ENGLISH {
  return language === 'es' ? SPANISH : ENGLISH
}

/** The id of the line that says why Send is disabled; Send is described by it. */
export const SEND_WHY_ID = 'composer-send-why'

/** The tallest the box grows before it scrolls (the artboard's `max-height`). */
export const MAX_HEIGHT = 120

/** What the chip says when pi reported no percentage: right after a compaction. */
export const CONTEXT_DASH = '\u2013'

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
 * compaction), and a missing one is simply not said. `tokens` is the same word
 * in both languages, so this line is a format and stays outside the dictionary.
 */
export function tokensTitle(tokens: number | null, window: number | null): string {
  if (tokens === null) return ''
  if (window === null) return `${formatTokens(tokens)} tokens`
  return `${formatTokens(tokens)} / ${formatTokens(window)} tokens`
}
