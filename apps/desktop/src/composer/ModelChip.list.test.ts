// What the native reviews found in the pill dropdown, one test per finding: the
// typed filter survives a state push (R3-1), a list that shrinks while open
// keeps a row highlighted (R4-1), and Enter and Space are left to the native
// button (R3-2, R4-2). Split from ModelChip.test.ts so neither file grows past
// its lint cap.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import type { ModelChipView } from '../contract/bindings'
import { resetTauriMock, setHandler } from '../test-support/tauri-mock'
import Composer from './Composer.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))
vi.mock('@tauri-apps/api/webview', () => import('../test-support/webview').then((m) => m.webviewModule()))

const READY: ModelChipView = {
  status: { type: 'Ready' },
  models: [
    { provider: 'prov', model_id: 'm1', name: 'Model One', reasoning: true },
    { provider: 'prov', model_id: 'm2', name: 'Model Two', reasoning: false },
    { provider: 'other', model_id: 'm3', name: 'Other Model', reasoning: false },
  ],
  current: { provider: 'prov', model_id: 'm1', name: 'Model One', reasoning: true },
  thinking_level: 'high',
  thinking_levels: ['off', 'medium', 'high'],
}

function chipProp(chip: ModelChipView | null) {
  return render(Composer, { props: { chat: 'c1', chip } })
}

function modelPill() {
  return screen.getByRole('button', { name: 'Model' })
}

/** Opens one pill's list and waits for the popover to render. */
async function openList(pill: HTMLElement) {
  await fireEvent.click(pill)
  return screen.getByRole('listbox', { name: pill.getAttribute('aria-label') as string })
}

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => Promise.resolve(null))
})

afterEach(() => {
  cleanup()
})

describe('the pill dropdown (review findings)', () => {
  // R3-1 (native review): ModelChip rebuilds the options array on every chip
  // push, and a chat that is answering pushes constantly. The reset that runs
  // when the list OPENS must not run again when the options merely change
  // identity, or the filter is wiped while the user types.
  it('keeps the typed filter and the choice when a state push brings the same options (R3-1)', async () => {
    const drawn = chipProp(READY)
    const list = await openList(modelPill())
    const filter = screen.getByPlaceholderText('Search models...') as HTMLInputElement
    await fireEvent.input(filter, { target: { value: 'other' } })
    expect(list.querySelectorAll('[role="option"]').length).toBe(1)

    // The same chip content as a NEW object, the way every push delivers it.
    const sameChip: ModelChipView = {
      ...READY,
      models: READY.models.map((model) => ({ ...model })),
    }
    await drawn.rerender({ chat: 'c1', chip: sameChip })

    const after = screen.getByPlaceholderText('Search models...') as HTMLInputElement
    expect(after.value).toBe('other')
    expect(
      screen.getByRole('listbox', { name: 'Model' }).querySelectorAll('[role="option"]').length,
    ).toBe(1)
  })

  // R4-1 (native review, re-read from the code since the reviewer's text was
  // lost): the highlighted row is an index into the visible list. When a state
  // push brings a SHORTER list while the list is open, the index pointed past
  // the end: no row was highlighted and Enter did nothing.
  it('keeps one row highlighted when the list shrinks while open (R4-1)', async () => {
    const drawn = chipProp(READY)
    await openList(modelPill())
    const list = screen.getByRole('listbox', { name: 'Model' })
    await fireEvent.keyDown(list, { key: 'ArrowDown' })
    await fireEvent.keyDown(list, { key: 'ArrowDown' })
    expect(list.querySelectorAll('.option.active').length).toBe(1)

    await drawn.rerender({ chat: 'c1', chip: { ...READY, models: READY.models.slice(0, 2) } })

    const after = screen.getByRole('listbox', { name: 'Model' })
    expect(after.querySelectorAll('[role="option"]').length).toBe(2)
    expect(after.querySelectorAll('.option.active').length).toBe(1)
  })

  // R3-2 / R4-2 (same caveat): a <button> already turns Enter and Space into a
  // click. Intercepting them as well duplicated the native activation (the key
  // opened the list and the click that follows could close it again). Only
  // ArrowDown, which a button does not handle by itself, is ours.
  it('leaves Enter and Space to the native button and opens on ArrowDown (R3-2, R4-2)', async () => {
    chipProp(READY)
    const pill = modelPill()
    // fireEvent answers false when a handler called preventDefault.
    expect(await fireEvent.keyDown(pill, { key: 'Enter' })).toBe(true)
    expect(await fireEvent.keyDown(pill, { key: ' ' })).toBe(true)
    expect(screen.queryByRole('listbox')).toBeNull()

    expect(await fireEvent.keyDown(pill, { key: 'ArrowDown' })).toBe(false)
    expect(screen.getByRole('listbox', { name: 'Model' })).toBeTruthy()
  })
})
