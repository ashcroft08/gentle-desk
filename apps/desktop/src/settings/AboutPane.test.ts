// The About pane (settings-redesign S9, L5, L6, L7): the maintainer's "aquí lo
// siento muy vacío" answered with the real facts only - the app and its version,
// the two installed versions the Updates read ALREADY knows, when that read last
// ran (D8), and the repository - never an invented number and never a new read.
//
// The what's-new button is deliberately NOT wired yet: it is drawn only when the
// dialog hands it an action (T5's Rust read lands first, D6), so this run shows
// no dead control and the label waits in the copy for the run that wires it.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, within } from '@testing-library/svelte'
import type { UpdatesView_Serialize as UpdatesView } from '../contract/bindings'
import { handlerArgs, resetTauriMock, setHandler } from '../test-support/tauri-mock'
import { setLanguage } from '../i18n/language.svelte'
import AboutPane from './AboutPane.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The repository the README names: the one address this pane links to. */
const REPOSITORY = 'https://github.com/ashcroft08/gentle-desk'

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('open_link', () => null)
  setLanguage('en')
})

/** The Updates read the dialog already holds, as the pane receives it. */
function read(over: Partial<UpdatesView> = {}): UpdatesView {
  return {
    status: 'Ready',
    desk: { current: '0.8.1', latest: '0.8.1', update_available: false },
    pi: { current: '4.8.0', latest: '4.8.0', update_available: false },
    shell: { current: '4.8.0', latest: '4.8.0', update_available: false },
    checked_ms: Date.now() - 3 * 60_000,
    ...over,
  }
}

function draw(
  version: string | null = '0.8.1',
  updates: UpdatesView | null = read(),
  onWhatsNew?: () => void,
): HTMLElement {
  const { container } = render(AboutPane, { props: { version, updates, onWhatsNew } })
  return container as HTMLElement
}

/** A row of the facts list, by its label, as the two columns draw it. */
function fact(pane: HTMLElement, label: string): { term: string; value: string } {
  const row = [...pane.querySelectorAll('.afact')].find(
    (candidate) => candidate.querySelector('dt')?.textContent?.trim() === label,
  )
  if (row === undefined) throw new Error(`no row for ${label}`)
  return {
    term: label,
    value: row.querySelector('dd')?.textContent?.trim() ?? '',
  }
}

describe('the About pane', () => {
  it('draws the app and the version this run runs as, as one title block', () => {
    const pane = draw()

    expect(pane.querySelector('.atitle .aname')?.textContent).toBe('Gentle Shell Desktop')
    expect(pane.querySelector('.atitle .aver')?.textContent).toBe('Version 0.8.1')
  })

  it('draws the two installed versions the Updates read already knew', () => {
    const pane = draw()

    // The rows are named by the Updates pane's own dictionary - one word, one
    // owner - and carry the versions that read reported, verbatim.
    expect(fact(pane, 'Gentle Shell').value).toBe('4.8.0')
    expect(fact(pane, 'pi').value).toBe('4.8.0')
  })

  it('says unknown, never a guess, when the read named no version', () => {
    const pane = draw('0.8.1', read({
      pi: { current: null, latest: null, update_available: null },
      shell: { current: null, latest: null, update_available: null },
    }))

    expect(fact(pane, 'Gentle Shell').value).toBe('unknown')
    expect(fact(pane, 'pi').value).toBe('unknown')

    // And before any read answered at all, both rows say the same word.
    cleanup()
    const none = draw('0.8.1', null)
    expect(fact(none, 'Gentle Shell').value).toBe('unknown')
    expect(fact(none, 'pi').value).toBe('unknown')
  })

  // D8: the same Updates read names when it last ran, in the age rule the pane
  // and the welcome already share (home/ago.ts), never a second formatter.
  it('says when the same read last ran, and never when it has not', () => {
    expect(draw().textContent).toContain('Last checked: 3m ago')
    expect(draw('0.8.1', null).textContent).toContain('Last checked: never')
    expect(draw('0.8.1', read({ checked_ms: null })).textContent).toContain(
      'Last checked: never',
    )
  })

  it('links the repository through the app own external-link path', async () => {
    const pane = draw()

    expect(fact(pane, 'Repository').value).toBe('github.com/ashcroft08/gentle-desk')
    const link = within(pane).getByRole('link', { name: 'github.com/ashcroft08/gentle-desk' })
    expect(link.getAttribute('href')).toBe(REPOSITORY)

    const click = new MouseEvent('click', { bubbles: true, cancelable: true })
    link.dispatchEvent(click)

    // The view never navigates: the click is cancelled and the shell opens it.
    expect(click.defaultPrevented).toBe(true)
    await vi.waitFor(() => expect(handlerArgs('open_link')).toEqual([{ url: REPOSITORY }]))
  })

  // D7: no note under the button, and no button either while nothing can answer
  // its click - a dead control is worse than an absent one.
  it('draws no what-is-new button until the dialog can open the window', () => {
    const pane = draw()

    expect(within(pane).queryByRole('button')).toBeNull()
    expect(pane.textContent).not.toContain('What is new in this version')
    expect(pane.textContent).not.toContain('Opens the window')
  })

  it('draws that one button, and only it, once the dialog wires it', async () => {
    const whatsNew = vi.fn()
    const pane = draw('0.8.1', read(), whatsNew)

    await fireEvent.click(within(pane).getByRole('button', { name: 'What is new in this version' }))

    expect(whatsNew).toHaveBeenCalledTimes(1)
  })

  it('flips every word it owns, and never a version or the address', async () => {
    const pane = draw()

    setLanguage('es')
    await vi.waitFor(() => expect(pane.textContent).toContain('Versión 0.8.1'))

    expect(pane.textContent).toContain('Repositorio')
    expect(pane.textContent).toContain('Última comprobación: hace 3m')
    // D7: the versions and the address are the shell's own readings.
    expect(fact(pane, 'Gentle Shell').value).toBe('4.8.0')
    expect(pane.textContent).toContain('github.com/ashcroft08/gentle-desk')
  })

  it('says desconocida in Spanish when the read named no version', async () => {
    const pane = draw('0.8.1', null)

    setLanguage('es')
    await vi.waitFor(() => expect(pane.textContent).toContain('desconocida'))

    expect(pane.textContent).toContain('Última comprobación: nunca')
  })
})
