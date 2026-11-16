// The language of the app's own words (0.2.0, S1/S2/S4/D2): a reactive value the
// whole web side reads, fed by the state Rust pushes, with the machine's own
// language standing in while the config stores none - and the one report that
// makes Rust store what the machine said.
//
// Every test boots a FRESH copy of the module, because the boot step runs on
// import and its one report belongs to a module lifetime: `vi.resetModules()`
// gives each test that lifetime, and the mock the fresh module talks to is the
// one this test imports after the reset (the factory binds whatever copy exists
// then, so handlers must be set on THAT copy).
import '../test-support/dom-env'
import { describe, expect, it, vi } from 'vitest'
import type { LanguageView } from '../contract/bindings'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

type Mock = typeof import('../test-support/tauri-mock')
type LanguageModule = typeof import('./language.svelte')

/** What the machine says its language is, for one test. */
function setLocale(locale: string): void {
  Object.defineProperty(window.navigator, 'language', { value: locale, configurable: true })
}

/** Lets the boot step's own read land before a test asserts on it. */
async function settle(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, 0))
}

/**
 * A fresh module booted the way the app boots it: the machine's locale first,
 * then the state the bridge answers with. `stored` is the language that state
 * carries, `'absent'` for a bridge that answers nothing at all.
 */
async function boot(
  locale: string,
  stored: LanguageView | null | 'absent' = 'absent',
): Promise<{ module: LanguageModule; mock: Mock }> {
  vi.resetModules()
  // The mock of one generation: `vi.resetModules` gives the app modules a fresh
  // registry but keeps the factory a test file registered, so the factory is
  // re-registered here over the copy this generation imported.
  const mock = await import('../test-support/tauri-mock')
  vi.doMock('@tauri-apps/api/core', () => mock.coreModule())
  vi.doMock('@tauri-apps/api/event', () => mock.eventModule())
  mock.resetTauriMock()
  setLocale(locale)
  if (stored !== 'absent') {
    mock.setHandler('get_view_state', () => mock.viewState({ language: stored }))
  }
  const module = await import('./language.svelte')
  await settle()
  return { module, mock }
}

/** The `SetLanguage` intents the bridge received, oldest first. */
function reports(mock: Mock): unknown[] {
  return mock
    .handlerArgs('dispatch_intent')
    .map((args) => args.intent)
    .filter((intent) => (intent as { type?: string }).type === 'SetLanguage')
}

describe('the locale rule', () => {
  it('is the Rust rule: exactly a Spanish primary subtag starts in Spanish', async () => {
    const { module } = await boot('en-US')
    // The same boundaries `gentle_core::i18n::Language::detect_from_locale` pins:
    // the primary subtag (before the first `-` or `_`), trimmed and lowercased.
    for (const locale of ['es', 'es-ES', 'es-419', 'es_MX', 'ES-ar', '  es  ']) {
      expect(module.detectLanguage(locale), locale).toBe('es')
    }
    for (const locale of ['en', 'en-US', 'de', 'fr', '', '   ', 'e', 'eses']) {
      expect(module.detectLanguage(locale), locale).toBe('en')
    }
    expect(module.detectLanguage(null), 'no locale at all').toBe('en')
    expect(module.detectLanguage(undefined), 'and none named').toBe('en')
  })
})

describe('the reactive language', () => {
  it('starts from the machine language while the config stores none', async () => {
    const { module } = await boot('es-AR', null)

    expect(module.language.current).toBe('es')
  })

  it('lets a stored preference win over the machine language', async () => {
    const stored = await boot('es-AR', 'en')
    expect(stored.module.language.current, 'a stored English survives an es machine').toBe('en')

    const spanish = await boot('en-US', 'es')
    expect(spanish.module.language.current, 'and a stored Spanish an en machine').toBe('es')
  })

  it('follows every pushed state, so a change needs no restart', async () => {
    const { module, mock } = await boot('en-US', 'en')

    mock.pushEvent('state-changed', mock.viewState({ language: 'es' }))
    expect(module.language.current).toBe('es')

    mock.pushEvent('state-changed', mock.viewState({ language: 'en' }))
    expect(module.language.current).toBe('en')
  })

  it('flips in this same tick when the user chooses, and sends the one intent', async () => {
    const { module, mock } = await boot('en-US', 'en')

    module.setLanguage('es')

    expect(module.language.current, 'the words change before the bridge answers').toBe('es')
    expect(mock.handlerArgs('dispatch_intent').map((args) => args.intent)).toEqual([
      { type: 'SetLanguage', data: { language: 'es' } },
    ])
  })

  it('reports a Spanish machine once, so Rust stores the language it detected', async () => {
    const { mock } = await boot('es-MX', null)

    expect(reports(mock), 'the boot report').toEqual([
      { type: 'SetLanguage', data: { language: 'es' } },
    ])

    // A second state without a preference adds nothing: the report is one-shot,
    // so a push never turns into a write loop.
    mock.pushEvent('state-changed', mock.viewState({ language: null }))
    await settle()
    expect(reports(mock)).toHaveLength(1)
  })

  it('stores nothing for an English machine: absence already means English', async () => {
    const { mock } = await boot('en-GB', null)

    expect(mock.handlerArgs('dispatch_intent')).toEqual([])
  })

  it('survives a bridge that cannot answer: the machine language still stands', async () => {
    const { module } = await boot('es-ES', 'absent')

    expect(module.language.current).toBe('es')
  })
})
