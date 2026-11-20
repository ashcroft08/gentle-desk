// The session permission mode, as the status line draws and wires it
// (design/composer-dock/02-mode-picker.html sheets A, B and C; composer-dock
// T3c). One test per rule of the slice:
//
// - a chat the shell answered nothing for draws NO control - no button, no gap;
// - the four modes the channel knows are named, and a value this build does not
//   know is shown raw;
// - Normal and Auto change at once, YOLO and Super YOLO ask first, and only the
//   danger button sends;
// - the shown mode never changes optimistically: the old text stays beside a
//   spinner until the pushed `attempt` moves, and a stuck pending drops;
// - a refused answer says its code and that the mode did not change;
// - the mode is READ once per chat, no matter how many pushes arrive;
// - the status row is PERMANENT (sheet D): the control is on it whether the chat
//   is working or not, and only the clock belongs to the turn.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import { tick } from 'svelte'
import type {
  ChatActivityMark,
  SessionModeView_Serialize as SessionModeView,
} from '../contract/bindings'
import { handlerArgs, resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import Thread from './Thread.svelte'
import WorkingLine from './WorkingLine.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** A mode view as the shell pushes it. */
function mode(over: Partial<SessionModeView> = {}): SessionModeView {
  return { mode: 'normal', refusal: null, attempt: 0, ...over }
}

/** The control's button, or `null` when the row drew none. */
function control(container: HTMLElement): HTMLButtonElement | null {
  return container.querySelector('.work-line .mode')
}

/** The intents this view sent, oldest first. */
function intents(): { type: string }[] {
  return handlerArgs('dispatch_intent').map((args) => args.intent as { type: string })
}

/** Only the writes: what the control asked the session to become. */
function writes(): unknown[] {
  return intents().filter((intent) => intent.type === 'SetSessionMode')
}

/** Only the reads: the one question the thread asks per chat. */
function reads(): unknown[] {
  return intents().filter((intent) => intent.type === 'ReadSessionMode')
}

/** Draws the line for a working chat with the given mode. */
function draw(sessionMode: SessionModeView | null, mark: ChatActivityMark | null = 'Working') {
  return render(WorkingLine, {
    props: { mark, chatId: 'chat-1', sessionMode },
    // The line's clock ticks; nothing here waits on a second.
  })
}

/** Opens the menu of the drawn line. */
async function openMenu(container: HTMLElement): Promise<void> {
  await fireEvent.click(control(container)!)
  await tick()
}

/** The menu's four choices, in the order the design lists them. */
function choices(): HTMLElement[] {
  return screen.getAllByRole('menuitemradio')
}

/** The choice at `index`: 0 Normal, 1 Auto, 2 YOLO, 3 Super YOLO. */
async function pick(index: 0 | 1 | 2 | 3): Promise<void> {
  await fireEvent.click(choices()[index])
}

afterEach(() => {
  cleanup()
  vi.useRealTimers()
})

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
})

