// One helper's thread, one test per rule: the header iced echoes (name, status
// as the mark's accessible name, prompt), every item kind, markdown through the
// thread's own renderer with its sanitizer and link handling, the dropped and
// empty lines, the focus the thread takes, the keys it ignores, and the end it
// follows.

import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/svelte'
import type { HelperThreadView as HelperThreadModel } from '../contract/bindings'
import {
  helperThread,
  noteItem,
  ranItem,
  saidItem,
  thoughtItem,
} from '../test-support/helper-thread'
import { handlerArgs, resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import HelperThread from './HelperThread.svelte'
import {
  BACK_TO_HELPERS,
  HELPER_THREAD_LABEL,
  NO_ACTIVITY,
  droppedLine,
} from './copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** Draws one thread and answers its region plus the Back spy. */
function draw(thread: HelperThreadModel = helperThread()) {
  const onBack = vi.fn()
  const view = render(HelperThread, { props: { thread, onBack } })
  const region = screen.getByRole('region', { name: HELPER_THREAD_LABEL })
  /** The scroll box the items live in, which jsdom cannot lay out. */
  const body = region.querySelector('.body') as HTMLElement
  return { onBack, region, body, rerender: view.rerender }
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
  setHandler('open_link', () => null)
})

describe('HelperThread', () => {
  it('names the helper by its agent and task, marks its status and echoes the prompt', () => {
    const { region } = draw(
      helperThread({
        agent: 'verify',
        title: 'Contract tests for the bridge',
        status: 'Waiting',
        status_label: 'waiting',
        prompt: 'Run the contract tests.',
      }),
    )

    expect(within(region).getByRole('heading').textContent).toBe(
      'verify - Contract tests for the bridge',
    )
    // The status word, not the colour, is the mark's accessible name.
    expect(within(region).getByRole('img', { name: 'waiting' })).toBeTruthy()
    expect(within(region).getByText('Run the contract tests.')).toBeTruthy()
  })

  it('draws what the helper said as markdown: bold, inline code and links', () => {
    const { region } = draw(
      helperThread({ items: [saidItem('The **bridge** is `thin`.\n\n- one\n- two')] }),
    )

    expect(region.querySelector('strong')?.textContent).toBe('bridge')
    expect(region.querySelector('p code')?.textContent).toBe('thin')
    expect(region.querySelectorAll('li')).toHaveLength(2)
  })

  it('neutralizes a hostile payload instead of drawing it', () => {
    const payload = '<script>alert(1)</script><img src=x onerror="alert(2)"> plain'
    const { region } = draw(helperThread({ items: [saidItem(payload)] }))

    expect(region.querySelector('script')).toBeNull()
    expect(region.querySelector('img')).toBeNull()
    expect(region.innerHTML).not.toContain('onerror')
    expect(region.innerHTML).not.toContain('alert(1)')
    expect(region.textContent).toContain('plain')
  })

  it('forwards a link of the helper own words once, never navigating itself', async () => {
    const { region } = draw(helperThread({ items: [saidItem('[the notes](https://example.com/a)')] }))

    const link = within(region).getByRole('link')
    expect(await fireEvent.click(link)).toBe(false)

    expect(handlerArgs('open_link')).toEqual([{ url: 'https://example.com/a' }])
  })

  it('draws a thought and a note as plain muted text, never as markdown', () => {
    const { region } = draw(
      helperThread({ items: [thoughtItem('**which** modules move'), noteItem('**left** a TODO')] }),
    )

    expect(within(region).getByText('**which** modules move')).toBeTruthy()
    expect(within(region).getByText('**left** a TODO')).toBeTruthy()
    expect(region.querySelector('strong')).toBeNull()
  })

  it('draws a tool call with its name, its status word, its arguments and its output', () => {
    const { region } = draw(
      helperThread({
        items: [
          ranItem('read', { status: 'Ok', args_line: 'path=rpc.md', output_preview: 'opened the file' }),
          ranItem('grep', { status: 'Running', args_line: 'pattern=Intent', output_preview: '' }),
          ranItem('bash', { status: 'Failed', args_line: 'cmd=cargo test', output_preview: '' }),
        ],
      }),
    )

    // The word carries the state; the colour only repeats it.
    expect(within(region).getByText('ok', { selector: '.status' })).toBeTruthy()
    expect(within(region).getByText('running', { selector: '.status' })).toBeTruthy()
    expect(within(region).getByText('failed', { selector: '.status' })).toBeTruthy()
    expect(within(region).getByText('read')).toBeTruthy()
    expect(within(region).getByText('path=rpc.md')).toBeTruthy()
    expect(within(region).getByText('opened the file')).toBeTruthy()
    expect(within(region).getByText('cmd=cargo test')).toBeTruthy()
    // Only the tool with an output preview draws one: an empty preview draws
    // no monospaced block at all.
    expect(region.querySelectorAll('.detail.out')).toHaveLength(1)
  })

  it('says how many earlier items were left out, with iced own words', () => {
    const { region } = draw(helperThread({ dropped: 3, items: [saidItem('the tail')] }))

    expect(within(region).getByText(droppedLine(3)).textContent).toBe(
      '3 earlier items were dropped',
    )
  })

  it('says a helper with an empty thread has no activity yet', () => {
    const { region } = draw()

    expect(within(region).getByText(NO_ACTIVITY).textContent).toBe('No activity yet')
  })

  it('takes the focus with its Back control, the only control of the thread', () => {
    draw()

    expect(document.activeElement).toBe(screen.getByRole('button', { name: BACK_TO_HELPERS }))
  })

  it('sends nothing on Escape: iced leaves a thread from its Back control only', async () => {
    const { region, onBack } = draw()

    await fireEvent.keyDown(region, { key: 'Escape' })

    expect(onBack).not.toHaveBeenCalled()
    expect(handlerArgs('dispatch_intent')).toEqual([])
    expect(screen.getByRole('region', { name: HELPER_THREAD_LABEL })).toBeTruthy()
  })

  it('goes back once when its Back control is pressed', async () => {
    const { onBack } = draw()

    await fireEvent.click(screen.getByRole('button', { name: BACK_TO_HELPERS }))

    expect(onBack).toHaveBeenCalledTimes(1)
  })

  it('follows the pushed end while the reader is at the end, and leaves them be when not', async () => {
    const onBack = vi.fn()
    const view = render(HelperThread, {
      props: { thread: helperThread({ items: [saidItem('first')] }), onBack },
    })
    const region = screen.getByRole('region', { name: HELPER_THREAD_LABEL })
    const body = region.querySelector('.body') as HTMLElement
    // jsdom lays nothing out, so the box reports its numbers the way a tall
    // thread does: more content than it shows.
    Object.defineProperty(body, 'scrollHeight', { value: 1000, configurable: true })
    Object.defineProperty(body, 'clientHeight', { value: 100, configurable: true })

    // The reader scrolled up: a pushed update must not move them.
    body.scrollTop = 100
    await fireEvent.scroll(body)
    await view.rerender({
      thread: helperThread({ items: [saidItem('first'), saidItem('second')] }),
      onBack,
    })
    await waitFor(() => expect(body.scrollTop).toBe(100))

    // The reader is at the end: the update follows it.
    body.scrollTop = 900
    await fireEvent.scroll(body)
    Object.defineProperty(body, 'scrollHeight', { value: 1200, configurable: true })
    await view.rerender({
      thread: helperThread({
        items: [saidItem('first'), saidItem('second'), saidItem('third')],
      }),
      onBack,
    })
    await waitFor(() => expect(body.scrollTop).toBe(1100))
  })
})
