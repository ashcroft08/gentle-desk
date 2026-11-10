// The thread's drawn rules, one test per rule: a user message is plain text, an
// assistant reply is markdown, an errored reply is the danger box instead of the
// reply, Copy belongs to a finished reply only, a link is forwarded and never
// navigated to, and the only live region is the small "replying" status.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import type { MessageView_Serialize as MessageView } from '../contract/bindings'
import { handlerArgs, resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import Thread from './Thread.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The words of the thread, quoted so a change of copy is a change here. */
const THREAD = 'Chat thread'
const YOU_SAID = 'You said'
const ASSISTANT_SAID = 'Assistant said'
const COPY = 'Copy'
const REPLYING = 'Assistant is replying'
const MODEL_ERROR = 'Model error'

function user(id: string, text: string): MessageView {
  return { id, role: 'User', text, streaming: false, error: null, images: [] }
}

function assistant(id: string, text: string, over: Partial<MessageView> = {}): MessageView {
  return { id, role: 'Assistant', text, streaming: false, error: null, images: [], ...over }
}

/** Draws the thread and returns its scroll region. */
function draw(messages: MessageView[]): HTMLElement {
  render(Thread, { props: { messages, foregroundChat: 'chat-1' } })
  return screen.getByRole('region', { name: THREAD })
}

/** The intents the thread sent, oldest first. */
function intents(): unknown[] {
  return handlerArgs('dispatch_intent').map((args) => args.intent)
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
  setHandler('open_link', () => null)
})

describe('Thread activity line', () => {
  it('draws Working under the last message while a turn runs', () => {
    render(Thread, {
      props: { messages: [user('u1', 'hi')], foregroundChat: 'chat-1', mark: 'Working' },
    })

    expect(screen.getByText('Working')).toBeTruthy()
  })

  it('draws Opening chat in an empty thread while the chat opens', () => {
    render(Thread, { props: { messages: [], foregroundChat: 'chat-1', mark: 'Opening' } })

    expect(screen.getByRole('status').textContent).toContain('Opening chat')
  })

  it('draws no line once the chat is idle', () => {
    render(Thread, { props: { messages: [user('u1', 'hi')], foregroundChat: 'chat-1', mark: null } })

    expect(screen.queryByText('Working')).toBeNull()
  })
})

describe('Thread', () => {
  it('draws a user message as plain text, never as markdown', () => {
    const region = draw([user('m1', '# not a heading\n\n**not bold**')])
    const article = within(region).getByRole('article', { name: YOU_SAID })

    expect(article.querySelector('h1')).toBeNull()
    expect(article.querySelector('strong')).toBeNull()
    expect(article.textContent).toContain('# not a heading')
    expect(article.textContent).toContain('**not bold**')
  })

  it('draws an assistant reply as markdown: headings, lists, code and links', () => {
    const reply = [
      '# Heading',
      '- first\n- second',
      '`inline` and:',
      '```ts',
      'const x = 1',
      '```',
      '[the notes](https://example.com/a)',
    ].join('\n\n')
    const region = draw([assistant('m1', reply)])
    const article = within(region).getByRole('article', { name: ASSISTANT_SAID })

    expect(article.querySelector('h1')?.textContent).toBe('Heading')
    expect(article.querySelectorAll('li')).toHaveLength(2)
    expect(article.querySelector('p code')?.textContent).toBe('inline')
    expect(article.querySelector('pre code')?.textContent).toContain('const x = 1')
    expect(within(article).getByRole('link').getAttribute('href')).toBe('https://example.com/a')
  })

  it('draws a model error as a danger box with the error, not the reply', () => {
    const error = 'The provider stopped with stopReason: error.'
    const region = draw([assistant('m1', 'a reply that never arrived', { error })])

    expect(within(region).getByText(MODEL_ERROR)).toBeTruthy()
    expect(within(region).getByText(error)).toBeTruthy()
    expect(region.textContent).not.toContain('a reply that never arrived')
  })

  it('offers Copy only on a finished reply, and sends the message id once', async () => {
    const region = draw([
      user('m1', 'a question'),
      assistant('m2', 'still writing', { streaming: true }),
      assistant('m3', 'a finished reply'),
      assistant('m4', 'a broken reply', { error: 'stopReason: error' }),
    ])
    const copies = within(region).getAllByRole('button', { name: COPY })

    expect(copies).toHaveLength(1)
    const owner = copies[0].closest('article')
    expect(owner?.textContent).toContain('a finished reply')
    expect(owner?.textContent).not.toContain('still writing')

    await fireEvent.click(copies[0])
    expect(intents()).toEqual([{ type: 'CopyMessage', data: { message_id: 'm3' } }])
  })

  it('forwards a link once and never lets the view navigate itself', async () => {
    const region = draw([assistant('m1', '[the notes](https://example.com/a?b=c)')])
    const link = within(region).getByRole('link')

    // A cancelled event is what keeps WebView2 from navigating.
    expect(await fireEvent.click(link)).toBe(false)
    expect(handlerArgs('open_link')).toEqual([{ url: 'https://example.com/a?b=c' }])
  })

  it('intercepts ctrl+click and middle click the same way', async () => {
    const region = draw([assistant('m1', '[the notes](https://example.com/a)')])
    const link = within(region).getByRole('link')

    await fireEvent.click(link, { ctrlKey: true })
    // `auxclick` is what a middle click dispatches, and the DOM testing library
    // has no shorthand for it.
    fireEvent(link, new MouseEvent('auxclick', { bubbles: true, cancelable: true, button: 1 }))
    expect(handlerArgs('open_link')).toEqual([
      { url: 'https://example.com/a' },
      { url: 'https://example.com/a' },
    ])
  })

  it('never forwards an empty or javascript: href', async () => {
    const region = draw([assistant('m1', '[js](javascript:alert(1)) and [empty]()')])
    const anchors = Array.from(region.querySelectorAll('a'))

    expect(anchors.length).toBeGreaterThanOrEqual(1)
    for (const anchor of anchors) {
      expect(anchor.getAttribute('href') ?? '').not.toContain('javascript:')
      expect(await fireEvent.click(anchor)).toBe(false)
    }
    expect(handlerArgs('open_link')).toEqual([])
  })

  it('keeps one small live status while a message streams', () => {
    const region = draw([assistant('m1', 'still writing', { streaming: true })])

    expect(within(region).getByRole('status').textContent).toBe(REPLYING)
    // The reply itself is never a live region: a screen reader must not re-read
    // the whole answer on every delta.
    expect(region.querySelectorAll('[aria-live]')).toHaveLength(1)
    expect(
      within(region).getByRole('article', { name: ASSISTANT_SAID }).closest('[aria-live]'),
    ).toBeNull()
  })

  it('has no live status once nothing streams', () => {
    const region = draw([assistant('m1', 'a finished reply')])

    expect(within(region).queryByRole('status')).toBeNull()
    expect(region.querySelectorAll('[aria-live]')).toHaveLength(0)
  })

  it('names every message article after its author', () => {
    const region = draw([user('m1', 'a question'), assistant('m2', 'an answer')])

    expect(within(region).getByRole('article', { name: YOU_SAID })).toBeTruthy()
    expect(within(region).getByRole('article', { name: ASSISTANT_SAID })).toBeTruthy()
  })
})
