// The transcript a chat now pushes is a list of ROWS, not only messages
// (chat-transcript T2a, L4): a text row, a reasoning row and a tool row, in the
// order the child produced them. This file pins the thread drawing all three -
// and keeps the old shape working, because the shell still hands it `messages`
// alone while the switch is completed.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen, within } from '@testing-library/svelte'
import type {
  ChatItemView_Serialize as Item,
  MessageView_Serialize as MessageView,
} from '../contract/bindings'
import { resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import Thread from './Thread.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

const THREAD = 'Chat thread'

function user(id: string, text: string): MessageView {
  return { id, role: 'User', text, streaming: false, error: null, images: [] }
}

function assistant(id: string, text: string): MessageView {
  return { id, role: 'Assistant', text, streaming: false, error: null, images: [] }
}

function message(data: MessageView): Item {
  return { kind: 'Message', data }
}

function tool(over: Partial<Extract<Item, { kind: 'Tool' }>['data']> = {}): Item {
  return {
    kind: 'Tool',
    data: {
      id: 'tool-call-1',
      tool_name: 'bash',
      args: { command: 'ls node_modules' },
      partial_result: null,
      result: 'mocker.d.ts\nnode.d.ts',
      is_error: false,
      running: false,
      duration_ms: 200,
      truncated: false,
      dropped: false,
      ...over,
    },
  }
}

function thinking(): Item {
  return {
    kind: 'Thinking',
    data: {
      id: 'think-1',
      text: 'OneDrive holds the folder, not the repo.',
      streaming: false,
      duration_ms: 1200,
    },
  }
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
  setHandler('open_link', () => null)
})

describe('the transcript', () => {
  it('draws every row kind the contract sends, in its order', () => {
    const { container } = render(Thread, {
      props: {
        items: [message(user('msg-1', 'why does the ls fail?')), thinking(), tool()],
        foregroundChat: 'chat-1',
      },
    })
    const region = screen.getByRole('region', { name: THREAD })
    const rows = Array.from(container.querySelectorAll('.thread-in > *')).map((row) =>
      row.classList.contains('tool-card')
        ? 'tool'
        : row.classList.contains('think-row')
          ? 'thinking'
          : 'message',
    )

    expect(rows).toEqual(['message', 'thinking', 'tool'])
    // The card is the real one: its command, its body and its footer are drawn.
    expect(within(region).getByText('ls node_modules')).toBeTruthy()
    expect(region.querySelector('.tc-out')?.textContent).toContain('mocker.d.ts')
    expect(region.textContent).toContain('Done 0.2s')
    expect(within(region).getByText('Thought')).toBeTruthy()
    expect(within(region).getByRole('article', { name: 'You said' })).toBeTruthy()
  })

  it('draws a reply that arrives inside the items as markdown, with its Copy', () => {
    const region = (() => {
      render(Thread, {
        props: { items: [message(assistant('msg-1', '# Heading'))], foregroundChat: 'chat-1' },
      })
      return screen.getByRole('region', { name: THREAD })
    })()

    expect(within(region).getByRole('heading', { level: 1 }).textContent).toBe('Heading')
    expect(within(region).getByRole('button', { name: 'Copy' })).toBeTruthy()
  })

  it('still draws the messages it is handed when no row list arrives', () => {
    const region = (() => {
      render(Thread, {
        props: { messages: [user('m1', 'a question'), assistant('m2', 'an answer')], foregroundChat: 'chat-1' },
      })
      return screen.getByRole('region', { name: THREAD })
    })()

    expect(within(region).getByRole('article', { name: 'You said' }).textContent).toContain(
      'a question',
    )
    expect(
      within(region).getByRole('article', { name: 'Assistant said' }).textContent,
    ).toContain('an answer')
    expect(region.querySelector('.tool-card')).toBeNull()
  })

  it('draws the live status while a text row of the list still streams', () => {
    render(Thread, {
      props: {
        items: [message({ ...assistant('msg-1', 'still writing'), streaming: true })],
        foregroundChat: 'chat-1',
      },
    })

    expect(screen.getByRole('status').textContent).toBe('Assistant is replying')
  })

  it('draws no live status for a tool row: a card carries its own state', () => {
    render(Thread, {
      props: {
        items: [tool({ running: true, duration_ms: null, result: null })],
        foregroundChat: 'chat-1',
      },
    })

    expect(screen.queryByRole('status')).toBeNull()
  })
})

