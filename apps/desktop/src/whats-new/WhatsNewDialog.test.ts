// The what's-new modal (T2, S4): the run's own version in the title, the release
// body as plain-text paragraphs, and the one dismiss that tells the shell to
// store the version. Rust owns WHEN the modal exists (`ViewState.whats_new` is
// pushed for exactly one run and cleared by that dismiss), so these tests assert
// only what the view draws and the one command it sends.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import { handlerArgs, resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import type { WhatsNewView } from '../contract/bindings'
import WhatsNewDialog from './WhatsNewDialog.svelte'
import { DISMISS } from './copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

const VERSION = '0.1.8'
const TITLE = "What's new in gentle-desk 0.1.8"
const FIRST = 'The sidebar now names the workspace you are in.'
const SECOND = 'A dropped image can be taken away before it is sent.'
const NOTES = `${FIRST}\n\n${SECOND}`

const DISMISS_COMMAND = 'store_last_run_version'

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  // The command exists server-side; its answer is the push that clears the slice.
  setHandler(DISMISS_COMMAND, () => viewState({ whats_new: null }))
})

/** The modal as Rust pushes it, with the fields a test cares about set. */
function notes(over: Partial<WhatsNewView> = {}): WhatsNewView {
  return { version: VERSION, notes: NOTES, ...over }
}

function draw(whatsNew: WhatsNewView | null): void {
  render(WhatsNewDialog, { props: { whatsNew } })
}

async function modal(): Promise<HTMLElement> {
  return await screen.findByRole('dialog')
}

/** The notes' paragraphs, in the order the modal drew them. */
function paragraphs(dialog: HTMLElement): (string | null)[] {
  return [...dialog.querySelectorAll('p')].map((line) => line.textContent)
}

describe('the what\u2019s-new modal', () => {
  it('names the version in its title and draws the notes as plain paragraphs', async () => {
    draw(notes())
    const dialog = await modal()

    expect(within(dialog).getByText(TITLE)).toBeTruthy()
    expect(paragraphs(dialog)).toEqual([FIRST, SECOND])
  })

  it('draws one paragraph per blank-line chunk of the notes', async () => {
    draw(notes({ notes: `${FIRST}\n\n${SECOND}` }))
    const dialog = await modal()
    expect(paragraphs(dialog)).toHaveLength(2)

    cleanup()
    draw(notes({ notes: FIRST }))
    const single = await modal()
    expect(paragraphs(single)).toEqual([FIRST])
  })

  it('draws no markup a release body could carry, only the text it holds', async () => {
    draw(notes({ notes: '# <b>Bold</b> notes' }))
    const dialog = await modal()

    expect(paragraphs(dialog)).toEqual(['# <b>Bold</b> notes'])
    expect(dialog.querySelector('b')).toBeNull()
    expect(dialog.querySelector('h1')).toBeNull()
  })

  it('opens with the focus on the one answer it has, and keeps it inside', async () => {
    draw(notes())
    const dialog = await modal()

    const dismiss = within(dialog).getByRole('button', { name: DISMISS })
    await vi.waitFor(() => expect(document.activeElement).toBe(dismiss))

    // The focus is trapped: a Tab has nowhere else to go but the one answer.
    await fireEvent.keyDown(dismiss, { key: 'Tab' })
    expect(document.activeElement?.closest('[role="dialog"]')).toBe(dialog)
  })

  it('sends the dismiss once when the button is pressed, and closes', async () => {
    draw(notes())
    const dialog = await modal()

    await fireEvent.click(within(dialog).getByRole('button', { name: DISMISS }))

    await vi.waitFor(() => expect(dialog.isConnected).toBe(false))
    expect(handlerArgs(DISMISS_COMMAND)).toHaveLength(1)
  })

  it('sends the dismiss once on Escape, and a second Escape sends nothing', async () => {
    draw(notes())
    const dialog = await modal()

    await fireEvent.keyDown(document.body, { key: 'Escape' })
    await vi.waitFor(() => expect(dialog.isConnected).toBe(false))
    await fireEvent.keyDown(document.body, { key: 'Escape' })

    expect(handlerArgs(DISMISS_COMMAND)).toHaveLength(1)
  })

  it('sends the dismiss once on a backdrop click', async () => {
    draw(notes())
    const dialog = await modal()

    const overlay = document.querySelector('.ovl')
    expect(overlay).toBeTruthy()
    // jsdom has no PointerEvent, so the click is a MouseEvent that carries the
    // coordinates the primitive checks: a backdrop press lands far from the
    // modal's own box.
    const backdrop = new MouseEvent('pointerdown', {
      bubbles: true,
      cancelable: true,
      clientX: 500,
      clientY: 500,
    })
    await fireEvent(overlay as HTMLElement, backdrop)

    await vi.waitFor(() => expect(dialog.isConnected).toBe(false))
    expect(handlerArgs(DISMISS_COMMAND)).toHaveLength(1)
  })

  it('draws nothing at all while the run follows no update', () => {
    draw(null)

    expect(screen.queryByRole('dialog')).toBeNull()
    expect(document.querySelector('.dlg')).toBeNull()
  })
})
