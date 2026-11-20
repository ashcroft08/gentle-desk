// The Settings dialog's own words (0.2.0, S2, S3 and D4): the dialog's title, the
// rail's accessible name, each section's title and hint, the two-home explanation,
// the app's name, the version line and Close live in ONE language-keyed dictionary,
// the same shape the sidebar took in T6. This file pins the two columns and the one
// line that carries a version; the words a DRAWN dialog shows are pinned in
// `copy.flip.test.ts`.
//
// The English column is quoted literally: it is what the dialog has drawn since
// settings-dialog L4, so a change of copy is a change here. The version NUMBER is
// not a word of this module: it arrives from the bridge and is only wrapped (D7).
import { describe, expect, it } from 'vitest'
import { settingsCopy } from './copy'
import type { Section } from './sections'

/** The English words, exactly as the dialog drew them before 0.2.0. */
const ENGLISH = {
  languageGroup: 'App language',
  settings: 'Settings',
  railLabel: 'Settings sections',
  searchLabel: 'Find a setting',
  searchPlaceholder: 'Find a setting…',
  noMatches: 'No settings match.',
  themeGroup: 'Theme',
  homeExplain:
    'My pi uses the pi installation already on the machine: sign-ins, local models and chats appear in the terminal. Separate creates a clean space at ~/.gentle-shell/agent and touches nothing else.',
  appName: 'Gentle Shell Desktop',
  versionUnknown: 'Version unknown',
  close: 'Close',
}

/** The Spanish words: neutral Spanish, accents allowed. */
const SPANISH = {
  languageGroup: 'Idioma de la app',
  settings: 'Ajustes',
  railLabel: 'Secciones de ajustes',
  searchLabel: 'Buscar un ajuste',
  searchPlaceholder: 'Buscar un ajuste…',
  noMatches: 'Ningún ajuste coincide.',
  themeGroup: 'Tema',
  homeExplain:
    'Mi pi usa la instalación de pi que ya está en la máquina: los inicios de sesión, los modelos locales y los chats aparecen en la terminal. Separado crea un espacio limpio en ~/.gentle-shell/agent y no toca nada más.',
  appName: 'Gentle Shell Desktop',
  versionUnknown: 'Versión desconocida',
  close: 'Cerrar',
}

/** Each section's word in the rail and its one-line hint, per language. */
const ENGLISH_TITLES: Record<Section, string> = {
  appearance: 'Appearance',
  home: 'Shell home',
  updates: 'Updates',
  account: 'Account',
  usage: 'Usage',
  language: 'Language',
  about: 'About',
}

const SPANISH_TITLES: Record<Section, string> = {
  appearance: 'Apariencia',
  home: 'Home del shell',
  updates: 'Actualizaciones',
  account: 'Cuenta',
  usage: 'Uso',
  language: 'Idioma',
  about: 'Acerca de',
}

const ENGLISH_HINTS: Record<Section, string> = {
  appearance: 'The look of this window.',
  home: 'Which pi this shell runs on.',
  updates: 'What this machine runs, and whether something newer is out.',
  account: 'Which providers are signed in, and where to sign in or out.',
  usage: "What each subscription has used, as the shell's own sources report it.",
  language: 'The words this app says, in English or Spanish.',
  about: 'Which build this is.',
}

const SPANISH_HINTS: Record<Section, string> = {
  appearance: 'El aspecto de esta ventana.',
  home: 'En qué pi corre este shell.',
  updates: 'Lo que corre en esta máquina y si hay algo más nuevo.',
  account: 'Qué proveedores tienen sesión abierta y dónde iniciar o cerrar sesión.',
  usage: 'Lo que consumió cada suscripción, según las propias fuentes del shell.',
  language: 'Las palabras de esta app, en inglés o en español.',
  about: 'Qué versión es esta.',
}

/** The plain words of one language: the two records are pinned below them. */
const WORDS = Object.keys(ENGLISH) as (keyof typeof ENGLISH)[]

/** The app's own NAME is the same in both languages: a name is never translated. */
const SAME_IN_BOTH = new Set<keyof typeof ENGLISH>(['appName'])

/** The words of one language, as a template reads them. */
function words(language: 'en' | 'es'): Record<string, string> {
  const copy = settingsCopy(language)
  return Object.fromEntries(WORDS.map((word) => [word, copy[word]]))
}

describe('the settings copy', () => {
  it('draws the English words the dialog already had', () => {
    const copy = settingsCopy('en')

    expect(words('en')).toEqual(ENGLISH)
    expect(copy.titles).toEqual(ENGLISH_TITLES)
    expect(copy.hints).toEqual(ENGLISH_HINTS)
    expect(copy.version('0.1.9')).toBe('Version 0.1.9')
  })

  it('draws Spanish for the same shape', () => {
    const copy = settingsCopy('es')

    expect(words('es')).toEqual(SPANISH)
    expect(copy.titles).toEqual(SPANISH_TITLES)
    expect(copy.hints).toEqual(SPANISH_HINTS)
    expect(copy.version('0.1.9')).toBe('Versión 0.1.9')
  })

  it('translates every word: nothing is left in English', () => {
    const english = settingsCopy('en')
    const spanish = settingsCopy('es')

    expect(Object.keys(spanish).sort()).toEqual(Object.keys(english).sort())
    for (const word of WORDS) {
      if (SAME_IN_BOTH.has(word)) {
        expect(spanish[word], word).toBe(english[word])
        continue
      }
      expect(spanish[word], word).not.toBe(english[word])
    }
    for (const section of Object.keys(english.titles) as Section[]) {
      expect(spanish.titles[section], `the title of ${section}`).not.toBe(english.titles[section])
      expect(spanish.hints[section], `the hint of ${section}`).not.toBe(english.hints[section])
    }
  })
})
