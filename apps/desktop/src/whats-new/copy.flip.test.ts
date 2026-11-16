// A what's-new modal DRAWN from its dictionary (0.2.0, S2, S3): one local
// language write redraws the heading, the "since X" header, the hint and the one
// answer in the same tick - no push, no restart (S2) - while the release BODY
// keeps reading exactly as latest.json carried it (D7).
//
// The English words are asserted literally: they are what the modal has drawn
// since T2. The two columns themselves are pinned in `copy.test.ts`.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import { tick } from 'svelte'
import type { WhatsNewView } from '../contract/bindings'
import { resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import { setLanguage } from '../i18n/language.svelte'
import WhatsNewDialog from './WhatsNewDialog.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The release's own line, in English because the release wrote it (D7). */
const BODY = 'The sidebar now names the workspace you are in.'

/** The push the modal draws: a version, the one before it, and the body. */
const NOTES: WhatsNewView = {
  version: '0.2.0',
  previous_version: '0.1.14',
  notes: `- **Sidebar** ${BODY}`,
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('store_last_run_version', () => viewState({ whats_new: null }))
})

describe("a drawn what's-new modal", () => {
  it('flips its heading, its since line, its hint and its answer in the same tick', async () => {
    setLanguage('en')
    render(WhatsNewDialog, { props: { whatsNew: NOTES } })

    expect(screen.getByText("What's new")).toBeTruthy()
    expect(screen.getByText('since 0.1.14')).toBeTruthy()
    expect(screen.getByText('Esc to close')).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Got it' })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(screen.getByText('Novedades')).toBeTruthy()
    expect(screen.getByText('desde 0.1.14')).toBeTruthy()
    expect(screen.getByText('Esc para cerrar')).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Entendido' })).toBeTruthy()
    // D7: the release body is the release's, so it reads as it was written.
    expect(screen.getByText(BODY)).toBeTruthy()

    setLanguage('en')
  })
})
