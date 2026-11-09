// Assertions for the melt-ui lab widgets: dialog, list of options, menu.
// Tests live next to the samples. The library's real behaviour decides which
// of these pass; anything expected-failing is marked with a stated reason.
import { cleanup, render } from '@testing-library/svelte'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it } from 'vitest'
import './../test-env'
import DialogSample from './DialogSample.svelte'
import ListSample from './ListSample.svelte'
import MenuSample from './MenuSample.svelte'

afterEach(cleanup)

describe('a11y lab melt-ui dialog', () => {
  it('opens a modal dialog: role=dialog and aria-modal=true', async () => {
    const user = userEvent.setup()
    const { getByRole, queryByRole } = render(DialogSample)

    expect(queryByRole('dialog')).toBeNull()
    await user.click(getByRole('button', { name: 'Open dialog' }))

    const dialog = getByRole('dialog')
    expect(dialog.getAttribute('aria-modal')).toBe('true')
    expect(getByRole('heading', { name: 'Lab dialog' })).toBeTruthy()
  })

  it('closes on Escape and returns focus to the trigger', async () => {
    const user = userEvent.setup()
    const { getByRole, queryByRole } = render(DialogSample)
    const trigger = getByRole('button', { name: 'Open dialog' })

    await user.click(trigger)
    await user.keyboard('{Escape}')

    expect(queryByRole('dialog')).toBeNull()
    expect(document.activeElement).toBe(trigger)
  })
})

describe('a11y lab melt-ui list of options', () => {
  it('shows a listbox with options and a selected state', async () => {
    const user = userEvent.setup()
    const { getByRole, getAllByRole } = render(ListSample)

    await user.click(getByRole('combobox'))

    expect(getAllByRole('option')).toHaveLength(3)
    expect(getByRole('option', { name: 'Alpha' }).getAttribute('aria-selected')).toBe('true')
    expect(getByRole('option', { name: 'Beta' }).getAttribute('aria-selected')).not.toBe('true')
  })

  it('moves through the options with the arrow keys and selects with Enter', async () => {
    const user = userEvent.setup()
    const { getByRole } = render(ListSample)

    await user.click(getByRole('combobox'))

    await user.keyboard('{ArrowDown}')
    expect(getByRole('option', { name: 'Beta' }).hasAttribute('data-highlighted')).toBe(true)

    await user.keyboard('{ArrowDown}')
    expect(getByRole('option', { name: 'Gamma' }).hasAttribute('data-highlighted')).toBe(true)

    await user.keyboard('{Enter}')
    const listTrigger = getByRole('combobox')
    expect(listTrigger.getAttribute('aria-expanded')).toBe('false')
    expect(listTrigger.textContent?.trim()).toBe('Gamma')
  })
})

describe('a11y lab melt-ui menu', () => {
  it('opens a menu with items from the button', async () => {
    const user = userEvent.setup()
    const { getByRole, getAllByRole, queryByRole } = render(MenuSample)

    expect(queryByRole('menu')).toBeNull()
    await user.click(getByRole('button', { name: 'Actions' }))

    expect(getByRole('menu')).toBeTruthy()
    expect(getAllByRole('menuitem')).toHaveLength(3)
  })

  it('moves the highlight with the arrow keys, closes on Escape and returns focus', async () => {
    const user = userEvent.setup()
    const { getByRole, queryByRole } = render(MenuSample)
    const trigger = getByRole('button', { name: 'Actions' })

    await user.click(trigger)
    await user.keyboard('{ArrowDown}')
    const items = queryByRole('menu')?.querySelectorAll('[role="menuitem"]') ?? []
    expect(items[0]?.hasAttribute('data-highlighted')).toBe(true)

    await user.keyboard('{Escape}')
    expect(queryByRole('menu')).toBeNull()
    expect(document.activeElement).toBe(trigger)
  })
})
