// An Account pane DRAWN from its dictionary (0.2.0, S3): choosing Español redraws
// every word the pane owns in the same tick the language changes - no push, no
// restart (S2) - while the provider names and the shell's own reason keep arriving
// exactly as they were written (D7).
//
// The English words are asserted literally: they are what the pane has drawn since
// 0.1.12, so a change of copy is a change here. The two columns themselves are
// pinned in `account-copy.test.ts`.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, within } from '@testing-library/svelte'
import { tick } from 'svelte'
import type { AuthView_Serialize as AuthView } from '../contract/bindings'
import { resetTauriMock, setHandler } from '../test-support/tauri-mock'
import { setLanguage } from '../i18n/language.svelte'
import AccountPane from './AccountPane.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The two names the shell's own store lists: never translated (D7). */
const PROVIDER = 'anthropic'
const OTHER = 'commandcode'
/** The shell's own reason for a check that did not pass: never translated (D7). */
const REASON = 'OAuth token expired'

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => null)
  setHandler('open_terminal', () => null)
})

/** The two rows the pane draws: one connected, one not ready. */
function auth(over: Partial<AuthView> = {}): AuthView {
  return {
    status: { type: 'Ready' },
    providers: [
      { name: PROVIDER, status: 'Ready', reason: null },
      { name: OTHER, status: 'NotReady', reason: REASON },
    ],
    ...over,
  }
}

function draw(view: AuthView | null = auth()): HTMLElement {
  const { container } = render(AccountPane, { props: { auth: view } })
  return container as HTMLElement
}

describe('a drawn Account pane', () => {
  it('flips its own words, and leaves the provider names and reasons alone', async () => {
    setLanguage('en')
    const pane = draw()

    expect(pane.textContent).toContain('Connected')
    expect(pane.textContent).toContain('Not ready')
    expect(within(pane).getByRole('button', { name: 'Check again' })).toBeTruthy()
    expect(within(pane).getByRole('button', { name: 'Open shell to sign in or out' })).toBeTruthy()
    expect(pane.textContent).toContain('type /login to sign in')

    setLanguage('es')
    await tick()

    expect(pane.textContent).toContain('Conectado')
    expect(pane.textContent).toContain('No está listo')
    expect(within(pane).getByRole('button', { name: 'Comprobar de nuevo' })).toBeTruthy()
    expect(
      within(pane).getByRole('button', { name: 'Abrir la shell para iniciar o cerrar sesión' }),
    ).toBeTruthy()
    expect(pane.textContent).toContain('escribe /login para iniciar sesión')
    // D7: the names and the reason are the shell's, so they read the same twice.
    expect(pane.textContent).toContain(PROVIDER)
    expect(pane.textContent).toContain(OTHER)
    expect(pane.textContent).toContain(REASON)

    setLanguage('en')
  })

  it('flips the empty and the reading line', async () => {
    setLanguage('en')
    const empty = draw(auth({ providers: [] }))
    expect(within(empty).getByText('No providers are signed in.')).toBeTruthy()
    const reading = draw(null)
    expect(within(reading).getByText('Reading the sign-ins...')).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(within(empty).getByText('No hay proveedores con sesión iniciada.')).toBeTruthy()
    expect(within(reading).getByText('Leyendo los inicios de sesión...')).toBeTruthy()

    setLanguage('en')
  })

  it('flips the failed line around the shell own reason', async () => {
    setLanguage('en')
    const pane = draw(auth({ status: { type: 'Failed', data: 'permission denied' } }))
    expect(within(pane).getByText('Could not read the sign-ins: permission denied')).toBeTruthy()
    expect(within(pane).getByText('Checking again is safe.')).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(
      within(pane).getByText('No se pudieron leer los inicios de sesión: permission denied'),
    ).toBeTruthy()
    expect(within(pane).getByText('Volver a comprobar es seguro.')).toBeTruthy()

    setLanguage('en')
  })
})
