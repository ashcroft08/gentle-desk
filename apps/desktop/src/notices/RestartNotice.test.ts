// The restart notice as the user meets it: what happened, one button that
// restarts THAT chat, and a dismiss. Both leave through the generated bindings,
// so what is asserted is the intent Rust would receive.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import { handlerArgs, plainNotice, resetTauriMock, restartNotice, setHandler, viewState } from '../test-support/tauri-mock'
import RestartNotice from './RestartNotice.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

const TEXT = 'Chat died: the child that runs this chat exited.'

/** The intents the notice sent, oldest first. */
function intents(): unknown[] {
  return handlerArgs('dispatch_intent').map((args) => args.intent)
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
})

describe('RestartNotice', () => {
  it('says what happened and offers the one action that fixes it', () => {
    render(RestartNotice, { props: { notices: [restartNotice(7, TEXT, 'c:chat-1')] } })

    const alert = screen.getByRole('alert')
    expect(alert.textContent).toContain(TEXT)
    expect(screen.getByRole('button', { name: 'Restart chat' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Dismiss' })).toBeTruthy()
  })

  it('restarts the chat the notice names', async () => {
    render(RestartNotice, { props: { notices: [restartNotice(7, TEXT, 'c:chat-1')] } })

    await fireEvent.click(screen.getByRole('button', { name: 'Restart chat' }))

    expect(intents()).toEqual([{ type: 'RestartChat', data: { chat_id: 'c:chat-1' } }])
  })

  it('dismisses by the notice id, which is not the chat id', async () => {
    render(RestartNotice, { props: { notices: [restartNotice(7, TEXT, 'c:chat-1')] } })

    await fireEvent.click(screen.getByRole('button', { name: 'Dismiss' }))

    expect(intents()).toEqual([{ type: 'DismissNotice', data: { id: 7 } }])
  })

  it('draws no line for a notice with nothing to fix', () => {
    render(RestartNotice, {
      props: { notices: [plainNotice(1, 'Could not read 3 chat files.')] },
    })

    expect(screen.queryByRole('alert')).toBeNull()
    expect(screen.queryByRole('button')).toBeNull()
  })
})
