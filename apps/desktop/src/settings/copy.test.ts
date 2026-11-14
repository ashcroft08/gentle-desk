// The Settings dialog's own words (0.2.0, S2, S4 and D4): the section titles and
// hints live in a language-keyed dictionary, the rail and the pane read it
// through the reactive language, and choosing a language in the Language section
// redraws them in the SAME tick - no push, no restart, no reload.
//
// The English column is quoted literally, so a change of copy is a change here.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, within } from '@testing-library/svelte'
import { intents, openSettings, pane, pick, rail } from '../test-support/settings'
import { resetTauriMock } from '../test-support/tauri-mock'
import { settingsCopy } from './copy'
import type { Section } from './sections'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The English dictionary, exactly as the dialog drew it before 0.2.0. */
const ENGLISH = {
  languageGroup: 'App language',
  titles: {
    appearance: 'Appearance',
    home: 'Shell home',
    updates: 'Updates',
    profiles: 'Profiles',
    account: 'Account',
    usage: 'Usage',
    language: 'Language',
    about: 'About',
  },
  hints: {
    appearance: 'The look of this window.',
    home: 'Which pi this shell runs on.',
    updates: 'What this machine runs, and whether something newer is out.',
    profiles: 'Your agent profiles, as the terminal keeps them.',
    account: 'Which providers are signed in, and where to sign in or out.',
    usage: "What each subscription has used, as the shell's own sources report it.",
    language: 'The words this app says, in English or Spanish.',
    about: 'Which build this is.',
  },
}

/** The Spanish dictionary: neutral Spanish, no voseo and no regionalisms. */
const SPANISH = {
  languageGroup: 'Idioma de la app',
  titles: {
    appearance: 'Apariencia',
    home: 'Home del shell',
    updates: 'Actualizaciones',
    profiles: 'Perfiles',
    account: 'Cuenta',
    usage: 'Uso',
    language: 'Idioma',
    about: 'Acerca de',
  },
  hints: {
    appearance: 'El aspecto de esta ventana.',
    home: 'En qué pi corre este shell.',
    updates: 'Lo que corre en esta máquina y si hay algo más nuevo.',
    profiles: 'Tus perfiles de agente, tal como los guarda la terminal.',
    account: 'Qué proveedores tienen sesión abierta y dónde iniciar o cerrar sesión.',
    usage: 'Lo que consumió cada suscripción, según las propias fuentes del shell.',
    language: 'Las palabras de esta app, en inglés o en español.',
    about: 'Qué versión es esta.',
  },
}

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

describe('the settings copy', () => {
  it('draws the English words the dialog already had', () => {
    expect(settingsCopy('en')).toEqual(ENGLISH)
  })

  it('draws Spanish for every section of the same shape', () => {
    expect(settingsCopy('es')).toEqual(SPANISH)
  })

  it('translates every word: no section is left in English', () => {
    const english = settingsCopy('en')
    const spanish = settingsCopy('es')

    expect(Object.keys(spanish.titles).sort()).toEqual(Object.keys(english.titles).sort())
    expect(Object.keys(spanish.hints).sort()).toEqual(Object.keys(english.hints).sort())
    for (const section of Object.keys(english.titles) as Section[]) {
      expect(spanish.titles[section], `the title of ${section}`).not.toBe(english.titles[section])
      expect(spanish.hints[section], `the hint of ${section}`).not.toBe(english.hints[section])
    }
  })
})

describe('the Language section', () => {
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

  it('is one of the rail entries, between Usage and About', async () => {
    const dialog = await openSettings()

    expect(rail(dialog).map((entry) => entry.textContent?.trim())).toEqual(RAIL_EN)
  })

  it('names the two languages in their own language, the current one marked', async () => {
    const dialog = await openSettings()

    await pick(dialog, 'Language')
    const choices = within(pane(dialog)).getAllByRole('radio')

    expect(choices.map((choice) => choice.textContent?.trim())).toEqual(['English', 'Español'])
    expect(choices.map((choice) => choice.getAttribute('aria-checked'))).toEqual(['true', 'false'])
  })

  it('flips the dialog own words in the same tick, and sends the one intent', async () => {
    const dialog = await openSettings()
    await pick(dialog, 'Language')

    await fireEvent.click(within(pane(dialog)).getByRole('radio', { name: 'Español' }))

    // No push and no restart: every word the dialog owns is Spanish already.
    expect(rail(dialog).map((entry) => entry.textContent?.trim())).toEqual(RAIL_ES)
    expect(within(pane(dialog)).getByRole('heading', { level: 3 }).textContent).toBe('Idioma')
    expect(within(pane(dialog)).getByText(SPANISH.hints.language)).toBeTruthy()
    expect(
      within(pane(dialog))
        .getAllByRole('radio')
        .map((choice) => choice.getAttribute('aria-checked')),
      'the chosen language is the marked one',
    ).toEqual(['false', 'true'])
    expect(intents()).toEqual([{ type: 'SetLanguage', data: { language: 'es' } }])

    // And back: the words are English again in the same tick, so the flip leaves
    // the language the rest of this file reads where it found it.
    await fireEvent.click(within(pane(dialog)).getByRole('radio', { name: 'English' }))

    expect(rail(dialog).map((entry) => entry.textContent?.trim())).toEqual(RAIL_EN)
    expect(intents()).toEqual([
      { type: 'SetLanguage', data: { language: 'es' } },
      { type: 'SetLanguage', data: { language: 'en' } },
    ])
  })
})
