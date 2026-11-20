// One tool call, drawn as ONE card (chat-transcript T2b, S3): the header (glyph,
// the tool's name, a chip only when the name implies a language, the chevron that
// folds the body), the monospace subject, the folded output whose head says how
// many lines are hidden, and the footer that carries the state and the elapsed
// time - a check and "Done 0.2s", a spinner and "Running", an X and "Failed 12.8s".
//
// Two more rules this file pins because the card is the only thing that can say
// them: the first line of a failed call IS its error line (never an empty body),
// and the two payload losses are worded apart - the output was cut at the bound,
// or a field of it was dropped - so the card never draws a loss as a whole.
import { afterEach, describe, expect, it } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import type { ToolCallItemView_Serialize as ToolCall } from '../contract/bindings'
import ToolCard from './ToolCard.svelte'
import { FOLD_LINES } from './card'

/** One tool call as Rust pushes it, with the fields a test does not set defaulted. */
function tool(over: Partial<ToolCall> = {}): ToolCall {
  return {
    id: 'tool-1',
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
  }
}

/** The card's footer, where the state and the time live. */
function foot(container: HTMLElement): HTMLElement {
  const element = container.querySelector<HTMLElement>('.tc-foot')
  if (element === null) throw new Error('the card drew no footer')
  return element
}

/** One line of elements, whitespace collapsed, so a footer reads as one phrase. */
function words(element: HTMLElement): string {
  return (element.textContent ?? '').replace(/\s+/g, ' ').trim()
}

afterEach(cleanup)

describe('a completed tool card', () => {
  it('draws its tool, its command, its folded body and "Done <n>s"', () => {
    const { container } = render(ToolCard, { props: { tool: tool() } })

    expect(container.querySelector('.tc-tool')?.textContent?.trim()).toBe('bash')
    expect(container.querySelector('.tc-chip')?.textContent).toBe('bash')
    expect(container.querySelector('.tc-cmd')?.textContent).toBe('ls node_modules')
    expect(container.querySelector('.tc-cmd')?.classList.contains('shell')).toBe(true)
    expect(container.querySelector('.tc-out')?.textContent).toContain('mocker.d.ts')
    expect(container.querySelector('.tc-out')?.textContent).toContain('node.d.ts')
    expect(words(foot(container))).toBe('Done 0.2s')
    expect(foot(container).querySelector('.tc-spin')).toBeNull()
  })

  it('chips the language the tool’s name names, and none it does not', () => {
    const bash = render(ToolCard, { props: { tool: tool() } })
    expect(bash.container.querySelector('.tc-chip')?.textContent).toBe('bash')
    cleanup()

    const read = render(ToolCard, {
      props: { tool: tool({ tool_name: 'read', args: { path: 'src/app.css' }, result: '' }) },
    })
    expect(read.container.querySelector('.tc-chip')).toBeNull()
    expect(read.container.querySelector('.tc-cmd')?.textContent).toBe('read({"path":"src/app.css"})')
  })

  it('folds the body and says how many earlier lines are hidden', async () => {
    const lines = Array.from({ length: 34 }, (_, i) => `line ${i + 1}`)
    const { container } = render(ToolCard, {
      props: { tool: tool({ result: lines.join('\n') }) },
    })
    const body = container.querySelector('.tc-out')
    expect(body?.querySelectorAll('.tc-line')).toHaveLength(FOLD_LINES)
    // The path is the one the artboard draws: the visible lines are the tail, and
    // the button under them counts the earlier ones.
    expect(body?.textContent).toContain('line 34')
    expect(body?.textContent).not.toContain('line 14\n')

    const more = screen.getByRole('button', { name: '14 earlier lines' })
    await fireEvent.click(more)

    expect(container.querySelectorAll('.tc-out .tc-line')).toHaveLength(34)
    expect(screen.getByRole('button', { name: 'Show less' })).toBeTruthy()
  })

  it('draws no "earlier lines" button when the body already fits', () => {
    render(ToolCard, { props: { tool: tool() } })

    expect(screen.queryByRole('button', { name: /earlier lines/ })).toBeNull()
  })

  it('folds and unfolds the body from the header chevron', async () => {
    const { container } = render(ToolCard, { props: { tool: tool() } })
    const detail = container.querySelector<HTMLElement>('.tc-detail')
    const chevron = within(container).getByRole('button', { name: 'Collapse' })

    expect(detail?.hidden).toBe(false)
    expect(chevron.getAttribute('aria-expanded')).toBe('true')

    await fireEvent.click(chevron)

    expect(container.querySelector<HTMLElement>('.tc-detail')?.hidden).toBe(true)
    expect(
      within(container).getByRole('button', { name: 'Expand' }).getAttribute('aria-expanded'),
    ).toBe('false')
    // The footer is the card's own line: folding the body never hides the state.
    expect(words(foot(container))).toBe('Done 0.2s')
  })
})

