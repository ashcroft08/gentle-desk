// The header's Chat | Helpers tablist, one test per rule: the ARIA wiring 01b
// draws (tablist, tab, aria-controls, roving tabindex), the arrow/Home/End walk
// that takes the focus with it, and the working badge that only appears while a
// helper runs.

import { afterEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import type { ViewTab } from './tabs'
import ViewTabs from './ViewTabs.svelte'

afterEach(cleanup)

/** Draws the tablist and answers the callback it reports a choice to. */
function draw(over: { tab?: ViewTab; running?: number } = {}): ReturnType<typeof vi.fn> {
  const onSelect = vi.fn()
  render(ViewTabs, { props: { tab: over.tab ?? 'chat', running: over.running ?? 0, onSelect } })
  return onSelect
}

describe('ViewTabs', () => {
  it('wires a tablist of two tabs to the two panels it switches', () => {
    draw()

    const list = screen.getByRole('tablist', { name: 'View' })
    const chat = within(list).getByRole('tab', { name: 'Chat' })
    const helpers = within(list).getByRole('tab', { name: 'Helpers' })

    expect(chat.getAttribute('aria-selected')).toBe('true')
    expect(chat.getAttribute('aria-controls')).toBe('p-chat')
    expect(chat.id).toBe('t-chat')
    expect(helpers.getAttribute('aria-selected')).toBe('false')
    expect(helpers.getAttribute('aria-controls')).toBe('p-helpers')
    expect(helpers.id).toBe('t-help')
  })

  it('keeps one tab in the tab order: the selected one', () => {
    draw({ tab: 'helpers' })

    expect(screen.getByRole('tab', { name: 'Chat' }).getAttribute('tabindex')).toBe('-1')
    expect(screen.getByRole('tab', { name: 'Helpers' }).getAttribute('tabindex')).toBe('0')
  })

  it('walks the tabs with the arrow keys, taking the focus along', async () => {
    const onSelect = draw()
    const chat = screen.getByRole('tab', { name: 'Chat' })

    await fireEvent.keyDown(chat, { key: 'ArrowRight' })

    expect(onSelect).toHaveBeenLastCalledWith('helpers')
    expect(document.activeElement).toBe(screen.getByRole('tab', { name: 'Helpers' }))
  })

  it('wraps at the ends and jumps with Home and End', async () => {
    const onSelect = draw({ tab: 'helpers' })
    const helpers = screen.getByRole('tab', { name: 'Helpers' })

    await fireEvent.keyDown(helpers, { key: 'ArrowRight' })
    expect(onSelect).toHaveBeenLastCalledWith('chat')
    await fireEvent.keyDown(helpers, { key: 'Home' })
    expect(onSelect).toHaveBeenLastCalledWith('chat')
    await fireEvent.keyDown(helpers, { key: 'End' })
    expect(onSelect).toHaveBeenLastCalledWith('helpers')
  })

  it('leaves every other key to the tab itself', async () => {
    const onSelect = draw()

    await fireEvent.keyDown(screen.getByRole('tab', { name: 'Chat' }), { key: 'Enter' })

    expect(onSelect).not.toHaveBeenCalled()
  })

  it('draws the working badge on the Helpers tab only while a helper runs', () => {
    draw({ running: 0 })
    expect(screen.queryByRole('img')).toBeNull()

    cleanup()
    draw({ running: 1 })
    const helpersTab = screen.getByRole('tab', { name: /Helpers/ })
    expect(within(helpersTab).getByRole('img', { name: '1 helper working' })).toBeTruthy()

    cleanup()
    draw({ running: 3 })
    expect(screen.getByRole('tab', { name: /Helpers/ })).toBeTruthy()
    expect(screen.getByRole('img', { name: '3 helpers working' })).toBeTruthy()
  })
})
