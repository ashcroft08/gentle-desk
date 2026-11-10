// The composer's drawn behaviour, one test per rule: Enter sends once and clears
// the box, Shift+Enter and an IME Enter do not, an empty draft does nothing, Stop
// replaces Send while the chat works, each disabled state says why and keeps the
// typed text, the draft is per chat, and the focus follows the rules.
//
// Every send leaves through the generated bindings, so what is asserted is the
// intent Rust would receive.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import { tick } from 'svelte'
import type { ChatActivityMark } from '../contract/bindings'
import { handlerArgs, resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import Composer from './Composer.svelte'
import { ENDED_REASON, OPENING_REASON } from './copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))
vi.mock('@tauri-apps/api/webview', () => import('../test-support/webview').then((m) => m.webviewModule()))

const MESSAGE = 'Message'

type Props = { chat: string | null; mark?: ChatActivityMark | null }

/** Draws the composer for one chat in one state. */
function draw(props: Props) {
  const drawn = render(Composer, { props })
  return { ...drawn, box: screen.getByRole('textbox', { name: MESSAGE }) as HTMLTextAreaElement }
}

/** Types `text` into the box, the way the browser reports it. */
async function type(box: HTMLTextAreaElement, text: string): Promise<void> {
  await fireEvent.input(box, { target: { value: text } })
}

/** The intents the composer sent, oldest first. */
function intents(): unknown[] {
  return handlerArgs('dispatch_intent').map((args) => args.intent)
}

/**
 * One Enter as the browser reports it, with whatever the case needs. It bubbles,
 * because that is what the framework's own key listener is attached to: a
 * non-bubbling event would test nothing at all.
 */
function enter(options: KeyboardEventInit = {}): KeyboardEvent {
  return new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true, ...options })
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
  setHandler('attach_images', () => viewState())
  setHandler('remove_image', () => viewState())
})