describe('a running tool card', () => {
  it('draws the spinner state without a duration it has no clock for', () => {
    const { container } = render(ToolCard, {
      props: {
        tool: tool({
          result: null,
          partial_result: 'Waiting for OneDrive to release the lock…',
          running: true,
          duration_ms: null,
        }),
      },
    })

    expect(foot(container).querySelector('.tc-spin')).toBeTruthy()
    expect(words(foot(container))).toBe('Running')
    expect(foot(container).textContent).not.toMatch(/\d/)
    expect(container.querySelector('.tc-out')?.textContent).toContain('Waiting for OneDrive')
  })

  it('draws the elapsed time the clock did give it', () => {
    const { container } = render(ToolCard, {
      props: { tool: tool({ running: true, duration_ms: 4000 }) },
    })

    expect(words(foot(container))).toBe('Running 4.0s')
  })
})

describe('a failed tool card', () => {
  it('draws the first line of its output as the error line', () => {
    const { container } = render(ToolCard, {
      props: {
        tool: tool({
          result: 'ERR_PNPM_NO_LOCKFILE  Cannot install with "frozen-lockfile"\n  at install (pnpm.js:1)',
          is_error: true,
          duration_ms: 12800,
        }),
      },
    })

    const error = container.querySelector('.tc-out .tc-line-error')
    expect(error?.textContent).toBe('ERR_PNPM_NO_LOCKFILE  Cannot install with "frozen-lockfile"')
    // Never an empty body: the error line stands even when the call produced one line.
    expect(container.querySelector('.tc-out')?.textContent).toBeTruthy()
    expect(words(foot(container))).toBe('Failed 12.8s')
    expect(foot(container).querySelector('.tc-spin')).toBeNull()
  })

  it('keeps the error line visible when the output is longer than the fold', () => {
    const lines = [`first line is the error`, ...Array.from({ length: 40 }, (_, i) => `line ${i}`)]
    const { container } = render(ToolCard, {
      props: { tool: tool({ result: lines.join('\n'), is_error: true }) },
    })

    expect(container.querySelector('.tc-out .tc-line-error')?.textContent).toBe(
      'first line is the error',
    )
    expect(screen.getByRole('button', { name: `${40 - FOLD_LINES} earlier lines` })).toBeTruthy()
  })
})

describe('the two payload losses', () => {
  it('says the output was cut at the bound when it was truncated', () => {
    const { container } = render(ToolCard, { props: { tool: tool({ truncated: true }) } })

    expect(container.querySelector('.tc-cut')?.textContent).toBe('output cut at the bound')
  })

  it('says a field was dropped when one was, in different words', () => {
    const { container } = render(ToolCard, { props: { tool: tool({ dropped: true }) } })

    expect(container.querySelector('.tc-cut')?.textContent).toBe('a payload field was lost')
  })

  it('draws both marks when both are true, and neither otherwise', () => {
    const both = render(ToolCard, {
      props: { tool: tool({ truncated: true, dropped: true }) },
    })
    const marks = Array.from(both.container.querySelectorAll('.tc-cut')).map(
      (mark) => mark.textContent,
    )
    expect(marks).toEqual(['output cut at the bound', 'a payload field was lost'])
    cleanup()

    const clean = render(ToolCard, { props: { tool: tool() } })
    expect(clean.container.querySelector('.tc-cut')).toBeNull()
  })
})
