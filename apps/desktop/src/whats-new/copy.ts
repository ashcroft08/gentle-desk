// The what's-new modal's words, in one place: the title carries the version this
// run runs as, and the notes arrive from Rust as plain text (S4). No locale, no
// markup: the modal decides how the release body is laid out.

/** The title, with the run's own version (S5: never the manifest's). */
export const WHATS_NEW_TITLE = (version: string): string =>
  `What's new in gentle-desk ${version}`

/** The modal's one answer: it stores the version and clears the notes. */
export const DISMISS = 'Got it'

/**
 * The notes split into paragraphs on blank lines. A release body separates its
 * sections with one, and the modal draws one paragraph per chunk as plain text,
 * so no markup of the body ever becomes markup in the app.
 */
export function paragraphsOf(notes: string): string[] {
  return notes
    .split(/\r?\n\s*\r?\n/)
    .map((chunk) => chunk.trim())
    .filter((chunk) => chunk !== '')
}
