// The bar a run without a usable launcher shows: the one message iced drew there,
// announced as an alert, with no dismiss and no action because nothing on this
// screen can fix it.

import { afterEach, describe, expect, it } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import FatalBanner from './FatalBanner.svelte'

const TEXT = 'gentle-shell is not available'

afterEach(cleanup)

describe('the fatal banner', () => {
  it('says why the run cannot start a chat, as an alert', () => {
    render(FatalBanner, { props: { text: TEXT } })

    const alert = screen.getByRole('alert')
    expect(alert.textContent).toContain(TEXT)
  })

  it('offers nothing to click: the bar has no fix of its own', () => {
    render(FatalBanner, { props: { text: TEXT } })

    expect(screen.queryByRole('button')).toBeNull()
  })
})
