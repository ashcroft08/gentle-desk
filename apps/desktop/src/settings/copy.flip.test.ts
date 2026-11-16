// A Settings dialog DRAWN from its dictionary (0.2.0, S2, S3): choosing Español
// redraws every word the dialog owns in the same tick the language changes - no
// push, no restart (S2) - while the version NUMBER the bridge sent keeps arriving
// exactly as it was written (D7).
//
// The English words are asserted literally: they are what the dialog has drawn
// since settings-dialog L4, so a change of copy is a change here. The two columns
// themselves are pinned in `copy.test.ts`.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, within } from '@testing-library/svelte'
import { tick } from 'svelte'
import { openSettings, pane, pick, rail } from '../test-support/settings'
import { resetTauriMock } from '../test-support/tauri-mock'
import { setLanguage } from '../i18n/language.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The rail's words, in the order the dialog draws them. */
const RAIL_EN = [
  'Appearance',
  'Shell home',
  'Updates',
  'Profiles',
  'Account',
  'Usage',
  'Language',
  'About',
]
const RAIL_ES = [
  'Apariencia',
  'Home del shell',
  'Actualizaciones',
  'Perfiles',
  'Cuenta',
  'Uso',
  'Idioma',
  'Acerca de',
]

/** The two-home line, drawn by the section that holds the choice. */
const EXPLAIN_EN =
  'My pi uses the pi installation already on the machine: sign-ins, local models and chats appear in the terminal. Separate creates a clean space at ~/.gentle-shell/agent and touches nothing else.'
const EXPLAIN_ES =
  'Mi pi usa la instalación de pi que ya está en la máquina: los inicios de sesión, los modelos locales y los chats aparecen en la terminal. Separado crea un espacio limpio en ~/.gentle-shell/agent y no toca nada más.'

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

/** The dialog's title, the rail's name and each entry's word, as they read now. */
function words(dialog: HTMLElement): { title: string; railName: string | null; rail: string[] } {
  return {
    title: within(dialog).getByRole('heading', { level: 2 }).textContent?.trim() ?? '',
    railName: within(dialog).getByRole('tablist').getAttribute('aria-label'),
    rail: rail(dialog).map((entry) => entry.textContent?.trim() ?? ''),
  }
}

describe('a drawn Settings dialog', () => {
  it('flips its title, its rail name and every rail entry in the same tick', async () => {
    setLanguage('en')
    const dialog = await openSettings()

    expect(words(dialog)).toEqual({
      title: 'Settings',
      railName: 'Settings sections',
      rail: RAIL_EN,
    })
    expect(within(pane(dialog)).getByRole('heading', { level: 3 }).textContent).toBe('Appearance')
    expect(within(pane(dialog)).getByText('The look of this window.')).toBeTruthy()
    expect(within(dialog).getByRole('button', { name: 'Close' })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(words(dialog)).toEqual({
      title: 'Ajustes',
      railName: 'Secciones de ajustes',
      rail: RAIL_ES,
    })
    expect(within(pane(dialog)).getByRole('heading', { level: 3 }).textContent).toBe('Apariencia')
    expect(within(pane(dialog)).getByText('El aspecto de esta ventana.')).toBeTruthy()
    expect(within(dialog).getByRole('button', { name: 'Cerrar' })).toBeTruthy()

    setLanguage('en')
  })

  it('flips the two-home explanation while the home names it draws stay the shell own', async () => {
    setLanguage('en')
    const dialog = await openSettings({ choice: 'link' })
    await pick(dialog, 'Shell home')
    expect(within(pane(dialog)).getByText(EXPLAIN_EN)).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(within(pane(dialog)).getByText(EXPLAIN_ES)).toBeTruthy()
    // The control under it is the home setting's own (sidebar/home-copy.ts), which
    // flips with the same language: one word, one owner.
    expect(within(pane(dialog)).getByRole('button', { name: 'Mi pi' })).toBeTruthy()

    setLanguage('en')
  })

  it('flips the About line and never renames the version the bridge sent', async () => {
    setLanguage('en')
    const dialog = await openSettings({}, '0.1.9')
    await pick(dialog, 'About')
    expect(within(pane(dialog)).getByText('Version 0.1.9')).toBeTruthy()

    setLanguage('es')
    await tick()

    await pick(dialog, 'Acerca de')
    expect(pane(dialog).textContent).toContain('Gentle Shell Desktop')
    expect(within(pane(dialog)).getByText('Versión 0.1.9')).toBeTruthy()
    // D7: the number is the bridge's, so it reads the same in both languages.
    expect(pane(dialog).textContent).toContain('0.1.9')

    setLanguage('en')
  })

  it('says it does not know the version in the language of the dialog', async () => {
    setLanguage('en')
    const dialog = await openSettings({}, null)
    await pick(dialog, 'About')
    expect(within(pane(dialog)).getByText('Version unknown')).toBeTruthy()

    setLanguage('es')
    await tick()

    await pick(dialog, 'Acerca de')
    expect(within(pane(dialog)).getByText('Versión desconocida')).toBeTruthy()

    setLanguage('en')
  })

  it('keeps the language choice itself out of the flip, and sends the one intent', async () => {
    setLanguage('en')
    const dialog = await openSettings()
    await pick(dialog, 'Language')

    await fireEvent.click(within(pane(dialog)).getByRole('radio', { name: 'Español' }))

    // Every word the dialog owns is Spanish already, in the tick of the choice.
    expect(rail(dialog).map((entry) => entry.textContent?.trim())).toEqual(RAIL_ES)
    expect(within(pane(dialog)).getByRole('heading', { level: 3 }).textContent).toBe('Idioma')
    // A language's own name is never translated: each choice keeps its own.
    expect(
      within(pane(dialog))
        .getAllByRole('radio')
        .map((choice) => choice.textContent?.trim()),
    ).toEqual(['English', 'Español'])

    await fireEvent.click(within(pane(dialog)).getByRole('radio', { name: 'English' }))
    expect(rail(dialog).map((entry) => entry.textContent?.trim())).toEqual(RAIL_EN)
  })
})
