// The chat header, one test per rule: it names the open chat, the whole title is
// there for whoever asks (a hover, a screen reader) even when CSS cuts it short,
// it carries the Chat | Helpers tablist with the tab's working badge, and since
// chat-chrome-helpers S2 it carries the pencil that renames the chat in place -
// Enter commits, Escape cancels, losing the focus commits - beside the switch
// pinned to the right edge.

import { afterEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import Header from './Header.svelte'
import { RENAME_WHY_ID, chatCopy } from './copy'

afterEach(cleanup)

/** The pencil the header draws: its accessible name is the tooltip's own. */
function pencil(): HTMLElement {
  return screen.getByRole('button', { name: 'Rename chat' })
}

/** The inline rename field the pencil opens. */
function field(): HTMLElement {
  return screen.getByRole('textbox', { name: 'Chat name' })
}

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
    // The artboard's "..." menu (Delete and the rest) is not built; the buttons
    // are the sidebar toggle and the pencil (S2).
    expect(screen.getAllByRole('button')).toHaveLength(2)
    expect(pencil()).toBeTruthy()
  })

  it('pins the Chat | Helpers switch to the right edge (S2)', () => {
    render(Header, { props: { title: 'Idle one' } })

    const row = screen.getByRole('banner')
    const children = [...row.children]
    // The switch closes the row, and the one flexible spacer sits right before
    // it: nothing between the two, so the switch takes the edge and the title
    // takes what is left.
    expect(children.at(-1)?.getAttribute('role')).toBe('tablist')
    expect(children.at(-2)?.classList.contains('sp')).toBe(true)
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

  it('names the pencil and its tooltip together', () => {
    render(Header, { props: { title: 'Idle one' } })

    expect(pencil().getAttribute('data-tip')).toBe('Rename chat')
  })

  it('opens the inline rename holding the title, ready to type (S2)', async () => {
    render(Header, { props: { title: 'Idle one' } })

    await fireEvent.click(pencil())

    const input = field()
    expect((input as HTMLInputElement).value).toBe('Idle one')
    await vi.waitFor(() => expect(document.activeElement).toBe(input))
    expect(screen.queryByRole('heading')).toBeNull()
  })

  it('commits the typed name on Enter, trimmed', async () => {
    const onRename = vi.fn()
    render(Header, { props: { title: 'Idle one', onRename } })
    await fireEvent.click(pencil())

    const input = field()
    await fireEvent.input(input, { target: { value: '  Season two  ' } })
    await fireEvent.keyDown(input, { key: 'Enter' })

    expect(onRename).toHaveBeenCalledWith('Season two')
    // The input is gone and the title is the header's again.
    expect(screen.queryByRole('textbox', { name: 'Chat name' })).toBeNull()
    expect(screen.getByRole('heading')).toBeTruthy()
  })

  it('cancels on Escape and renames nothing', async () => {
    const onRename = vi.fn()
    render(Header, { props: { title: 'Idle one', onRename } })
    await fireEvent.click(pencil())

    const input = field()
    await fireEvent.input(input, { target: { value: 'Never sent' } })
    await fireEvent.keyDown(input, { key: 'Escape' })

    expect(onRename).not.toHaveBeenCalled()
    expect(screen.queryByRole('textbox', { name: 'Chat name' })).toBeNull()
    expect(screen.getByRole('heading', { name: 'Idle one' })).toBeTruthy()
  })

  it('commits when the field loses the focus (S2)', async () => {
    const onRename = vi.fn()
    render(Header, { props: { title: 'Idle one', onRename } })
    await fireEvent.click(pencil())

    const input = field()
    await fireEvent.input(input, { target: { value: 'Season three' } })
    await fireEvent.blur(input)

    expect(onRename).toHaveBeenCalledWith('Season three')
    expect(screen.queryByRole('textbox', { name: 'Chat name' })).toBeNull()
  })

  it('sends nothing when the name did not change', async () => {
    const onRename = vi.fn()
    render(Header, { props: { title: 'Idle one', onRename } })
    await fireEvent.click(pencil())

    await fireEvent.keyDown(field(), { key: 'Enter' })

    expect(onRename).not.toHaveBeenCalled()
  })

  it('says why the pencil cannot rename while no chat is open, and opens nothing', async () => {
    const onRename = vi.fn()
    render(Header, { props: { title: '', canRename: false, onRename } })

    // The house's disabled control: `aria-disabled`, still in the tab order, and
    // described by the line that says why (the states board's rule).
    expect(pencil().getAttribute('aria-disabled')).toBe('true')
    expect(pencil().getAttribute('aria-describedby')).toBe(RENAME_WHY_ID)
    expect(screen.getByText(chatCopy('en').renameReason).id).toBe(RENAME_WHY_ID)

    pencil().focus()
    await fireEvent.click(pencil())

    // No field opens, so no name can be committed and dropped on the way out.
    expect(screen.queryByRole('textbox', { name: 'Chat name' })).toBeNull()
    expect(onRename).not.toHaveBeenCalled()
  })

  it('draws no reason and an enabled pencil while there is a chat to rename', () => {
    render(Header, { props: { title: 'Idle one' } })

    expect(document.getElementById(RENAME_WHY_ID)).toBeNull()
    expect(pencil().getAttribute('aria-disabled')).toBeNull()
    expect(pencil().getAttribute('aria-describedby')).toBeNull()
  })
})
