// The model chip's drawn behaviour (S7): the pills sit in their own row below
// the composer form; each opens the app's own themed listbox (no native
// popup), the model list filters as one types, arrows move and Enter chooses,
// Escape and a click outside close, and each choice dispatches the intent
// Rust maps. The failure states are drawn, never guessed.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import type { ModelChipView } from '../contract/bindings'
import { handlerArgs, resetTauriMock, setHandler } from '../test-support/tauri-mock'
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

/** The intents the composer sent, oldest first. */
function intents(): unknown[] {
  return handlerArgs('dispatch_intent').map((args) => args.intent)
}

function modelPill() {
  return screen.getByRole('button', { name: 'Model' })
}

function levelPill() {
  return screen.getByRole('button', { name: 'Thinking level' })
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

describe('model chip', () => {
  it('the pills share the keys-hint line below the composer form', () => {
    const { container } = chipProp(READY)
    const form = container.querySelector('form.composer')
    const bottom = container.querySelector('.dock-bottom')
    const chip = container.querySelector('.chip')
    const keys = container.querySelector('.dock-bottom .keys')
    expect(form).toBeTruthy()
    expect(bottom).toBeTruthy()
    expect(chip).toBeTruthy()
    expect(keys).toBeTruthy()
    // The row comes after the form, and the hints and the pills share it.
    expect(form!.compareDocumentPosition(bottom as Node) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
    expect(bottom!.contains(keys as Node)).toBeTruthy()
    expect(bottom!.contains(chip as Node)).toBeTruthy()
  })

  it('the pills show the current model and level without a list open', () => {
    chipProp(READY)
    expect(modelPill().getAttribute('aria-expanded')).toBe('false')
    expect(screen.getByText('Model One')).toBeTruthy()
    expect(screen.getByText('high')).toBeTruthy()
    expect(screen.queryByRole('listbox')).toBeNull()
  })

  it('opening the model pill shows the themed listbox and its filter', async () => {
    chipProp(READY)
    const list = await openList(modelPill())
    expect(modelPill().getAttribute('aria-expanded')).toBe('true')
    expect(screen.getByPlaceholderText('Search models...')).toBeTruthy()
    expect(list.querySelectorAll('[role="option"]').length).toBe(3)
  })

  it('typing filters the model list', async () => {
    chipProp(READY)
    const list = await openList(modelPill())
    const filter = screen.getByPlaceholderText('Search models...') as HTMLInputElement
    await fireEvent.input(filter, { target: { value: 'other' } })
    const options = list.querySelectorAll('[role="option"]')
    expect(options.length).toBe(1)
    expect(options[0].textContent).toContain('Other Model')
  })

  it('the filter with no match says so instead of showing an empty list', async () => {
    chipProp(READY)
    await openList(modelPill())
    const filter = screen.getByPlaceholderText('Search models...') as HTMLInputElement
    await fireEvent.input(filter, { target: { value: 'zzz' } })
    expect(screen.getByText('Nothing matches.')).toBeTruthy()
  })

  it('filtering and pressing Enter chooses the first match (SelectModel)', async () => {
    chipProp(READY)
    await openList(modelPill())
    const filter = screen.getByPlaceholderText('Search models...') as HTMLInputElement
    await fireEvent.input(filter, { target: { value: 'other' } })
    await fireEvent.keyDown(screen.getByRole('listbox', { name: 'Model' }), { key: 'Enter' })
    expect(intents()).toContainEqual({
      type: 'SelectModel',
      data: { provider: 'other', model_id: 'm3' },
    })
    expect(screen.queryByRole('listbox')).toBeNull()
  })

  it('Enter on the unfiltered list chooses the active (selected) option', async () => {
    chipProp(READY)
    await openList(modelPill())
    await fireEvent.keyDown(screen.getByRole('listbox', { name: 'Model' }), { key: 'Enter' })
    expect(intents()).toContainEqual({
      type: 'SelectModel',
      data: { provider: 'prov', model_id: 'm1' },
    })
  })

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

  it('Escape closes the list', async () => {
    chipProp(READY)
    await openList(modelPill())
    await fireEvent.keyDown(screen.getByRole('listbox', { name: 'Model' }), { key: 'Escape' })
    expect(screen.queryByRole('listbox')).toBeNull()
  })

  it('a click outside closes the list', async () => {
    chipProp(READY)
    await openList(modelPill())
    await fireEvent.pointerDown(document, { bubbles: true })
    expect(screen.queryByRole('listbox')).toBeNull()
  })

  it('choosing a level dispatches SelectThinkingLevel', async () => {
    chipProp(READY)
    await openList(levelPill())
    const options = screen
      .getByRole('listbox', { name: 'Thinking level' })
      .querySelectorAll('[role="option"]')
    expect(options.length).toBe(3)
    fireEvent.click(options[1])
    expect(intents()).toContainEqual({ type: 'SelectThinkingLevel', data: { level: 'medium' } })
  })

  it('a loading or unasked chip says so and offers no pills', () => {
    chipProp({
      status: { type: 'Loading' },
      models: [],
      current: null,
      thinking_level: null,
      thinking_levels: [],
    })
    expect(screen.queryByRole('button', { name: 'Model' })).toBeNull()
    expect(screen.getByText('Loading models...')).toBeTruthy()
  })

  it('an empty list is its own line, not an error', () => {
    chipProp({
      status: { type: 'Empty' },
      models: [],
      current: null,
      thinking_level: null,
      thinking_levels: [],
    })
    expect(screen.getByText('No models are available for this chat.')).toBeTruthy()
  })

  it('a failed ask names the child s error instead of inventing entries', () => {
    chipProp({
      status: { type: 'Failed', data: 'child gone' },
      models: [],
      current: null,
      thinking_level: null,
      thinking_levels: [],
    })
    expect(screen.getByText(/Models unavailable: child gone/)).toBeTruthy()
    expect(screen.queryByRole('button', { name: 'Model' })).toBeNull()
  })

  it('no chip draws nothing at all', () => {
    chipProp(null)
    expect(screen.queryByRole('button', { name: 'Model' })).toBeNull()
  })
})
