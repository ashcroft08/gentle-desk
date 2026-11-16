// The what's-new modal's language tabs (design/whats-new-modal, artboard D): one
// panel per language, the tab of the language the app holds selected first, and
// the release body split at its `Novedades en X.Y.Z` marker (D7: the body is the
// release's - the modal only decides how it lays it out).
//
// The folder-tab ANATOMY is pinned here through what the view draws (which
// element is the tablist, which tab is selected, what the panel is named by);
// the rule blocks that give it its shape are pinned in `stylesheet.test.ts`.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import { tick } from 'svelte'
import type { WhatsNewView } from '../contract/bindings'
import { resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import { setLanguage } from '../i18n/language.svelte'
import WhatsNewDialog from './WhatsNewDialog.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

const VERSION = '0.3.0'

/** The two section titles the maintainer writes above each half. */
const EN_TITLE = "What's new in 0.3.0"
const ES_TITLE = 'Novedades en 0.3.0'

/** One bullet of each half: the same news, written in each language. */
const EN_BULLET = 'Remove a workspace, cleanly.'
const ES_BULLET = 'Quitá un workspace con lógica.'

/** The release body as it arrives from latest.json: English, then Spanish. */
const BILINGUAL = [
  `## ${EN_TITLE}`,
  '',
  `- **${EN_BULLET}** gentle-desk asks before it acts.`,
  '',
  `## ${ES_TITLE}`,
  '',
  `- **${ES_BULLET}** gentle-desk pregunta antes de actuar.`,
].join('\n')

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setLanguage('en')
  setHandler('store_last_run_version', () => viewState({ whats_new: null }))
})

/** The modal as Rust pushes it, drawing `notes` unless a test says otherwise. */
function draw(over: Partial<WhatsNewView> = {}): void {
  render(WhatsNewDialog, { props: { whatsNew: { version: VERSION, notes: BILINGUAL, ...over } } })
}

/** The name of the tab the reader is reading, as `aria-selected` reports it. */
function selectedTab(dialog: HTMLElement): string {
  return within(dialog).getAllByRole('tab').filter((tab) => tab.getAttribute('aria-selected') === 'true')[0]
    ?.textContent ?? ''
}

