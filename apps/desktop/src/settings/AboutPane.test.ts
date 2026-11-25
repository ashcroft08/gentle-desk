// The About pane (settings-redesign S9, L5, L6, L7): the maintainer's "aquí lo
// siento muy vacío" answered with the real facts only - the app and its version,
// the two installed versions the Updates read ALREADY knows, when that read last
// ran (D8), and the repository - never an invented number and never a new read.
//
// The what's-new button (D6, D7, T5's read): the pane owns the click, asks the
// shell for the notes of the version this run RUNS as, and draws NOTHING when
// they were shown - the notes travel in `ViewState.whats_new` and the existing
// modal opens by itself. The only thing this pane draws of its own is the one
// quiet line a read that had nothing to show answers with.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, within } from '@testing-library/svelte'
import type { UpdatesView_Serialize as UpdatesView } from '../contract/bindings'
import { deferred, handlerArgs, resetTauriMock, setHandler } from '../test-support/tauri-mock'
import { setLanguage } from '../i18n/language.svelte'
import AboutPane from './AboutPane.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The repository the README names: the one address this pane links to. */
const REPOSITORY = 'https://github.com/ashcroft08/gentle-desk'

/** The one command the button sends, wired to T5's read of the running version. */
const WHATS_NEW_COMMAND = 'show_whats_new'
/** The button's own words, as the dictionary of the language drawn spells them. */
const WHATS_NEW = 'What is new in this version'

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

    packages: { available: false, behind: [], unknown: false, truncated: false, unchecked: false, checked: null },
    checked_ms: Date.now() - 3 * 60_000,
    ...over,
  }
}

function draw(
  version: string | null = '0.8.1',
  updates: UpdatesView | null = read(),
  onShown: () => void = () => {},
): HTMLElement {
  const { container } = render(AboutPane, { props: { version, updates, onShown } })
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

  // D6, D7: the button is always there now that the read exists, and it is the
  // button ALONE - no three-line note under it, and nothing else clickable.
  it('draws the one what-is-new button and no note under it', () => {
    const pane = draw()

    const buttons = within(pane).getAllByRole('button')
    expect(buttons).toHaveLength(1)
    expect(buttons[0].textContent?.trim()).toBe(WHATS_NEW)
    expect(pane.textContent).not.toContain('Opens the window')
  })

  // `Shown`: the notes reached `ViewState.whats_new` and the modal is up, so the
  // pane adds nothing to the screen - no line, no modal of its own.
  it('asks for the notes once on a click, and draws nothing when they are shown', async () => {
    setHandler(WHATS_NEW_COMMAND, () => ({ type: 'Shown' }))
    const pane = draw()

    await fireEvent.click(within(pane).getByRole('button', { name: WHATS_NEW }))

    expect(handlerArgs(WHATS_NEW_COMMAND)).toHaveLength(1)
    await vi.waitFor(() => expect(pane.querySelector('.quiet')).toBeNull())
    expect(within(pane).queryByRole('dialog')).toBeNull()
  })

  // The what's-new window's backdrop sits under the Settings card (both share the
  // app's overlay layers), so once the notes are on screen the dialog is told, and
  // closes itself. A read that showed nothing leaves it open with its quiet line.
  it('tells the dialog once the notes are shown, and never when they could not be', async () => {
    const shown = vi.fn()
    setHandler(WHATS_NEW_COMMAND, () => ({ type: 'Shown' }))
    const pane = draw('0.8.1', read(), shown)
    await fireEvent.click(within(pane).getByRole('button', { name: WHATS_NEW }))
    await vi.waitFor(() => expect(shown).toHaveBeenCalledTimes(1))

    const none = vi.fn()
    setHandler(WHATS_NEW_COMMAND, () => ({ type: 'Unavailable', data: 'No notes.' }))
    const other = draw('0.8.1', read(), none)
    await fireEvent.click(within(other).getByRole('button', { name: WHATS_NEW }))
    await vi.waitFor(() => expect(other.querySelector('.quiet')?.textContent).toBe('No notes.'))
    expect(none).not.toHaveBeenCalled()
  })

  // `Unavailable(line)`: nothing was pushed, the modal stays closed, and the
  // line - already worded by the shell in the app's language - is the pane's
  // whole answer. A new press clears it BEFORE the answer arrives.
  it('draws the shell quiet line when nothing could be shown, and clears it on the next press', async () => {
    setHandler(WHATS_NEW_COMMAND, () => ({ type: 'Unavailable', data: 'The notes could not be read.' }))
    const pane = draw()

    await fireEvent.click(within(pane).getByRole('button', { name: WHATS_NEW }))

    await vi.waitFor(() =>
      expect(pane.querySelector('.quiet')?.textContent).toBe('The notes could not be read.'),
    )

    const next = deferred<unknown>()
    setHandler(WHATS_NEW_COMMAND, () => next.promise)
    await fireEvent.click(within(pane).getByRole('button', { name: WHATS_NEW }))

    await vi.waitFor(() => expect(pane.querySelector('.quiet')).toBeNull())

    // The second answer says the notes were shown: still no line of the pane's.
    next.resolve({ type: 'Shown' })
    await vi.waitFor(() => expect(handlerArgs(WHATS_NEW_COMMAND)).toHaveLength(2))
    expect(pane.querySelector('.quiet')).toBeNull()
  })

  it('disables the button while the read is in flight, so a second press asks nothing', async () => {
    const answer = deferred<unknown>()
    setHandler(WHATS_NEW_COMMAND, () => answer.promise)
    const pane = draw()
    const button = within(pane).getByRole('button', { name: WHATS_NEW }) as HTMLButtonElement

    await fireEvent.click(button)
    expect(button.disabled).toBe(true)
    await fireEvent.click(button)
    expect(handlerArgs(WHATS_NEW_COMMAND)).toHaveLength(1)

    answer.resolve({ type: 'Shown' })
    await vi.waitFor(() => expect(button.disabled).toBe(false))
  })

  it('flips every word it owns, and never a version or the address', async () => {
    const pane = draw()

    setLanguage('es')
    await vi.waitFor(() => expect(pane.textContent).toContain('Versión 0.8.1'))

    expect(pane.textContent).toContain('Repositorio')
    expect(pane.textContent).toContain('Última comprobación: hace 3m')
    // The button is there in both languages too (D6).
    expect(pane.textContent).toContain('Novedades de esta versión')
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
