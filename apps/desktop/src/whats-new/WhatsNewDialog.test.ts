// The what's-new modal (T2, S4): the run's own version in the title, the release
// body drawn as sanitized markdown, and the one dismiss that tells the shell to
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
const FIRST = 'The sidebar now names the workspace you are in.'
const SECOND = 'A dropped image can be taken away before it is sent.'
// Two spaces before a newline are a markdown hard break: title over description.
const NOTES = ['- **Sidebar**  ', FIRST, '- **Images**  ', SECOND].join('\n')

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

describe('the what’s-new modal', () => {
  it('names the version in its accessible title, with the product line', async () => {
    draw(notes())
    const dialog = await modal()

    const title = within(dialog).getByRole('heading', { name: /What's new.*0\.1\.8/ })
    expect(title.querySelector('.ver')?.textContent).toBe(VERSION)
  })

  it('draws the notes as markdown: bullets are separate items, no markup shows', async () => {
    draw(notes())
    const dialog = await modal()

    const items = [...dialog.querySelectorAll('.notes li')]
    expect(items).toHaveLength(2)
    expect(items[0].querySelector('strong')?.textContent).toBe('Sidebar')
    expect(items[1].textContent).toContain(SECOND)
    expect(dialog.textContent).not.toContain('**')
    expect(dialog.textContent).not.toContain('##')
  })

  it('keeps a leading heading, drawn as a small section label', async () => {
    draw(notes({ notes: `## Highlights

- one` }))
    const dialog = await modal()

    expect(dialog.querySelector('.notes h2')?.textContent).toBe('Highlights')
    expect(dialog.textContent).not.toContain('##')
  })

  it('draws no active markup a release body could carry', async () => {
    draw(notes({ notes: 'hi <script>window.x=1</script> <img src="http://x/y.png"> **b**' }))
    const dialog = await modal()

    expect(dialog.querySelector('script')).toBeNull()
    expect(dialog.querySelector('img')).toBeNull()
    expect(dialog.querySelector('.notes strong')?.textContent).toBe('b')
  })

  it('says "since X" only when the previous version is strictly older', async () => {
    draw(notes({ version: '0.1.11', previous_version: '0.1.9' }))
    expect((await modal()).querySelector('.since')?.textContent).toBe('since 0.1.9')

    for (const previous of [null, undefined, '0.1.11', '0.2.0', '0.1.12', 'garbage']) {
      cleanup()
      draw(notes({ version: '0.1.11', previous_version: previous }))
      const dialog = await modal()
      expect(dialog.querySelector('.since')).toBeNull()
    }
  })

  it('compares versions numerically, not as text', async () => {
    draw(notes({ version: '0.1.11', previous_version: '0.1.9' }))
    expect((await modal()).querySelector('.since')).toBeTruthy()
    cleanup()
    draw(notes({ version: '0.1.9', previous_version: '0.1.11' }))
    expect((await modal()).querySelector('.since')).toBeNull()
  })

  it('shows the Esc hint beside the one answer', async () => {
    draw(notes())
    expect(within(await modal()).getByText('Esc to close')).toBeTruthy()
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