describe("the what's-new modal's language tabs", () => {
  it('splits the body in two, opens on the app’s language and drops each section title', async () => {
    setLanguage('es')
    draw()
    const dialog = await screen.findByRole('dialog')

    const labels = within(dialog)
      .getAllByRole('tab')
      .map((tab) => tab.textContent?.trim())
    expect(labels).toEqual(['English', 'Español'])
    expect(selectedTab(dialog).trim()).toBe('Español')
    expect(within(dialog).getByRole('tab', { name: 'English' }).getAttribute('aria-selected')).toBe(
      'false',
    )

    // The artboard's panels start at the bullets: each half loses its own title.
    const panel = within(dialog).getByRole('tabpanel')
    expect(panel.textContent).toContain(ES_BULLET)
    expect(panel.textContent).not.toContain(EN_BULLET)
    expect(panel.querySelector('h2')).toBeNull()
    expect(dialog.textContent).not.toContain(ES_TITLE)
    expect(dialog.textContent).not.toContain(EN_TITLE)
  })

  it('draws no tabs when the body carries no marker, and keeps the whole body as it was', async () => {
    draw({ notes: ['## Highlights', '', '- one', '- two'].join('\n') })
    const dialog = await screen.findByRole('dialog')

    expect(within(dialog).queryByRole('tablist')).toBeNull()
    expect(within(dialog).queryAllByRole('tab')).toHaveLength(0)

    // Exactly the single panel the modal drew before the tabs: no tab semantics,
    // every bullet, and the leading heading kept as its small section label.
    const panel = dialog.querySelector('.notes')
    expect(panel?.getAttribute('role')).toBeNull()
    expect(panel?.querySelectorAll('li')).toHaveLength(2)
    expect(panel?.querySelector('h2')?.textContent).toBe('Highlights')
  })

  it('swaps the panels when the other tab is clicked, and names the panel after it', async () => {
    setLanguage('es')
    draw()
    const dialog = await screen.findByRole('dialog')

    const english = within(dialog).getByRole('tab', { name: 'English' })
    await fireEvent.click(english)

    const panel = within(dialog).getByRole('tabpanel')
    expect(panel.textContent).toContain(EN_BULLET)
    expect(panel.textContent).not.toContain(ES_BULLET)
    expect(english.getAttribute('aria-selected')).toBe('true')
    expect(selectedTab(dialog).trim()).toBe('English')
    // The panel speaks for the tab that opened it, and only that tab is a stop.
    expect(panel.getAttribute('aria-labelledby')).toBe(english.id)
    expect(english.getAttribute('tabindex')).toBe('0')
    expect(within(dialog).getByRole('tab', { name: 'Español' }).getAttribute('tabindex')).toBe('-1')
  })

  it('draws one folder tab per language, each named in its own language', async () => {
    draw()
    const dialog = await screen.findByRole('dialog')

    const row = within(dialog).getByRole('tablist')
    expect(row.className).toBe('tabs')
    expect(row.getAttribute('aria-label')).toBe('Notes language')

    const [english, spanish] = within(dialog).getAllByRole('tab')
    // The names are the languages' own, in the language the app holds too.
    expect(english.textContent?.trim()).toBe('English')
    expect(spanish.textContent?.trim()).toBe('Español')
    expect(english.getAttribute('lang')).toBe('en')
    expect(spanish.getAttribute('lang')).toBe('es')
    // One component: both tabs point at the same panel, which the row opens.
    expect(english.getAttribute('aria-controls')).toBe(within(dialog).getByRole('tabpanel').id)
    expect(spanish.getAttribute('aria-controls')).toBe(english.getAttribute('aria-controls'))
  })

  it('keeps the reading the reader chose while the same version is pushed again', async () => {
    setLanguage('es')
    const { rerender } = render(WhatsNewDialog, {
      props: { whatsNew: { version: VERSION, notes: BILINGUAL } },
    })
    const dialog = await screen.findByRole('dialog')
    await fireEvent.click(within(dialog).getByRole('tab', { name: 'English' }))
    expect(within(dialog).getByRole('tabpanel').textContent).toContain(EN_BULLET)

    // Any other slice (a chat delta, a rename) arrives as a NEW view state: the
    // tab the reader chose does not move under them.
    await rerender({ whatsNew: { version: VERSION, notes: BILINGUAL } })
    expect(selectedTab(dialog).trim()).toBe('English')
    expect(within(dialog).getByRole('tabpanel').textContent).toContain(EN_BULLET)

    // A NEW version's notes are another reading: they open on the app's language.
    await rerender({ whatsNew: { version: '0.4.0', notes: BILINGUAL } })
    expect(selectedTab(dialog).trim()).toBe('Español')
  })

  it('walks the two tabs with the arrow keys, the focus walk the settings rail uses', async () => {
    draw()
    const dialog = await screen.findByRole('dialog')
    await tick()

    const [english, spanish] = within(dialog).getAllByRole('tab')
    english.focus()

    await fireEvent.keyDown(english, { key: 'ArrowRight' })
    expect(document.activeElement).toBe(spanish)
    expect(selectedTab(dialog).trim()).toBe('Español')
    expect(within(dialog).getByRole('tabpanel').textContent).toContain(ES_BULLET)

    await fireEvent.keyDown(spanish, { key: 'Home' })
    expect(document.activeElement).toBe(english)
    expect(within(dialog).getByRole('tabpanel').textContent).toContain(EN_BULLET)

    // A key that is not a walk keeps its own meaning: the reading does not move.
    await fireEvent.keyDown(english, { key: 'a' })
    expect(selectedTab(dialog).trim()).toBe('English')
  })
})
