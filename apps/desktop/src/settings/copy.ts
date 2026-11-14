// The Settings dialog's words (settings-dialog S2-S6; 0.1.12 S1-S7; 0.2.0 S2/S4).
// The dialog's own rail - each section's title and its one-line hint - is written
// in BOTH languages here, because it is the section that changes the language:
// choosing one has to redraw these words in the same tick (0.2.0, S2), so they
// cannot be a single set of constants (D4).
//
// The shape is one dictionary per language and a builder that answers the whole
// object for a language, so every entry stays a plain string and no string-keyed
// lookup is introduced. The English column is what the dialog has drawn since
// settings-dialog L4, unchanged; the Spanish one is neutral - no voseo, no
// regionalisms - and keeps its accents.
//
// The rest of the dialog's words (the two homes' explanation, the app's name, the
// version line, Close) are still one language each: 0.2.0's T7 migrates the rest
// of the Settings area, and this slice owns the rail's own words.
import type { LanguageView } from '../contract/bindings'
import type { Section } from './sections'

/** The dialog's own words for one language: the rail's titles and hints. */
export interface SettingsCopy {
  /** The name of the Language section's choice group. */
  languageGroup: string
  /** Each section's word, for the rail entry and the pane's heading. */
  titles: Record<Section, string>
  /** Each section's one-line hint, drawn at the top of its pane (L4). */
  hints: Record<Section, string>
}

/** English: the words the dialog already drew. */
const ENGLISH: SettingsCopy = {
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

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH: typeof ENGLISH = {
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

/** The dialog's own words in `language`, the whole dictionary at once (D4). */
export function settingsCopy(language: LanguageView): typeof ENGLISH {
  return language === 'es' ? SPANISH : ENGLISH
}

/** The gear's name and the dialog's title: the same word for both (S5, S2). */
export const SETTINGS = 'Settings'

/**
 * The full two-home explanation, the same two answers the welcome screen gives
 * (shell-choice S2) now that the control lives where a restart-needing choice is
 * not clicked by accident. The restart line itself is the home setting's own,
 * drawn while a change waits.
 */
export const HOME_EXPLAIN =
  'My pi uses the pi installation already on the machine: sign-ins, local models and chats appear in the terminal. Separate creates a clean space at ~/.gentle-shell/agent and touches nothing else.'

/** The app's name, as the About section says it (S5): the same name the window's
 * title and the installer carry. */
export const APP_NAME = 'Gentle Shell Desktop'

/** The running version, from the bridge: the one this run runs as (S5). */
export const VERSION = (version: string): string => `Version ${version}`

/** What the version line says while the bridge has named none: never a guess. */
export const VERSION_UNKNOWN = 'Version unknown'

export const CLOSE = 'Close'
