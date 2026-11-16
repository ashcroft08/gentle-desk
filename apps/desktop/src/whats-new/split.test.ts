// The split of a release body into its two languages (design/whats-new-modal,
// artboard D): the body is ONE text written in English then Spanish, and the
// modal finds the Spanish half at its `Novedades en X.Y.Z` marker. This file
// pins the rule itself; what a DRAWN modal does with the two halves is pinned in
// `language-tabs.test.ts`.
//
// D7: the body is the release's, never rewritten here - the split only decides
// which of its parts a panel draws, and each panel drops its own leading section
// title (the panels start at the bullets, as the artboard draws them).
import { describe, expect, it } from 'vitest'
import { splitNotes } from './split'

const EN_BULLET = '- **Remove a workspace, cleanly.** gentle-desk asks before it acts.'
const ES_BULLET = '- **Quitá un workspace con lógica.** gentle-desk pregunta antes de actuar.'

/** A body as the maintainer writes it: one title, its bullets, then the other. */
const BODY = [
  "## What's new in 0.3.0",
  '',
  EN_BULLET,
  '',
  '## Novedades en 0.3.0',
  '',
  ES_BULLET,
].join('\n')

describe('splitting a release body into its two languages', () => {
  it('splits at the Novedades marker, and starts each half at the bullets', () => {
    expect(splitNotes(BODY)).toEqual({ en: EN_BULLET, es: ES_BULLET })
  })

  it('finds the marker with or without the markdown heading marks around it', () => {
    for (const marker of ['Novedades en 0.3.0', '## Novedades en 0.3.0', '  ### Novedades en 0.3.0  ']) {
      expect(splitNotes(`- one\n${marker}\n- dos`), marker).toEqual({ en: '- one', es: '- dos' })
    }
  })

  it('only takes a marker that names a full version', () => {
    // A line that mentions the marker mid-sentence, or names no dotted version,
    // is content: the body stays whole rather than being cut where it should not.
    for (const line of [
      'See Novedades en 0.3.0 for the Spanish.',
      'Novedades en la versión 0.3.0',
      'Novedades en 0.3',
      'Novedades 0.3.0',
      'notes',
    ]) {
      expect(splitNotes(`- one\n${line}\n- two`), line).toBeNull()
    }
  })

  it('answers null when the body carries no marker: one panel, as before', () => {
    expect(splitNotes(`## Highlights\n\n- one`)).toBeNull()
    expect(splitNotes('')).toBeNull()
  })

  it('answers null when one side would be empty, so no tab opens on nothing', () => {
    expect(splitNotes('## Novedades en 0.3.0\n\n- dos')).toBeNull()
    expect(splitNotes('- one\n\n## Novedades en 0.3.0')).toBeNull()
  })

  it('keeps the release body as it was written, apart from the split', () => {
    const split = splitNotes(BODY)
    expect(split?.en).toContain('**Remove a workspace, cleanly.**')
    expect(split?.es).toContain('**Quitá un workspace con lógica.**')
  })
})
