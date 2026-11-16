// The release body's two languages, found without touching a word of it
// (design/whats-new-modal, artboard D): the maintainer writes one body with the
// news in English and then again under a `Novedades en X.Y.Z` line, so that line
// is where the modal splits it - English is everything before it, Spanish
// everything from it on. Each half then loses its OWN leading section title
// ("What's new in X.Y.Z" and the marker itself), because the artboard's panels
// start at the bullets.
//
// D7 stays whole: the body is the release's content, so this module never
// rewrites or translates a line - it only decides which lines each panel gets.
// No marker (every release before 0.3.0, and any body written in one language)
// means no split: the modal draws the body whole, exactly as it did before the
// tabs. A half that would be empty is treated the same way, so no tab ever opens
// on nothing.
import type { LanguageView } from '../contract/bindings'

/** The line that starts the Spanish half: `Novedades en 0.3.0`. */
const ES_TITLE = /^Novedades en \d+\.\d+\.\d+\b/

/** The English half's own section title: `What's new in 0.3.0`. */
const EN_TITLE = /^What[’']s new in \d+\.\d+\.\d+\b/

/** One body per language, as each panel draws it. */
export type SplitNotes = Record<LanguageView, string>

/**
 * A line's own text: no markdown heading marks, no surrounding space. The
 * marker is matched on this, so `## Novedades en 0.3.0` and a bare line name the
 * same split.
 */
function text(line: string): string {
  return line
    .replace(/^\s*#{1,6}\s*/, '')
    .trim()
}

/**
 * The lines of one half, from the first content line - skipping that half's
 * section title when it is the first thing there.
 */
function half(lines: readonly string[], title: RegExp): string {
  let at = 0
  while (at < lines.length && lines[at].trim() === '') at++
  if (at < lines.length && title.test(text(lines[at]))) at++
  return lines.slice(at).join('\n').trim()
}

/**
 * Splits a release body at its `Novedades en X.Y.Z` marker, dropping each half's
 * leading section title. `null` when the body carries no marker, or when one half
 * would be empty: the modal then draws the whole body in one panel.
 */
export function splitNotes(notes: string): SplitNotes | null {
  const lines = notes.split(/\r?\n/)
  const at = lines.findIndex((line) => ES_TITLE.test(text(line)))
  if (at < 0) return null
  const en = half(lines.slice(0, at), EN_TITLE)
  const es = half(lines.slice(at), ES_TITLE)
  if (en === '' || es === '') return null
  return { en, es }
}
