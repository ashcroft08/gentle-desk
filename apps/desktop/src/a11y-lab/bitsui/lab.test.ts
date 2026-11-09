// Assertions for the bits-ui lab widgets: dialog, list of options, menu.
// Tests live next to the samples. The library's real behaviour decides which
// of these pass; anything expected-failing is marked with a stated reason.
import { cleanup, render } from '@testing-library/svelte'
import { waitFor } from '@testing-library/dom'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it } from 'vitest'
import './../test-env'
import DialogSample from './DialogSample.svelte'
import ListSample from './ListSample.svelte'
import MenuSample from './MenuSample.svelte'

afterEach(cleanup)

describe('a11y lab bits-ui dialog', () => {
  it('opens a modal dialog: role=dialog and aria-modal=true', async () => {
    const user = userEvent.setup()
    const { getByRole, queryByRole } = render(DialogSample)

    console.log('DBG dialogs in doc=', document.querySelectorAll('[role=dialog]').length)
    expect(queryByRole('dialog')).toBeNull()
    await user.click(getByRole('button', { name: 'Open dialog' }))

    const dialog = getByRole('dialog')
    expect(dialog.getAttribute('aria-modal')).toBe('true')
    expect(getByRole('heading', { name: 'Lab dialog' })).toBeTruthy()
  })

  it('closes on Escape and returns focus to the trigger', async () => {
    const user = userEvent.setup()
    const { getByRole } = render(DialogSample)
    const trigger = getByRole('button', { name: 'Open dialog' })

    await user.click(trigger)
    console.log('DBG dialog after click active=', document.activeElement?.id, (document.activeElement as HTMLElement | null)?.tagName)
    await user.keyboard('{Escape}')

    // jsdom may never fire CSS transitionend, so bits-ui may keep the closing
    // content mounted; the delivered accessibility state is data-state=closed,
    // after which the content is removed entirely.
    await expect
      .poll(() => {
        const dialog = document.querySelector('[data-state][role="dialog"]')
        // content already unmounted (no transition in jsdom) counts as closed
        return dialog === null ? 'closed' : dialog.getAttribute('data-state')
      })
      .toBe('closed')
    console.log('DBG dialog poll done active=', document.activeElement?.id)
    expect(document.activeElement).toBe(trigger)
  })
})

describe('a11y lab bits-ui list of options', () => {
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
    const openTrigger = getByRole('combobox')

    await user.keyboard('{ArrowDown}')
    expect(getByRole('option', { name: 'Beta' }).hasAttribute('data-highlighted')).toBe(true)

    await user.keyboard('{ArrowDown}')
    expect(getByRole('option', { name: 'Gamma' }).hasAttribute('data-highlighted')).toBe(true)

    await user.keyboard('{Enter}')
    // focus returns to the combobox trigger, whose text is the chosen value
    expect(openTrigger.getAttribute('aria-expanded')).toBe('false')
    expect(getByRole('combobox').textContent?.trim()).toBe('Gamma')
  })
})

describe('a11y lab bits-ui menu', () => {
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
    const { getAllByRole, getByRole, queryByRole } = render(MenuSample)
    const trigger = getByRole('button', { name: 'Actions' })

    await user.click(trigger)
    // wait until the menu effects have run and the items are queryable
    await waitFor(() => expect(getAllByRole('menuitem')).toHaveLength(3))
    console.log('DBG menu active=', document.activeElement?.id, (document.activeElement as HTMLElement | null)?.textContent?.trim(), 'items=', getAllByRole('menuitem').map((x) => x.getAttribute('data-highlighted')))
    await user.keyboard('{ArrowDown}')
    await user.keyboard('{ArrowDown}')
    const items = queryByRole('menu')?.querySelectorAll('[role="menuitem"]') ?? []
    expect(items[1]?.hasAttribute('data-highlighted')).toBe(true)

    await user.keyboard('{Escape}')
    // jsdom may never fire CSS transitionend, so bits-ui may keep the closing
    // menu mounted; the delivered accessibility state is data-state=closed,
    // after which the content is removed entirely.
    await expect
      .poll(() => {
        const menu = document.querySelector('[data-state][role="menu"]')
        // content already unmounted (no transition in jsdom) counts as closed
        return menu === null ? 'closed' : menu.getAttribute('data-state')
      })
      .toBe('closed')
    expect(document.activeElement).toBe(trigger)
  })
})
