// The WHOLE app in one language (0.2.0, S2, S3; T9's completion audit): every
// area owns its own dictionary, so this file is the one place that proves the
// areas agree - a stored preference of "es" draws the sidebar's rail, the
// composer, a thread label and the update line in Spanish, and one local switch
// brings them all back to English in the same tick (S2: no restart).
//
// It stays a file of its own rather than a case in `App.test.ts` because it needs
// the updater plugin mocked (`App.update.test.ts`'s seam): the update line is one
// of the surfaces, and it is only drawn when the check found a release.
//
// The English words are asserted literally - they are what the app drew before
// 0.2.0 - and the chat title stays Rust's own text in both languages (D7).
import './test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import { tick } from 'svelte'
import { resetTauriMock, viewState } from './test-support/tauri-mock'
import { serveSidebar } from './test-support/sidebar'
import { setLanguage, startLanguage } from './i18n/language.svelte'
import App from './App.svelte'

const { check } = vi.hoisted(() => ({ check: vi.fn() }))

vi.mock('@tauri-apps/api/core', () => import('./test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('./test-support/tauri-mock').then((m) => m.eventModule()))
vi.mock('@tauri-apps/api/webview', () => import('./test-support/webview').then((m) => m.webviewModule()))
vi.mock('@tauri-apps/plugin-updater', () => ({ check }))

/** The chat's own title, as Rust sends it: never a word of this app. */
const TITLE = 'Tauri skeleton: window and bridge'

const MESSAGE = {
  id: 'msg-1',
  role: 'User' as const,
  text: 'hello from the pushed thread',
  streaming: false,
  error: null,
  images: [],
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  check.mockReset()
  // `update/plugin` asks the updater only inside the app, so the test says it is
  // one (the marker `@tauri-apps/api` leaves in the webview); no Tauri is built.
  const globals = globalThis as unknown as Record<string, unknown>
  globals.__TAURI_INTERNALS__ = {}
})

afterEach(() => {
  delete (globalThis as unknown as Record<string, unknown>).__TAURI_INTERNALS__
})

describe("the app's own words", () => {
  it('draws every surface in the stored language, and flips them together', async () => {
    // A newer release, so the update line is one of the drawn surfaces.
    check.mockResolvedValue({ version: '0.2.0', downloadAndInstall: vi.fn() })
    // The stored preference Rust carries: the boot step adopts it, so the first
    // paint is already Spanish and no restart or reload is involved (S1, S4).
    serveSidebar({
      view: viewState({
        language: 'es',
        foreground_chat: 'chat-1',
        foreground_title: TITLE,
        messages: [MESSAGE],
      }),
    })
    await startLanguage()

    render(App)

    // The four surfaces, all from the same slice and the same tick: the rail's
    // gear, the composer's Send, the thread's own name, and the update line.
    expect(await screen.findByRole('button', { name: 'Ajustes' })).toBeTruthy()
    expect(await screen.findByRole('button', { name: 'Enviar' })).toBeTruthy()
    expect(await screen.findByRole('region', { name: 'Hilo del chat' })).toBeTruthy()
    expect(
      await screen.findByRole('button', { name: 'Instalar actualización' }),
    ).toBeTruthy()
    // D7: the title arrived from Rust, so it reads the same in both languages.
    expect(screen.getByRole('heading', { name: TITLE })).toBeTruthy()

    // One switch, one tick: the whole app is English again, which is the words it
    // always drew - nothing of the Spanish column sticks.
    setLanguage('en')
    await tick()

    expect(screen.getByRole('button', { name: 'Settings' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Send' })).toBeTruthy()
    expect(screen.getByRole('region', { name: 'Chat thread' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Install update' })).toBeTruthy()
    expect(screen.getByRole('heading', { name: TITLE })).toBeTruthy()

    // And back once, so the Spanish column is reached from that same switch.
    setLanguage('es')
    await tick()

    expect(screen.getByRole('button', { name: 'Ajustes' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Enviar' })).toBeTruthy()

    setLanguage('en')
  })
})
