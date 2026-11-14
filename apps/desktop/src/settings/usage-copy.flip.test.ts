// A Usage pane DRAWN from its dictionary (0.2.0, S3): choosing Español redraws
// every word the pane owns in the same tick the language changes - no push, no
// restart (S2) - while the provider names, the plan names, the window labels, the
// shares and the countdowns the shell reported keep arriving exactly as they were
// written (D7).
//
// The English words are asserted literally: they are what the pane has drawn since
// 0.1.12, so a change of copy is a change here. The two columns themselves are
// pinned in `usage-copy.test.ts`.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, within } from '@testing-library/svelte'
import { tick } from 'svelte'
import type { UsageView_Serialize as UsageView } from '../contract/bindings'
import { resetTauriMock, setHandler } from '../test-support/tauri-mock'
import { setLanguage } from '../i18n/language.svelte'
import UsagePane from './UsagePane.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The shell's own readings: a provider name, a window, a share, a countdown (D7). */
const PROVIDER = 'commandcode'
const WINDOW = '5h'
const SHARE = '5%'
const RESET = 'resets in 2h'

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => null)
  setHandler('open_terminal', () => null)
})

/** The real document this machine's sources printed, trimmed to one provider. */
function installed(): UsageView {
  return {
    status: { type: 'Installed' },
    providers: [
      {
        provider: PROVIDER,
        status: 'Ok',
        plan: 'individual goat',
        error: null,
        limits: [
          {
            name: 'command code',
            limit_reached: false,
            windows: [{ label: WINDOW, used_percent: 4.6771898400000005, resets_in: RESET }],
          },
          {
            name: 'credits',
            limit_reached: true,
            windows: [{ label: 'billing', used_percent: 44.13765581735515, resets_in: 'resets in 30d' }],
          },
        ],
      },
    ],
  }
}

function draw(view: UsageView | null = installed()): HTMLElement {
  const { container } = render(UsagePane, { props: { usage: view } })
  return container as HTMLElement
}

describe('a drawn Usage pane', () => {
  it('flips its tag, its button and its note, and never a reading', async () => {
    setLanguage('en')
    const pane = draw()

    expect(pane.querySelector('.tag')?.textContent?.trim()).toBe('Limit reached')
    expect(within(pane).getByRole('button', { name: 'Check again' })).toBeTruthy()
    expect(pane.textContent).toContain("These numbers come from the shell's own sources")

    setLanguage('es')
    await tick()

    expect(pane.querySelector('.tag')?.textContent?.trim()).toBe('Límite alcanzado')
    expect(within(pane).getByRole('button', { name: 'Comprobar de nuevo' })).toBeTruthy()
    expect(pane.textContent).toContain('Estos números vienen de las propias fuentes de la shell')
    // D7: every reading the shell sent keeps its own words and numbers.
    expect(pane.textContent).toContain(PROVIDER)
    expect(pane.textContent).toContain(WINDOW)
    expect(pane.textContent).toContain(SHARE)
    expect(pane.textContent).toContain(RESET)
    // `Plan` is the same word in Spanish: the plan name is the shell's either way.
    expect(pane.textContent).toContain('Plan: individual goat')

    setLanguage('en')
  })

  it('flips the missing-sources state and its one terminal action', async () => {
    setLanguage('en')
    const pane = draw({ status: { type: 'Missing', data: 'not installed' }, providers: [] })

    expect(within(pane).getByText('The usage sources are not installed.')).toBeTruthy()
    expect(within(pane).getByRole('button', { name: 'Install in terminal' })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(within(pane).getByText('Las fuentes de uso no están instaladas.')).toBeTruthy()
    expect(within(pane).getByRole('button', { name: 'Instalar en la terminal' })).toBeTruthy()
    expect(pane.textContent).toContain('Son las propias fuentes de la shell')

    setLanguage('en')
  })

  it('flips the reading and the unreadable lines around the shell own reason', async () => {
    setLanguage('en')
    const reading = draw({ status: { type: 'Reading' }, providers: [] })
    expect(within(reading).getByText('Reading the usage...')).toBeTruthy()
    const unreadable = draw({ status: { type: 'Unreadable', data: 'permission denied' }, providers: [] })
    expect(within(unreadable).getByText('Could not read the usage: permission denied')).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(within(reading).getByText('Leyendo el uso...')).toBeTruthy()
    expect(
      within(unreadable).getByText('No se pudo leer el uso: permission denied'),
    ).toBeTruthy()

    setLanguage('en')
  })
})
