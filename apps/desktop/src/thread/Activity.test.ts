// The line under the last message that tells the user the app is opening a chat:
// that takes seconds and used to show only a changed button. The working turn is
// NOT this line any more - design/composer-dock S1 moved it to the status line,
// which draws the word, a spinner and the seconds it has been working - so this
// one draws the opening state and nothing else.

import { afterEach, describe, expect, it } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import Activity from './Activity.svelte'

const OPENING = 'Opening chat'
const WORKING = 'Working'

afterEach(cleanup)

describe('the activity line', () => {
  it('says the chat is opening, as a status', () => {
    render(Activity, { props: { mark: 'Opening' } })

    expect(screen.getByRole('status').textContent).toContain(OPENING)
  })

  // The working turn has its own line now (`./WorkingLine`): this one must not
  // draw it too, or the thread says "working" twice.
  it('leaves the working turn to the status line', () => {
    const { container } = render(Activity, { props: { mark: 'Working' } })

    expect(container.querySelector('.activity')).toBeNull()
    expect(screen.queryByText(WORKING)).toBeNull()
  })

  it.each(['Queued', 'NewReply', 'NeedsAttention', 'Ended', null] as const)(
    'draws nothing for %s',
    (mark) => {
      const { container } = render(Activity, { props: { mark } })

      expect(container.querySelector('.activity')).toBeNull()
    },
  )

  it('animates three dots that a screen reader skips', () => {
    const { container } = render(Activity, { props: { mark: 'Opening' } })

    const dots = container.querySelectorAll('.activity .dot')
    expect(dots.length).toBe(3)
    expect(container.querySelector('.activity .dots')?.getAttribute('aria-hidden')).toBe('true')
  })
})