describe('the first frame read from the session file (chat-open-latency T1, T2)', () => {
  /** The rows a stored chat's file paints: the core's own `prov-<n>` ids. */
  function fromFile(): Item[] {
    return [message(user('prov-0', 'a pregunta')), message(assistant('prov-1', 'a resposta'))]
  }

  /** The same exchange once the child answered: its own `msg-<n>` ids. */
  function fromChild(): Item[] {
    return [message(user('msg-0', 'a pregunta')), message(assistant('msg-1', 'a resposta'))]
  }

  it('draws the file rows, gives them no Copy, and puts Opening below them', () => {
    render(Thread, {
      props: {
        items: fromFile(),
        foregroundChat: 'chat-1',
        mark: 'Opening',
        provisional: true,
      },
    })
    const region = screen.getByRole('region', { name: THREAD })

    // T2: the thread is never an empty frame while a stored chat opens.
    expect(within(region).getByRole('article', { name: 'You said' }).textContent).toContain(
      'a pregunta',
    )
    expect(
      within(region).getByRole('article', { name: 'Assistant said' }).textContent,
    ).toContain('a resposta')

    // The file's rows are drawn but never ACTED on: no Copy, because a copy is
    // the chat's own and these rows are not (T1, S4).
    expect(within(region).queryByRole('button', { name: 'Copy' })).toBeNull()

    // The existing Opening line, below the rows instead of in an empty thread.
    const opening = within(region).getByRole('status')
    expect(opening.textContent).toContain('Opening chat')
    const last = within(region).getByRole('article', { name: 'Assistant said' })
    expect(last.compareDocumentPosition(opening) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
  })

  it('draws the same text with its Copy the moment the child answers', async () => {
    // The swap is ONE push of the pushed slice - the rows and the flag travel
    // together (view::provisional) - so the thread never draws a frame without
    // them. What this pins is that the rows the answer carries are the rows the
    // file painted, now with the actions a row the chat owns has.
    const { rerender } = render(Thread, {
      props: {
        items: fromFile(),
        messages: fromFile().map((row) => row.data as MessageView),
        foregroundChat: 'chat-1',
        mark: 'Opening',
        provisional: true,
      },
    })
    expect(toArticles(screen.getByRole('region', { name: THREAD }))).toHaveLength(2)

    await rerender({
      items: fromChild(),
      messages: fromChild().map((row) => row.data as MessageView),
      foregroundChat: 'chat-1',
      mark: 'Working',
      provisional: false,
    })
    const region = screen.getByRole('region', { name: THREAD })

    expect(toArticles(region)).toHaveLength(2)
    expect(region.textContent).toContain('a resposta')
    expect(within(region).getAllByRole('button', { name: 'Copy' })).toHaveLength(1)
  })

  it('draws as it always did when the file gave nothing to paint', () => {
    // A fresh chat and an unreadable file are the same slice: no row, no
    // provisional flag, and the Opening line alone.
    render(Thread, {
      props: { items: [], messages: [], foregroundChat: 'chat-1', mark: 'Opening', provisional: false },
    })
    const region = screen.getByRole('region', { name: THREAD })

    expect(toArticles(region)).toHaveLength(0)
    expect(within(region).getByRole('status').textContent).toContain('Opening chat')
  })
})

/** Every drawn message of a thread: the rows a reader counts. */
function toArticles(region: HTMLElement): HTMLElement[] {
  return Array.from(region.querySelectorAll('article'))
}
