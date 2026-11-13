// The chat header, one test per rule: it names the open chat, the whole title is
// there for whoever asks (a hover, a screen reader) even when CSS cuts it short,
// and it carries the Chat | Helpers tablist with the tab's working badge.

import { afterEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
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

  it('draws the view tabs beside the title, and no menu', () => {
    render(Header, { props: { title: 'Idle one' } })

    expect(screen.getByRole('tablist', { name: 'View' })).toBeTruthy()
    expect(screen.getAllByRole('tab')).toHaveLength(2)
    // The artboard's "..." menu (Rename, Delete) is not built yet; the only
    // button is the sidebar toggle (responsive-sidebar S1).
    expect(screen.getAllByRole('button')).toHaveLength(1)
  })

  it('reports the tab the user picks and draws the working badge when told to', async () => {
    const onSelect = vi.fn()
    render(Header, { props: { title: 'Busy one', running: 2, onSelect } })

    expect(screen.getByRole('img', { name: '2 helpers working' })).toBeTruthy()
    await fireEvent.click(screen.getByRole('tab', { name: /Helpers/ }))

    expect(onSelect).toHaveBeenCalledWith('helpers')
  })

  it('carries the sidebar toggle, expanded by default (responsive-sidebar S1)', () => {
    render(Header, { props: { title: 'Idle one' } })
    const toggle = screen.getByRole('button', { name: 'Toggle sidebar' })
    expect(toggle.getAttribute('aria-expanded')).toBe('true')
  })

  it('the toggle reports the collapsed state and fires the toggle (S1)', async () => {
    const onToggleSidebar = vi.fn()
    render(Header, { props: { title: 'Idle one', collapsed: true, onToggleSidebar } })
    const toggle = screen.getByRole('button', { name: 'Toggle sidebar' })
    expect(toggle.getAttribute('aria-expanded')).toBe('false')
    await fireEvent.click(toggle)
    expect(onToggleSidebar).toHaveBeenCalledTimes(1)
  })
})
