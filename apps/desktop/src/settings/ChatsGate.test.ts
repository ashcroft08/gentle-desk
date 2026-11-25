// The Updates gate (close-chats S1, S3): the sentence, the one button that closes
// the chats in a single step, and the line that says why some stay open. The button
// dispatches the app's `CloseAllChats` and nothing else: it never asks for a
// confirmation, because it never stops a turn that is running.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, within } from '@testing-library/svelte'
import { handlerArgs, resetTauriMock, setHandler } from '../test-support/tauri-mock'
import ChatsGate from './ChatsGate.svelte'
import { updatesCopy } from './updates-copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => null)
})

const english = updatesCopy('en')
const spanish = updatesCopy('es')

/** The gate for `open` chats, `working` of them mid-turn, in `copy`'s language. */
function draw(open: number, working: number, copy = english): HTMLElement {
  const { container } = render(ChatsGate, {
    props: { hint: copy.gateHint(open), hintId: 'updates-gate', copy, workingChats: working },
  })
  return container as HTMLElement
}

describe('the Updates gate', () => {
  it('says the rule and offers the button that closes the chats', () => {
    const gate = draw(3, 0)

    expect(gate.querySelector('#updates-gate')?.textContent).toBe(english.gateHint(3))
    expect(within(gate).getByRole('button', { name: 'Close the chats' })).toBeTruthy()
  })

  it('dispatches CloseAllChats exactly once per click, and nothing else', async () => {
    const gate = draw(2, 0)

    await fireEvent.click(within(gate).getByRole('button', { name: 'Close the chats' }))

    expect(handlerArgs('dispatch_intent')).toEqual([{ intent: { type: 'CloseAllChats' } }])
  })

  it('draws no working line while no chat is mid-turn', () => {
    expect(draw(2, 0).querySelector('.quiet')).toBeNull()
  })

  it('names one chat that stays open, in the singular', () => {
    const gate = draw(2, 1)

    expect(gate.querySelector('.quiet')?.textContent).toBe(
      'One chat is still working: the button does not close it; stop it and try again.',
    )
  })

  it('names how many chats stay open, in the plural', () => {
    const gate = draw(4, 3)

    expect(gate.querySelector('.quiet')?.textContent).toBe(
      '3 chats are still working: the button does not close them; stop them and try again.',
    )
  })

  it('says all of it in Spanish, and the button too', () => {
    const gate = draw(2, 2, spanish)

    expect(within(gate).getByRole('button', { name: 'Cerrar los chats' })).toBeTruthy()
    expect(gate.querySelector('.quiet')?.textContent).toBe(
      '2 chats siguen trabajando: el botón no los cierra; detenlos y vuelve a intentarlo.',
    )
    expect(spanish.chatsWorking(1)).toBe(
      'Un chat sigue trabajando: el botón no lo cierra; detenlo y vuelve a intentarlo.',
    )
  })
})
