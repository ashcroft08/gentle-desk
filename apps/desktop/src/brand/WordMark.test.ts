// The "Gentle Shell" lettering above New session: typed in once when the app
// opens, then still. It is a picture with a name, not a heading.

import { afterEach, describe, expect, it } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import WordMark from './WordMark.svelte'

afterEach(cleanup)

describe('the wordmark', () => {
  it('is one image named Gentle Shell', () => {
    render(WordMark)

    expect(screen.getByRole('img', { name: 'Gentle Shell' })).toBeTruthy()
  })

  it('draws the pixels of the letters and, apart, the pixels of their shadow', () => {
    const { container } = render(WordMark)

    expect(container.querySelectorAll('.wm-cell').length).toBeGreaterThan(100)
    expect(container.querySelectorAll('.wm-shadow').length).toBeGreaterThan(20)
  })

  it('hides its cells from a screen reader', () => {
    const { container } = render(WordMark)

    expect(container.querySelector('svg')?.getAttribute('aria-hidden')).toBe('true')
  })
})