describe('Composer', () => {
  it('sends the trimmed draft once on Enter and clears the box', async () => {
    const { box } = draw({ chat: 'chat-1' })
    await type(box, '  hello there  ')

    const event = enter()
    box.dispatchEvent(event)

    await tick()

    expect(event.defaultPrevented).toBe(true)
    expect(intents()).toEqual([{ type: 'SendPrompt', data: { text: 'hello there' } }])
    expect(box.value).toBe('')
  })

  it('leaves Shift+Enter to the box, which inserts a newline and sends nothing', async () => {
    const { box } = draw({ chat: 'chat-1' })
    await type(box, 'first line')

    const event = enter({ shiftKey: true })
    box.dispatchEvent(event)

    expect(event.defaultPrevented).toBe(false)
    expect(intents()).toEqual([])
    expect(box.value).toBe('first line')
  })

  it('never sends an Enter that commits an IME composition', async () => {
    const { box } = draw({ chat: 'chat-1' })
    await type(box, 'kana')

    const composing = enter({ isComposing: true })
    box.dispatchEvent(composing)
    // The legacy signal some browsers send instead of `isComposing`.
    const legacy = new KeyboardEvent('keydown', {
      key: 'Enter',
      keyCode: 229,
      bubbles: true,
      cancelable: true,
    } as KeyboardEventInit)
    box.dispatchEvent(legacy)

    expect(intents()).toEqual([])
    expect(box.value).toBe('kana')
  })

  it('does nothing with an empty or whitespace-only draft', async () => {
    const { box } = draw({ chat: 'chat-1' })

    box.dispatchEvent(enter())
    await type(box, '   ')
    box.dispatchEvent(enter())

    expect(intents()).toEqual([])
  })

  it('asks what the user needs, and lists its three keys under the box', () => {
    const { box } = draw({ chat: 'chat-1' })

    expect(box.placeholder).toBe('Tell Gentle what you need...')
    expect(screen.getByText('Enter to send \u00b7 Shift+Enter for a new line \u00b7 Esc to stop')).toBeTruthy()
  })

  it('stops the turn on Escape while the chat works', async () => {
    const { box } = draw({ chat: 'chat-1', mark: 'Working' })

    await fireEvent.keyDown(box, { key: 'Escape' })

    expect(intents()).toEqual([{ type: 'Abort' }])
  })

  it('does nothing on Escape when the chat is idle', async () => {
    const { box } = draw({ chat: 'chat-1' })

    await fireEvent.keyDown(box, { key: 'Escape' })

    expect(intents()).toEqual([])
  })

  it('shows Stop and sends Abort while the chat is working', async () => {
    draw({ chat: 'chat-1', mark: 'Working' })

    expect(screen.queryByRole('button', { name: 'Send' })).toBeNull()
    await fireEvent.click(screen.getByRole('button', { name: 'Stop' }))

    expect(intents()).toEqual([{ type: 'Abort' }])
  })

  it('shows Send and never Stop on an idle chat', () => {
    draw({ chat: 'chat-1' })

    expect(screen.getByRole('button', { name: 'Send' })).toBeTruthy()
    expect(screen.queryByRole('button', { name: 'Stop' })).toBeNull()
    expect(screen.queryByText(OPENING_REASON)).toBeNull()
  })

  it('disables Send while the chat is opening, says why, and keeps the draft', async () => {
    const { box } = draw({ chat: 'chat-1', mark: 'Opening' })
    await type(box, 'a draft')

    const send = screen.getByRole('button', { name: 'Send' })
    const reason = screen.getByText(OPENING_REASON)
    expect(send.getAttribute('aria-disabled')).toBe('true')
    expect(send.getAttribute('aria-describedby')).toBe(reason.id)
    expect((reason as HTMLElement).id).toBe('composer-send-why')

    box.dispatchEvent(enter())
    await fireEvent.click(send)

    expect(intents()).toEqual([])
    expect(box.value).toBe('a draft')
  })

  it('shows Stop for a chat that waits for a slot, and Stop cancels the queued prompt', async () => {
    const { box } = draw({ chat: 'chat-1', mark: 'Queued' })
    await type(box, 'a draft')

    expect(screen.queryByRole('button', { name: 'Send' })).toBeNull()
    await fireEvent.click(screen.getByRole('button', { name: 'Stop' }))

    expect(intents()).toEqual([{ type: 'Abort' }])
    expect(box.value).toBe('a draft')
  })

  it('disables Send once the chat is gone, says why, and keeps the draft', async () => {
    const { box } = draw({ chat: 'chat-1', mark: 'Ended' })
    await type(box, 'a draft')

    const send = screen.getByRole('button', { name: 'Send' })
    expect(send.getAttribute('aria-disabled')).toBe('true')
    expect(send.getAttribute('aria-describedby')).toBe(screen.getByText(ENDED_REASON).id)

    await fireEvent.click(send)

    expect(intents()).toEqual([])
    expect(box.value).toBe('a draft')
  })

  it('keeps one draft per chat across a switch', async () => {
    const drawn = draw({ chat: 'chat-1' })
    await type(drawn.box, 'first chat')

    await drawn.rerender({ chat: 'chat-2' })
    expect(drawn.box.value).toBe('')
    await type(drawn.box, 'second chat')

    await drawn.rerender({ chat: 'chat-1' })
    expect(drawn.box.value).toBe('first chat')
  })

  it('focuses the box when a chat opens and keeps it after a send', async () => {
    const { box } = draw({ chat: 'chat-1' })
    await Promise.resolve()
    expect(document.activeElement).toBe(box)

    await type(box, 'hello')
    box.dispatchEvent(enter())
    expect(document.activeElement).toBe(box)

    // A click on Send leaves the cursor in the box too, so the next turn can be
    // typed without reaching for the mouse again.
    await type(box, 'again')
    await fireEvent.click(screen.getByRole('button', { name: 'Send' }))
    expect(document.activeElement).toBe(box)
  })

  it('leaves the draft alone on Escape', async () => {
    const { box } = draw({ chat: 'chat-1' })
    await type(box, 'half a thought')

    await fireEvent.keyDown(box, { key: 'Escape' })

    expect(box.value).toBe('half a thought')
    expect(intents()).toEqual([])
  })
})
