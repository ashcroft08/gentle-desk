// The status line (design/composer-dock S1, R1): the one quiet line at the end of
// the thread, above the composer, that says what the chat is doing. It shares the
// reasoning row's anatomy - a thin spinner, a word and a time - instead of
// growing a third visual language, and it draws nothing at all once the turn
// ends.
//
// The seconds are the WEB's clock: it starts counting the moment the chat's mark
// becomes `Working` and stops when the mark leaves, which is why the contract
// carries no timestamp for it. The right side keeps the place the read-only
// session mode takes (T3); this slice puts nothing there, not even a gap.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import { tick } from 'svelte'
import WorkingLine from './WorkingLine.svelte'

/** The line's own element, or null when it drew nothing. */
function line(container: HTMLElement): HTMLElement | null {
  return container.querySelector('.work-line') as HTMLElement | null
}

/** Lets `seconds` pass on the app's clock and lets the line redraw. */
async function wait(seconds: number): Promise<void> {
  await vi.advanceTimersByTimeAsync(seconds * 1000)
  await tick()
}

beforeEach(() => {
  vi.useFakeTimers()
})

afterEach(() => {
  cleanup()
  vi.useRealTimers()
})

describe('the working line', () => {
  it('draws the working word and the seconds it has been working', async () => {
    const { container } = render(WorkingLine, { props: { mark: 'Working' } })

    expect(screen.getByText('Working')).toBeTruthy()
    expect(line(container)?.textContent).toContain('0s')

    await wait(12)

    expect(line(container)?.textContent).toContain('12s')
  })

  it('starts counting when the mark becomes working, not when the thread drew', async () => {
    const drawn = render(WorkingLine, { props: { mark: null } })
    await wait(5)
    expect(line(drawn.container)).toBeNull()

    await drawn.rerender({ mark: 'Working' })
    await wait(4)

    expect(line(drawn.container)?.textContent).toContain('4s')
  })

  it('disappears, and stops its clock, once the turn ends', async () => {
    const drawn = render(WorkingLine, { props: { mark: 'Working' } })
    await wait(3)
    expect(line(drawn.container)).not.toBeNull()

    await drawn.rerender({ mark: null })

    expect(line(drawn.container)).toBeNull()
    // Nothing keeps ticking behind a line nobody draws.
    expect(vi.getTimerCount()).toBe(0)
  })

  it('draws nothing for a chat that is not working', () => {
    for (const mark of ['Opening', 'Queued', 'NewReply', 'NeedsAttention', 'Ended'] as const) {
      const { container } = render(WorkingLine, { props: { mark } })
      expect(line(container)).toBeNull()
      cleanup()
    }
  })

  it('spins a glyph a screen reader skips, and keeps the ticking number out of the way', () => {
    const { container } = render(WorkingLine, { props: { mark: 'Working' } })
    const drawn = line(container)!

    expect(drawn.querySelector('.spin')?.getAttribute('aria-hidden')).toBe('true')
    expect(drawn.querySelector('.n')?.getAttribute('aria-hidden')).toBe('true')
    // The word alone is the state, so a reader is never read a number per second.
    expect(drawn.querySelector('.t')?.getAttribute('aria-hidden')).toBeNull()
  })
})
