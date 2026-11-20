// The status row (design/composer-dock S1; sheet D of 02-mode-picker): the one
// quiet line at the end of the thread, above the composer. It is PERMANENT - the
// chat always has this row, and the session mode control lives at its right edge
// whether or not a turn is running - and only its LEFT side, the clock, belongs
// to the turn: a thin spinner, a word and a time, sharing the reasoning row's
// anatomy instead of growing a third visual language.
//
// The seconds are the WEB's clock: it starts counting the moment the chat's mark
// becomes `Working` and stops when the mark leaves, which is why the contract
// carries no timestamp for it. The session mode control at its right edge
// (composer-dock T3c) is `SessionMode.test.ts`'s business; here the line is only
// the spinner, the word and the clock.
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
    // Whitespace between the snippets is the only thing an idle row holds.
    expect(line(drawn.container)?.textContent?.trim()).toBe('')

    await drawn.rerender({ mark: 'Working' })
    await wait(4)

    expect(line(drawn.container)?.textContent).toContain('4s')
  })

  it('stops its clock once the turn ends, and keeps its row', async () => {
    const drawn = render(WorkingLine, { props: { mark: 'Working' } })
    await wait(3)
    expect(line(drawn.container)?.textContent).toContain('3s')

    await drawn.rerender({ mark: null })

    // The row is permanent (sheet D): the turn ends, the row does not go away.
    expect(line(drawn.container)).not.toBeNull()
    expect(line(drawn.container)?.textContent?.trim()).toBe('')
    // Nothing keeps ticking behind a clock nobody draws.
    expect(vi.getTimerCount()).toBe(0)
  })

  it('keeps its row for a chat that is not working, with no clock and no placeholder', () => {
    for (const mark of ['Opening', 'Queued', 'NewReply', 'NeedsAttention', 'Ended'] as const) {
      const { container } = render(WorkingLine, { props: { mark } })
      const row = line(container)!

      expect(row).not.toBeNull()
      // Idle: no spinner, no word, no seconds, and nothing standing in for them.
      expect(row.querySelector('.spin')).toBeNull()
      expect(row.querySelector('.t')).toBeNull()
      expect(row.querySelector('.n')).toBeNull()
      expect(row.textContent?.trim()).toBe('')
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
