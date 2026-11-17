// The helpers panel's drawn words and the two rules the card needs: which chat
// mark stands for a helper status, and the one line that sums up what a helper
// is doing. The status label, the task title and the counters line all come
// from Rust (`gentle_runtime::helper_text`), so nothing here re-spells them.
//
// 0.2.0 (S3, D4): the words this view owns are written in BOTH languages here -
// one dictionary per language and a builder that answers the whole object, so a
// parametrized entry stays a function and no string-keyed lookup is introduced -
// and a drawn panel redraws them in the same tick it is told the new language
// (S2). The English column is what the panel drew before 0.2.0, unchanged; the
// Spanish one is neutral - no voseo, no regionalisms - and keeps its accents.
//
// What does NOT move: `markOfStatus` (a mapping, no words), `DOT` (a drawn
// separator, not a word) and `threadHeading`, which only joins the agent and the
// task title Rust sent with iced's ` - `: child content and a neutral separator
// are shown as they arrive (D7).

import type { HelperStatusView, LanguageView, ToolStatusView } from '../contract/bindings'
import type { MarkKind } from '../icons/shapes'

/**
 * The separator between the parts of a card's line: the one iced's card uses
 * (U+00B7). Written as an escape so the source stays ASCII, and drawn, never an
 * icon.
 */
export const DOT = '\u00b7'

/** The helpers view's own words for one language. */
export interface HelpersCopy {
  /** The listbox's accessible name, as 01b writes it. */
  listLabel: string
  /** The thread's accessible name, so the region says what it is. */
  threadLabel: string
  /** What an open chat with no delegated helper says (T7b-1 decision). */
  noHelpers: string
  /**
   * The Back control of a helper's thread. 01b draws the arrow with the words
   * "Back to chat", but the control returns to the helpers list (iced's
   * CloseHelper), so the name follows what it does, not what 01b's prototype
   * did with it.
   */
  back: string
  /** The thread bar's stop square, tip and first word (helper-stop S4). */
  stop: string
  /** The same square once the helper is no longer running: its tip. */
  stopped: string
  /**
   * The stop square's accessible name while the helper runs, naming the helper
   * it would stop (`Stop <agent> - <task>`, 01b's own condition).
   */
  stopNamed: (helper: string) => string
  /**
   * The accessible name once that helper is already stopped, so a reader learns
   * the helper's name and that there is nothing left to stop.
   */
  alreadyStopped: (helper: string) => string
  /** What a helper whose thread holds no item says: iced's own line. */
  noActivity: string
  /** "2 turns <dot> 1 tool calls", the counts iced's card spells out. */
  cardMeta: (turns: number, toolCalls: number) => string
  /**
   * The line above a thread whose first items were left out: iced's own wording
   * (`crates/ui/src/view/helper_thread.rs`), never a count re-derived here.
   */
  droppedLine: (dropped: number) => string
  /** The word one tool call's status is drawn with: iced's own three words. */
  toolStatus: (status: ToolStatusView) => string
}

/** English: the words the panel already drew. */
const ENGLISH: HelpersCopy = {
  listLabel: 'Helpers',
  threadLabel: 'Helper thread',
  noHelpers: 'No helpers for this chat',
  back: 'Back to helpers',
  stop: 'Stop',
  stopped: 'Stopped',
  stopNamed: (helper) => `Stop ${helper}`,
  alreadyStopped: (helper) => `${helper} already stopped`,
  noActivity: 'No activity yet',
  cardMeta: (turns, toolCalls) => `${turns} turns ${DOT} ${toolCalls} tool calls`,
  droppedLine: (dropped) => `${dropped} earlier items were dropped`,
  toolStatus: (status) => {
    switch (status) {
      case 'Running':
        return 'running'
      case 'Failed':
        return 'failed'
      case 'Ok':
        return 'ok'
    }
  },
}

/**
 * Neutral Spanish: no voseo, no regionalisms, nothing left in English. Spanish
 * marks the singular in the two counted lines; the English column keeps iced's
 * own uninflected wording, which is what it has always drawn.
 */
const SPANISH: typeof ENGLISH = {
  listLabel: 'Ayudantes',
  threadLabel: 'Hilo del ayudante',
  noHelpers: 'Este chat no tiene ayudantes',
  back: 'Volver a los ayudantes',
  stop: 'Detener',
  stopped: 'Detenido',
  stopNamed: (helper) => `Detener ${helper}`,
  alreadyStopped: (helper) => `${helper} ya está detenido`,
  noActivity: 'Todavía no hay actividad',
  cardMeta: (turns, toolCalls) =>
    `${turns} turnos ${DOT} ${toolCalls} ${
      toolCalls === 1 ? 'llamada a herramienta' : 'llamadas a herramienta'
    }`,
  droppedLine: (dropped) =>
    dropped === 1
      ? `Se descartó ${dropped} elemento anterior`
      : `Se descartaron ${dropped} elementos anteriores`,
  toolStatus: (status) => {
    switch (status) {
      case 'Running':
        return 'en ejecución'
      case 'Failed':
        return 'falló'
      case 'Ok':
        return 'correcto'
    }
  },
}

/** The view's own words in `language`, the whole dictionary at once (D4). */
export function helpersCopy(language: LanguageView): typeof ENGLISH {
  return language === 'es' ? SPANISH : ENGLISH
}

/**
 * The chat mark that stands for a helper status (approved T7b-1 mapping). The
 * shape is only the picture: the card's accessible name for it is the status
 * label Rust sent, so the text, not the colour, carries the state.
 */
export function markOfStatus(status: HelperStatusView): MarkKind {
  switch (status) {
    case 'Running':
      return 'Working'
    case 'Queued':
    case 'Waiting':
      return 'Queued'
    case 'Done':
      return 'NewReply'
    case 'Failed':
    case 'Cancelled':
      return 'Ended'
  }
}

/**
 * The thread bar's headline: "explore - Inventory of sidebar modules", as 01b
 * writes it, and just the task title for a task that names no agent. Neither
 * part is ours - the agent and the task title come from Rust (D7) - and ` - ` is
 * a neutral separator, so this is the same line in every language.
 */
export function threadHeading(agent: string, title: string): string {
  const name = agent.trim()
  return name === '' ? title : `${name} - ${title}`
}
