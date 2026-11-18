// The card's words are written in BOTH languages through the thread's own
// dictionary (chat-transcript T2b, the 0.2.0 S3/D4 pattern): the footer's three
// states, the fold button, the duration of a thought and the two payload losses.
// One local language write redraws them in the same tick - no push, no restart -
// while the tool's own name, its command and its output stay exactly as the child
// sent them.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import { tick } from 'svelte'
import type {
  ThinkingItemView_Serialize as Thinking,
  ToolCallItemView_Serialize as ToolCall,
} from '../contract/bindings'
import { resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import { setLanguage } from '../i18n/language.svelte'
import ReasoningRow from './ReasoningRow.svelte'
import ToolCard from './ToolCard.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The child's own words: no language of ours rewrites a command or an output. */
const COMMAND = 'ls node_modules'
const OUTPUT = 'mocker.d.ts'

function tool(over: Partial<ToolCall> = {}): ToolCall {
  return {
    id: 'tool-1',
    tool_name: 'bash',
    args: { command: COMMAND },
    partial_result: null,
    result: OUTPUT,
    is_error: false,
    running: false,
    duration_ms: 200,
    truncated: true,
    dropped: true,
    ...over,
  }
}

function thinking(over: Partial<Thinking> = {}): Thinking {
  return {
    id: 'think-1',
    text: 'weighing the two branches',
    streaming: false,
    duration_ms: 2700,
    ...over,
  }
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
})

describe('the card, drawn in both languages', () => {
  it('flips every word of the card and the reasoning row in the same tick', async () => {
    setLanguage('en')
    const { container } = render(ToolCard, { props: { tool: tool() } })
    render(ReasoningRow, { props: { item: thinking() } })

    expect(container.querySelector('.tc-foot')?.textContent).toContain('Done')
    expect(container.querySelector('.tc-cut')?.textContent).toBe('output cut at the bound')
    expect(screen.getAllByText('output cut at the bound')).toBeTruthy()
    expect(screen.getByText('a payload field was lost')).toBeTruthy()
    expect(screen.getByText('Thought')).toBeTruthy()
    expect(screen.getByText('2.7s')).toBeTruthy()
    // D7: the child's own words are not ours, so they read the same after the flip.
    expect(screen.getByText(COMMAND)).toBeTruthy()
    expect(screen.getByText(OUTPUT)).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(container.querySelector('.tc-foot')?.textContent).toContain('Hecho')
    expect(container.querySelector('.tc-cut')?.textContent).toBe('salida cortada en el límite')
    expect(screen.getByText('se perdió un campo del payload')).toBeTruthy()
    expect(screen.getByText('Pensamiento')).toBeTruthy()
    expect(screen.getByText(COMMAND)).toBeTruthy()
    expect(screen.getByText(OUTPUT)).toBeTruthy()

    setLanguage('en')
  })

  it('flips the running and failed states and the fold count', async () => {
    setLanguage('en')
    const lines = Array.from({ length: 25 }, (_, i) => `line ${i + 1}`)
    const running = render(ToolCard, {
      props: { tool: tool({ running: true, duration_ms: null, result: lines.join('\n') }) },
    })
    const failed = render(ToolCard, {
      props: { tool: tool({ is_error: true, result: 'boom', duration_ms: 12800 }) },
    })

    expect(running.container.querySelector('.tc-foot')?.textContent).toContain('Running')
    expect(screen.getByRole('button', { name: '5 earlier lines' })).toBeTruthy()
    expect(failed.container.querySelector('.tc-foot')?.textContent).toContain('Failed')

    setLanguage('es')
    await tick()

    expect(running.container.querySelector('.tc-foot')?.textContent).toContain('Ejecutando')
    expect(screen.getByRole('button', { name: '5 líneas antes' })).toBeTruthy()
    expect(failed.container.querySelector('.tc-foot')?.textContent).toContain('Falló')

    setLanguage('en')
  })
})
