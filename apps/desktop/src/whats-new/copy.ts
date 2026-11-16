// The what's-new modal's words, in one place. The notes arrive from Rust as the
// release body's markdown and are drawn through the thread's sanitized pipeline.
//
// 0.2.0 (S3, D4): the modal's own words - its heading, its one answer, the keyboard
// hint beside it and the "since X" header - are written in BOTH languages here, one
// dictionary per language and a builder that answers the whole object, so the
// parametrized header stays a function and no string-keyed lookup is introduced.
// The English column is what the modal has drawn since T2, unchanged; the Spanish
// one is neutral - no voseo, no regionalisms - and keeps its accents.
//
// design/whats-new-modal artboard D adds one more word here - the language tabs'
// accessible name - and nothing else: the two TAB NAMES are the languages' own
// (`English`, `Español`) and are never translated, so they live with the tabs
// (`tabs.ts`), not in this dictionary.
//
// What does NOT move: the release BODY. It travels in latest.json, so its words are
// the release's, not this app's chrome; only what the modal itself says around it
// is translated (D7).
import type { LanguageView } from '../contract/bindings'

/** The modal's own words for one language. */
export interface WhatsNewCopy {
  /** The modal's heading; the version pill sits beside it in the same title. */
  heading: string
  /** The modal's one answer: it stores the version and clears the notes. */
  dismiss: string
  /** The keyboard hint beside the answer. */
  escHint: string
  /** The language tabs' accessible name: the two tabs are the two readings. */
  tabsLabel: string
  /** "since X" for the header, when `previous` is strictly older than `version`. */
  since: (previous: string) => string
}

/** English: the words the modal already drew. */
const ENGLISH: WhatsNewCopy = {
  heading: "What's new",
  dismiss: 'Got it',
  escHint: 'Esc to close',
  tabsLabel: 'Notes language',
  since: (previous) => `since ${previous}`,
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH: typeof ENGLISH = {
  heading: 'Novedades',
  dismiss: 'Entendido',
  escHint: 'Esc para cerrar',
  tabsLabel: 'Idioma de las novedades',
  since: (previous) => `desde ${previous}`,
}

/** The modal's own words in `language`, the whole dictionary at once (D4). */
export function whatsNewCopy(language: LanguageView): typeof ENGLISH {
  return language === 'es' ? SPANISH : ENGLISH
}

/** The numeric parts of a dotted version, or null when it is not one. */
function parts(version: string): number[] | null {
  if (!/^\d+(\.\d+)*$/.test(version)) return null
  return version.split('.').map(Number)
}

/**
 * "since X" for the header, in the language of `copy`: only when `previous` is a
 * version strictly older than `version`, compared number by number (0.1.9 <
 * 0.1.11). A missing, equal, newer (downgrade) or unparsable previous version says
 * nothing.
 */
export function sinceLabel(
  version: string,
  previous: string | null | undefined,
  copy: WhatsNewCopy,
): string | null {
  const now = parts(version)
  if (previous === null || previous === undefined) return null
  const before = parts(previous)
  if (now === null || before === null) return null
  for (let i = 0; i < Math.max(now.length, before.length); i++) {
    const a = before[i] ?? 0
    const b = now[i] ?? 0
    if (a !== b) return a < b ? copy.since(previous) : null
  }
  return null
}