describe('the session mode control', () => {
  it('draws no control at all when the shell answered nothing', () => {
    const { container } = draw(null)
    const line = container.querySelector('.work-line')!

    expect(control(container)).toBeNull()
    // And no placeholder either: the row is the spinner, the word and the clock.
    expect(line.children).toHaveLength(3)
    expect(line.textContent).not.toContain('\u2013')
  })

  it('names the four modes the channel knows, in the channel’s own order', async () => {
    const cases: [string, string][] = [
      ['normal', 'Normal'],
      ['auto', 'Auto'],
      ['yolo', 'YOLO'],
      ['super-yolo', 'Super YOLO'],
      // A value this build has not learned yet says what the session really runs.
      ['semi-auto', 'semi-auto'],
    ]
    for (const [wire, label] of cases) {
      const drawn = draw(mode({ mode: wire }))
      expect(control(drawn.container)!.textContent).toContain(label)
      cleanup()
    }
  })

  it('tones YOLO and Super YOLO as danger and leaves Normal and Auto quiet', () => {
    for (const [wire, hot] of [
      ['normal', false],
      ['auto', false],
      ['yolo', true],
      ['super-yolo', true],
    ] as const) {
      const drawn = draw(mode({ mode: wire }))
      expect(control(drawn.container)!.classList.contains('hot')).toBe(hot)
      cleanup()
    }
  })

  it('opens a menu that marks the mode in effect and offers the four choices', async () => {
    const { container } = draw(mode({ mode: 'auto' }))
    const button = control(container)!

    expect(button.getAttribute('aria-haspopup')).toBe('menu')
    expect(button.getAttribute('aria-expanded')).toBe('false')

    await openMenu(container)

    expect(button.getAttribute('aria-expanded')).toBe('true')
    expect(screen.getByRole('menu')).toBeTruthy()
    expect(screen.getByText("This session's mode \u00b7 the default does not change")).toBeTruthy()
    const listed = choices()
    expect(listed).toHaveLength(4)
    expect(listed.filter((choice) => choice.getAttribute('aria-checked') === 'true')).toHaveLength(1)
    expect(listed.find((c) => c.getAttribute('aria-checked') === 'true')!.textContent).toContain('Auto')
    // The two widest ask first; the other two say nothing they have no source for.
    expect(listed[2].textContent).toContain('asks to confirm')
    expect(listed[3].textContent).toContain('asks to confirm')
    expect(listed[0].textContent).not.toContain('asks to confirm')
    expect(listed[1].textContent).not.toContain('asks to confirm')
    expect(screen.getByRole('separator')).toBeTruthy()
  })

  it('changes at once for Normal, with the exact intent and no confirmation', async () => {
    const { container } = draw(mode({ mode: 'auto' }))
    await openMenu(container)

    await pick(0)

    expect(writes()).toEqual([
      { type: 'SetSessionMode', data: { chat_id: 'chat-1', mode: 'normal' } },
    ])
    expect(screen.queryByRole('dialog')).toBeNull()
  })

  it('sends nothing for YOLO until its danger button is pressed', async () => {
    const { container } = draw(mode())
    await openMenu(container)

    await pick(2)

    // The list is replaced by the question, in the same popover.
    expect(writes()).toEqual([])
    expect(screen.getByRole('dialog', { name: 'Confirm YOLO' })).toBeTruthy()
    expect(screen.getByText('Switch this session to YOLO?')).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Cancel' })).toBeTruthy()

    await fireEvent.click(screen.getByRole('button', { name: 'Switch to YOLO' }))

    expect(writes()).toEqual([{ type: 'SetSessionMode', data: { chat_id: 'chat-1', mode: 'yolo' } }])
  })

  it('asks about Super YOLO with its own sentence and sends only from the danger button', async () => {
    const { container } = draw(mode())
    await openMenu(container)

    await pick(3)

    expect(writes()).toEqual([])
    expect(screen.getByText('Switch this session to Super YOLO?')).toBeTruthy()
    // YOLO drops the private-paths sentence; Super YOLO keeps it.
    expect(screen.getByText(/private paths stay blocked/)).toBeTruthy()

    await fireEvent.click(screen.getByRole('button', { name: 'Switch to Super YOLO' }))

    expect(writes()).toEqual([
      { type: 'SetSessionMode', data: { chat_id: 'chat-1', mode: 'super-yolo' } },
    ])
  })

  it('sends nothing when the confirmation is cancelled or escaped', async () => {
    const { container } = draw(mode())
    await openMenu(container)
    await pick(3)

    await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }))

    // Cancelling drops the question and puts the list back: nothing was asked.
    expect(writes()).toEqual([])
    expect(screen.queryByRole('dialog')).toBeNull()
    expect(choices()).toHaveLength(4)

    await pick(3)
    await fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Escape' })

    expect(writes()).toEqual([])
    expect(screen.queryByRole('dialog')).toBeNull()
  })

  it('closes on Escape and gives the focus back to the control', async () => {
    const { container } = draw(mode())
    const button = control(container)!
    await openMenu(container)

    await fireEvent.keyDown(screen.getByRole('menu'), { key: 'Escape' })

    expect(screen.queryByRole('menu')).toBeNull()
    expect(document.activeElement).toBe(button)
  })

  it('keeps the previous mode beside a spinner until the answer moves the attempt', async () => {
    vi.useFakeTimers()
    const drawn = draw(mode({ mode: 'normal', attempt: 0 }))
    await openMenu(drawn.container)
    await pick(1)

    // Sent, but nothing is shown as changed before the channel confirms.
    expect(writes()).toHaveLength(1)
    const button = control(drawn.container)!
    expect(button.textContent).toContain('Normal')
    expect(button.getAttribute('aria-busy')).toBe('true')
    expect(button.querySelector('.dots')).not.toBeNull()

    // A push that answers the write ends the wait, with the mode it reported.
    await drawn.rerender({ mark: 'Working', chatId: 'chat-1', sessionMode: mode({ mode: 'auto', attempt: 1 }) })

    expect(control(drawn.container)!.textContent).toContain('Auto')
    expect(control(drawn.container)!.getAttribute('aria-busy')).toBe('false')
    expect(control(drawn.container)!.querySelector('.dots')).toBeNull()
  })

  it('drops a pending that no answer moves, keeping the mode it had', async () => {
    vi.useFakeTimers()
    const drawn = draw(mode({ mode: 'normal', attempt: 0 }))
    await openMenu(drawn.container)
    await pick(1)

    await vi.advanceTimersByTimeAsync(7999)
    await tick()
    expect(control(drawn.container)!.getAttribute('aria-busy')).toBe('true')

    await vi.advanceTimersByTimeAsync(1)
    await tick()

    expect(control(drawn.container)!.getAttribute('aria-busy')).toBe('false')
    expect(control(drawn.container)!.textContent).toContain('Normal')
    expect(writes()).toHaveLength(1)
  })

  it('does not open a second change while a change is pending', async () => {
    vi.useFakeTimers()
    const { container } = draw(mode({ mode: 'normal', attempt: 0 }))
    await openMenu(container)
    await pick(1)

    await fireEvent.click(control(container)!)

    expect(screen.queryByRole('menu')).toBeNull()
    expect(writes()).toHaveLength(1)
  })

  it('says the refusal code and that the mode did not change', async () => {
    const drawn = draw(mode({ mode: 'normal', attempt: 0 }))
    await drawn.rerender({
      mark: 'Working',
      chatId: 'chat-1',
      sessionMode: mode({ mode: 'normal', refusal: 'permission_denied', attempt: 1 }),
    })

    const alert = screen.getByRole('alert')
    expect(alert.textContent).toContain('Could not change the mode')
    expect(alert.textContent).toContain('permission_denied')
    expect(alert.textContent).toContain('Still on Normal.')
    // The mode itself is left exactly as it was.
    expect(control(drawn.container)!.textContent).toContain('Normal')
  })

  it('shows a code it does not know exactly as the shell spelled it', async () => {
    const drawn = draw(mode())
    await drawn.rerender({
      mark: 'Working',
      chatId: 'chat-1',
      sessionMode: mode({ refusal: 'upstream_said_no', attempt: 1 }),
    })

    expect(screen.getByRole('alert').textContent).toContain('upstream_said_no')
  })

  it('clears the refusal on the next answered change and when the menu opens', async () => {
    const drawn = draw(mode({ mode: 'normal', attempt: 1, refusal: 'permission_denied' }))
    expect(screen.queryByRole('alert')).toBeNull()

    await drawn.rerender({
      mark: 'Working',
      chatId: 'chat-1',
      sessionMode: mode({ mode: 'normal', refusal: 'permission_denied', attempt: 2 }),
    })
    expect(screen.getByRole('alert')).toBeTruthy()

    // A second look at the menu is how the reader asks again.
    await openMenu(drawn.container)
    expect(screen.queryByRole('alert')).toBeNull()

    await drawn.rerender({
      mark: 'Working',
      chatId: 'chat-1',
      sessionMode: mode({ mode: 'auto', refusal: null, attempt: 3 }),
    })
    expect(screen.queryByRole('alert')).toBeNull()
  })

  it('keeps the control while the chat is not working, and draws no clock beside it', () => {
    for (const mark of [null, 'Opening', 'Queued', 'Ended'] as const) {
      const { container } = draw(mode({ mode: 'yolo' }), mark)
      const row = container.querySelector('.work-line')!

      // Sheet D: the row is permanent, and the control sits on it while idle.
      expect(row).not.toBeNull()
      expect(control(container)!.textContent).toContain('YOLO')
      // Idle left side: no spinner, no word, no seconds, no placeholder.
      expect(row.children).toHaveLength(1)
      expect(row.querySelector('.spin')).toBeNull()
      expect(row.querySelector('.t')).toBeNull()
      expect(row.querySelector('.n')).toBeNull()
      expect(row.textContent).not.toContain('\u2013')
      cleanup()
    }
  })

  it('draws an empty row for an idle chat the shell answered nothing for', () => {
    const { container } = draw(null, null)
    const row = container.querySelector('.work-line')!

    expect(row).not.toBeNull()
    expect(control(container)).toBeNull()
    expect(row.textContent?.trim()).toBe('')
    expect(row.children).toHaveLength(0)
  })

  it('keeps the refusal line under a row that is not working', async () => {
    const drawn = draw(mode({ mode: 'normal', attempt: 0 }))
    await drawn.rerender({
      mark: null,
      chatId: 'chat-1',
      sessionMode: mode({ mode: 'normal', refusal: 'permission_denied', attempt: 1 }),
    })

    const alert = screen.getByRole('alert')
    expect(alert.textContent).toContain('permission_denied')
    expect(alert.textContent).toContain('Still on Normal.')
    // The line belongs to the row: it sits under it, and the row is still there.
    expect(drawn.container.querySelector('.work-line')!.nextElementSibling).toBe(alert)
  })
})

describe('the mode read', () => {
  it('asks the chat for its mode once, however many pushes arrive', async () => {
    const props = { messages: [], foregroundChat: 'chat-1', mark: 'Working' as const }
    const drawn = render(Thread, { props })

    expect(reads()).toEqual([{ type: 'ReadSessionMode', data: { chat_id: 'chat-1' } }])

    for (let push = 0; push < 5; push += 1) await drawn.rerender({ ...props })

    expect(reads()).toHaveLength(1)
  })

  it('reads a chat the first time its mark is live, never while it is opening', async () => {
    const drawn = render(Thread, {
      props: { messages: [], foregroundChat: 'chat-1', mark: 'Opening' as const },
    })
    expect(reads()).toEqual([])

    await drawn.rerender({ messages: [], foregroundChat: 'chat-1', mark: 'Working' as const })
    expect(reads()).toHaveLength(1)

    // A second chat is its own question, and a return to the first one is not.
    await drawn.rerender({ messages: [], foregroundChat: 'chat-2', mark: 'Working' as const })
    expect(reads()).toHaveLength(2)
    await drawn.rerender({ messages: [], foregroundChat: 'chat-1', mark: 'Working' as const })
    expect(reads()).toHaveLength(2)
  })
})
