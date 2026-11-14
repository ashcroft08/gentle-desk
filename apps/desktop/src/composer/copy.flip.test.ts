// A composer DRAWN from its dictionary (0.2.0, S2, S3): one local language write
// redraws the box, its hint and its buttons in the same tick - no push, no
// restart (S2) - while the draft the person typed, the model names the child
// sent and an image's file name keep arriving exactly as they are (D7).
//
// The English words are asserted literally: they are what the composer has drawn
// since the artboard, so a change of copy is a change here. The two columns
// themselves are pinned in `copy.test.ts`.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import { tick } from 'svelte'
import type { ContextUsageView_Serialize as ContextUsageView, ModelChipView } from '../contract/bindings'
import { resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import { setLanguage } from '../i18n/language.svelte'
import Composer from './Composer.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))
vi.mock('@tauri-apps/api/webview', () => import('../test-support/webview').then((m) => m.webviewModule()))

/** The keys hint, in both languages, as the line under the box draws it. */
const KEYS_EN = 'Enter to send \u00b7 Shift+Enter for a new line \u00b7 Esc to stop'
const KEYS_ES = 'Enter para enviar \u00b7 Shift+Enter para una línea nueva \u00b7 Esc para detener'

/** A model list the child failed to send, in the child's own words (D7). */
const FAILED: ModelChipView = {
  status: { type: 'Failed', data: 'socket closed' },
  models: [],
  current: null,
  thinking_level: null,
  thinking_levels: [],
}

/** A reading pi gave, so the context chip draws a number. */
const READING: ContextUsageView = { tokens: 70_000, percent: 35, window: 200_000 }

/** The box, found by the name of the language it is drawn in. */
function box(name: string): HTMLTextAreaElement {
  return screen.getByRole('textbox', { name }) as HTMLTextAreaElement
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
})

describe('a drawn composer', () => {
  it('flips the box, its hint and Send in the same tick, and keeps the draft', async () => {
    setLanguage('en')
    render(Composer, { props: { chat: 'c:1' } })

    expect(box('Message').placeholder).toBe('Tell Gentle what you need...')
    expect(screen.getByText(KEYS_EN)).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Send' })).toBeTruthy()
    await fireEvent.input(box('Message'), { target: { value: 'keep me' } })

    setLanguage('es')
    await tick()

    expect(box('Mensaje').placeholder).toBe('Cuéntale a Gentle lo que necesitas...')
    expect(screen.getByText(KEYS_ES)).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Enviar' })).toBeTruthy()
    // D7: the draft is the person's own text, so no language rewrites it.
    expect(box('Mensaje').value).toBe('keep me')

    setLanguage('en')
  })

  it('says why Send is disabled, and turns it into Stop, in the language of the moment', async () => {
    setLanguage('en')
    render(Composer, { props: { chat: 'c:1', mark: 'Opening' } })
    expect(screen.getByText('Send is available once the chat is ready.')).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(screen.getByText('Enviar estará disponible cuando el chat esté listo.')).toBeTruthy()

    // The word is read when it is drawn: a chat that starts working while the
    // app is Spanish says Stop in Spanish.
    cleanup()
    render(Composer, { props: { chat: 'c:1', mark: 'Working' } })
    expect(screen.getByRole('button', { name: 'Detener' })).toBeTruthy()

    setLanguage('en')
    await tick()

    expect(screen.getByRole('button', { name: 'Stop' })).toBeTruthy()
  })

  it('flips the two chips and leaves the child own words and names alone', async () => {
    setLanguage('en')
    render(Composer, { props: { chat: 'c:1', chip: FAILED, contextUsage: READING } })

    expect(screen.getByText('Models unavailable: socket closed')).toBeTruthy()
    expect(
      screen.getByRole('status', { name: "Context used: 35 percent of the model's window" }),
    ).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(screen.getByText('Modelos no disponibles: socket closed')).toBeTruthy()
    expect(
      screen.getByRole('status', { name: 'Contexto usado: 35 por ciento de la ventana del modelo' }),
    ).toBeTruthy()

    setLanguage('en')
  })
})
