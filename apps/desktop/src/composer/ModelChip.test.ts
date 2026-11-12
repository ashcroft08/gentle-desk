// The model chip's drawn behaviour, one test per rule: the list and the levels
// come from the child's truth, a choice dispatches the intent Rust maps, and
// each state (loading, empty, failed) is drawn instead of guessed.
//
// Every choice leaves through the generated bindings, so what is asserted is
// the intent Rust would receive.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import type { ModelChipView } from '../contract/bindings'
import { handlerArgs, resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import Composer from './Composer.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))
vi.mock('@tauri-apps/api/webview', () => import('../test-support/webview').then((m) => m.webviewModule()))

const READY: ModelChipView = {
  status: { type: 'Ready' },
  models: [
    { provider: 'prov', model_id: 'm1', name: 'Model One', reasoning: true },
    { provider: 'prov', model_id: 'm2', name: 'Model Two', reasoning: false },
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

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => Promise.resolve(null))
})

afterEach(() => {
  cleanup()
})

describe('model chip', () => {
  it('draws the current model and level as selected options', () => {
    chipProp(READY)
    const model = screen.getByRole('combobox', { name: 'Model' }) as HTMLSelectElement
    const level = screen.getByRole('combobox', { name: 'Thinking level' }) as HTMLSelectElement
    expect(model.value).toBe('prov/m1')
    expect(level.value).toBe('high')
  })

  it('choosing a model dispatches SelectModel with provider and model id', async () => {
    chipProp(READY)
    const model = screen.getByRole('combobox', { name: 'Model' })
    await fireEvent.change(model, { target: { value: 'prov/m2' } })
    expect(intents()).toContainEqual({
      type: 'SelectModel',
      data: { provider: 'prov', model_id: 'm2' },
    })
  })

  it('choosing a level dispatches SelectThinkingLevel', async () => {
    chipProp(READY)
    const level = screen.getByRole('combobox', { name: 'Thinking level' })
    await fireEvent.change(level, { target: { value: 'medium' } })
    expect(intents()).toContainEqual({ type: 'SelectThinkingLevel', data: { level: 'medium' } })
  })

  it('a loading or unasked chip says so and offers no dropdown', () => {
    chipProp({ status: { type: 'Loading' }, models: [], current: null, thinking_level: null, thinking_levels: [] })
    expect(screen.queryByRole('combobox', { name: 'Model' })).toBeNull()
    expect(screen.getByText('Loading models...')).toBeTruthy()
  })

  it('an empty list is its own line, not an error', () => {
    chipProp({ status: { type: 'Empty' }, models: [], current: null, thinking_level: null, thinking_levels: [] })
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
    expect(screen.queryByRole('combobox')).toBeNull()
  })

  it('no chip draws nothing at all', () => {
    chipProp(null)
    expect(screen.queryByRole('combobox')).toBeNull()
  })
})
