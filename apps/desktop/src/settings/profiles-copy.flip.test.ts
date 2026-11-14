// A Profiles pane DRAWN from its dictionary (0.2.0, S3): choosing Español redraws
// every word the pane owns in the same tick the language changes - no push, no
// restart (S2) - while the profile name, the model names, the role names and the
// shell's own reason keep arriving exactly as they were written (D7).
//
// The English words are asserted literally: they are what the pane has drawn since
// 0.1.12, so a change of copy is a change here. The two columns themselves are
// pinned in `profiles-copy.test.ts`.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, within } from '@testing-library/svelte'
import { tick } from 'svelte'
import type { ProfilesView_Serialize as ProfilesView } from '../contract/bindings'
import { resetTauriMock, setHandler } from '../test-support/tauri-mock'
import { setLanguage } from '../i18n/language.svelte'
import ProfilesPane from './ProfilesPane.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The names pi's own profiles document carries: never translated (D7). */
const NAME = 'Mine'
const MODEL = 'prov/big'
const ROLE = 'worker'

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => null)
  setHandler('open_terminal', () => null)
})

/** The document the user's own machine holds, trimmed to what the pane draws. */
function profiles(over: Partial<ProfilesView> = {}): ProfilesView {
  return {
    status: { type: 'Ready' },
    active: NAME,
    mode: 'organic',
    profiles: [
      {
        name: NAME,
        active: true,
        orchestrator: { role: 'orchestrator', model: MODEL, thinking: 'medium', tier: null },
        roles: [
          { role: 'fallback', model: 'prov/fb', thinking: 'low', tier: null },
          { role: ROLE, model: 'prov/small', thinking: 'high', tier: null },
        ],
        mode: 'organic',
      },
    ],
    ...over,
  }
}

function draw(view: ProfilesView | null = profiles()): HTMLElement {
  const { container } = render(ProfilesPane, { props: { profiles: view } })
  return container as HTMLElement
}

describe('a drawn Profiles pane', () => {
  it('flips the switch, the two sub-tabs, the card and the terminal hand-off', async () => {
    setLanguage('en')
    const pane = draw()

    expect(within(pane).getByRole('tablist').getAttribute('aria-label')).toBe('Profile details')
    expect(
      within(pane)
        .getAllByRole('tab')
        .map((tab) => tab.textContent?.trim()),
    ).toEqual(['Profile', 'Agents'])
    expect(pane.textContent).toContain('Orchestrator mode')
    expect(within(pane).getByRole('radio', { name: 'Organic' })).toBeTruthy()
    expect(within(pane).getByRole('radio', { name: 'Full delegation' })).toBeTruthy()
    expect(pane.textContent).toContain('For chats opened from now on')
    expect(pane.textContent).toContain('Default for new sessions')
    expect(within(pane).getByRole('button', { name: 'Edit in terminal' })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(within(pane).getByRole('tablist').getAttribute('aria-label')).toBe(
      'Detalles del perfil',
    )
    expect(
      within(pane)
        .getAllByRole('tab')
        .map((tab) => tab.textContent?.trim()),
    ).toEqual(['Perfil', 'Agentes'])
    expect(pane.textContent).toContain('Modo de orquestación')
    expect(within(pane).getByRole('radio', { name: 'Orgánico' })).toBeTruthy()
    expect(within(pane).getByRole('radio', { name: 'Delegación total' })).toBeTruthy()
    expect(pane.textContent).toContain('Para los chats abiertos desde ahora')
    expect(pane.textContent).toContain('Predeterminado para las sesiones nuevas')
    expect(within(pane).getByRole('button', { name: 'Editar en la terminal' })).toBeTruthy()
    // D7: the profile's own name and its routes are pi's, word for word.
    expect(pane.textContent).toContain(NAME)
    expect(pane.textContent).toContain(MODEL)

    setLanguage('en')
  })

  it('flips the agent table head and the two route labels, and never a role name', async () => {
    setLanguage('en')
    const pane = draw()
    await fireEvent.click(within(pane).getByRole('tab', { name: 'Agents' }))

    expect(within(pane).getByRole('columnheader', { name: 'Agent' })).toBeTruthy()
    expect(within(pane).getByRole('columnheader', { name: 'Model' })).toBeTruthy()
    expect(within(pane).getByRole('columnheader', { name: 'Effort' })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(within(pane).getByRole('columnheader', { name: 'Agente' })).toBeTruthy()
    expect(within(pane).getByRole('columnheader', { name: 'Modelo' })).toBeTruthy()
    expect(within(pane).getByRole('columnheader', { name: 'Esfuerzo' })).toBeTruthy()
    // The role names are pi's and stay as they arrived; only the two route labels
    // the card draws are ours.
    expect(pane.textContent).toContain(ROLE)
    expect(pane.querySelector('.pair.orchestrator dt')?.textContent).toBe('Orquestador')
    expect(pane.querySelector('.pair.fallback dt')?.textContent).toBe('Fallback')

    setLanguage('en')
  })

  it('flips the reading, empty and failed lines', async () => {
    setLanguage('en')
    const reading = draw(null)
    expect(within(reading).getByText('Reading profiles...')).toBeTruthy()
    const empty = draw(profiles({ active: null, profiles: [] }))
    expect(within(empty).getByText('No profiles found.')).toBeTruthy()
    const failed = draw(
      profiles({ status: { type: 'Failed', data: 'wrong schema' }, active: null, profiles: [] }),
    )
    expect(within(failed).getByText('The profiles could not be read: wrong schema')).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(within(reading).getByText('Leyendo los perfiles...')).toBeTruthy()
    expect(within(empty).getByText('No se encontró ningún perfil.')).toBeTruthy()
    expect(
      within(failed).getByText('No se pudieron leer los perfiles: wrong schema'),
    ).toBeTruthy()
    expect(within(failed).getByRole('button', { name: 'Reintentar' })).toBeTruthy()

    setLanguage('en')
  })
})
