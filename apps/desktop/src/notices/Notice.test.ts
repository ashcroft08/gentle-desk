// One notice as the user meets it: its level, its text, at most one action that
// fixes what it reports, and a dismiss. Every button leaves through the
// generated bindings, so what is asserted is the intent Rust would receive.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import { handlerArgs, plainNotice, resetTauriMock, restartNotice, setHandler, viewState } from '../test-support/tauri-mock'
import type { NoticeActionView, NoticeView_Serialize } from '../contract/bindings'
import Notice from './Notice.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

const TEXT = 'Could not read 3 chat files.'
const DIED = 'Chat died: the child that runs this chat exited.'

/** The intents the notice sent, oldest first. */
function intents(): unknown[] {
  return handlerArgs('dispatch_intent').map((args) => args.intent)
}

function draw(notice: NoticeView_Serialize): HTMLElement {
  render(Notice, { props: { notice } })
  return screen.getByRole('status', { hidden: false })
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
})

describe('a notice', () => {
  it('says its level in words, not only in colour', () => {
    const card = draw(plainNotice(1, TEXT, 'Warning'))

    expect(card.textContent).toContain(TEXT)
    expect(within(card).getByText('Warning')).toBeTruthy()
  })

  it('says an info notice and offers only the dismiss', () => {
    const card = draw(plainNotice(2, 'Chat renamed.', 'Info'))

    expect(within(card).getByText('Info')).toBeTruthy()
    expect(within(card).getAllByRole('button')).toHaveLength(1)
    expect(within(card).getByRole('button', { name: 'Dismiss' })).toBeTruthy()
  })

  it('announces an error as an alert', () => {
    render(Notice, { props: { notice: restartNotice(3, DIED, 'c:chat-1') } })

    const alert = screen.getByRole('alert')
    expect(within(alert).getByText('Error')).toBeTruthy()
    expect(alert.textContent).toContain(DIED)
  })

  it('offers exactly one action, named after what it does', async () => {
    render(Notice, { props: { notice: restartNotice(4, DIED, 'c:chat-1') } })

    const buttons = screen.getAllByRole('button')
    expect(buttons.map((button) => button.getAttribute('aria-label') ?? button.textContent)).toEqual([
      'Restart chat',
      'Dismiss',
    ])

    await fireEvent.click(screen.getByRole('button', { name: 'Restart chat' }))
    expect(intents()).toEqual([{ type: 'RestartChat', data: { chat_id: 'c:chat-1' } }])
  })

  it('dismisses by the notice id, which is not the chat id', async () => {
    render(Notice, { props: { notice: restartNotice(7, DIED, 'c:chat-1') } })

    await fireEvent.click(screen.getByRole('button', { name: 'Dismiss' }))

    expect(intents()).toEqual([{ type: 'DismissNotice', data: { id: 7 } }])
  })

  it('draws a notice whose action this view does not know, with no button for it', () => {
    // The action is drawn from the contract, never guessed: an action this
    // build has no rule for keeps its text and its dismiss and adds nothing.
    const unknown = { type: 'RenameChat', data: { chat_id: 'c:chat-1' } } as unknown as NoticeActionView
    const card = draw({ id: 9, text: TEXT, level: 'Warning', action: unknown })

    expect(card.textContent).toContain(TEXT)
    expect(within(card).getAllByRole('button')).toHaveLength(1)
  })
})
