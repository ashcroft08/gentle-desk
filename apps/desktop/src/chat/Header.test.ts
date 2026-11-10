// The chat header, one test per rule: it names the open chat, the whole title is
// there for whoever asks (a hover, a screen reader) even when CSS cuts it short,
// and it draws nothing else.

import { afterEach, describe, expect, it } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import Header from './Header.svelte'

afterEach(cleanup)

describe('Header', () => {
  it('names the open chat by its title', () => {
    render(Header, { props: { title: 'Tauri skeleton: window and bridge' } })

    expect(screen.getByRole('heading', { name: 'Tauri skeleton: window and bridge' })).toBeTruthy()
  })

  it('keeps the whole title readable when the width cuts it short', () => {
    const title = 'A title long enough that the header has to cut it short on a narrow window'
    render(Header, { props: { title } })

    const heading = screen.getByRole('heading')
    // Truncation is a drawing decision: the text itself and the tooltip keep the
    // session's full name.
    expect(heading.textContent).toBe(title)
    expect(heading.getAttribute('title')).toBe(title)
  })

  it('draws no other control: the artboard menu and view tabs are not built yet', () => {
    render(Header, { props: { title: 'Idle one' } })

    expect(screen.queryByRole('button')).toBeNull()
    expect(screen.queryByRole('tab')).toBeNull()
  })
})
