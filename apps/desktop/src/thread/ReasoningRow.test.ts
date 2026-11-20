// The reasoning row (chat-transcript T2b, S3): NOT a card. One quiet line with a
// spark glyph, the word for "thought", the duration chip and a chevron; the text
// opens behind a hairline. It is open while the block still streams, because that
// text is still arriving, and folded once it closes.
import { afterEach, describe, expect, it } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import type { ThinkingItemView_Serialize as Thinking } from '../contract/bindings'
import ReasoningRow from './ReasoningRow.svelte'

/** One reasoning block as Rust pushes it. */
function item(over: Partial<Thinking> = {}): Thinking {
  return {
    id: 'think-1',
    text: 'OneDrive holds the folder, not the repo.',
    streaming: false,
    duration_ms: 1200,
    ...over,
  }
}

afterEach(cleanup)

/** The paragraph the reasoning text is drawn in, open or folded. */
function thought(container: HTMLElement): HTMLElement {
  const element = container.querySelector<HTMLElement>('.think-text')
  if (element === null) throw new Error('the row drew no text paragraph')
  return element
}

describe('the reasoning row', () => {
  it('is one quiet line, never a card', () => {
    const { container } = render(ReasoningRow, { props: { item: item() } })

    expect(container.querySelector('.tool-card')).toBeNull()
    expect(container.querySelector('.think-row')).toBeTruthy()
    expect(screen.getByText('Thought')).toBeTruthy()
    expect(screen.getByText('1.2s')).toBeTruthy()
  })

  it('is open while the block streams', () => {
    render(ReasoningRow, { props: { item: item({ streaming: true }) } })

    const toggle = screen.getByRole('button', { name: /Thought/ })
    expect(toggle.getAttribute('aria-expanded')).toBe('true')
    expect(screen.getByText('OneDrive holds the folder, not the repo.')).toBeTruthy()
  })

  it('folds the text once the block closes', async () => {
    const drawn = render(ReasoningRow, { props: { item: item({ streaming: true }) } })
    expect(thought(drawn.container).hidden).toBe(false)

    await drawn.rerender({ item: item({ streaming: false }) })

    expect(thought(drawn.container).hidden).toBe(true)
    expect(screen.getByRole('button', { name: /Thought/ }).getAttribute('aria-expanded')).toBe(
      'false',
    )
  })

  it('opens and folds from the chevron once the block is closed', async () => {
    const { container } = render(ReasoningRow, { props: { item: item() } })
    const toggle = screen.getByRole('button', { name: /Thought/ })

    expect(thought(container).hidden).toBe(true)

    await fireEvent.click(toggle)
    expect(thought(container).hidden).toBe(false)

    await fireEvent.click(toggle)
    expect(thought(container).hidden).toBe(true)
  })

  it('draws no duration for a block the clock never saw', () => {
    const { container } = render(ReasoningRow, {
      props: { item: item({ duration_ms: null }) },
    })

    expect(container.textContent).not.toMatch(/\d/)
  })
})
