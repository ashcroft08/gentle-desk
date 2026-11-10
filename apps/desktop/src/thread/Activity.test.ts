// The line under the last message that tells the user the app is busy: opening
// a chat takes seconds and a running turn used to show only a changed button.

import { afterEach, describe, expect, it } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import Activity from './Activity.svelte'

afterEach(cleanup)

describe('the activity line', () => {
  it('says the chat is opening, as a status', () => {
    render(Activity, { props: { mark: 'Opening' } })

    expect(screen.getByRole('status').textContent).toContain('Opening chat')
  })

  it('says the model is working while a turn runs', () => {
    render(Activity, { props: { mark: 'Working' } })

    expect(screen.getByText('Working')).toBeTruthy()
  })

  it.each(['Queued', 'NewReply', 'NeedsAttention', 'Ended', null] as const)(
    'draws nothing for %s',
    (mark) => {
      const { container } = render(Activity, { props: { mark } })

      expect(container.querySelector('.activity')).toBeNull()
    },
  )

  it('animates three dots that a screen reader skips', () => {
    const { container } = render(Activity, { props: { mark: 'Working' } })

    const dots = container.querySelectorAll('.activity .dot')
    expect(dots.length).toBe(3)
    expect(container.querySelector('.activity .dots')?.getAttribute('aria-hidden')).toBe('true')
  })
})
