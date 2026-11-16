// The update line's words, in one place.
//
// 0.2.0 (S3, D4): those words are written in BOTH languages here - one dictionary
// per language and a builder that answers the whole object, so every parametrized
// line (the version, the percentage, the failure) stays a function and no
// string-keyed lookup is introduced. The English column is what the line has drawn
// since 0.1.9, unchanged; the Spanish one is neutral - no voseo, no regionalisms -
// and keeps its accents.
//
// What does NOT move: the VERSION the updater reports and its own failure message,
// which are shown exactly as they arrive (D7).
import type { LanguageView } from '../contract/bindings'

/** The update line's own words for one language. */
export interface UpdateCopy {
  /**
   * A newer release, said before the button is pressed: on Windows the updater
   * closes the app by itself to hand over to the installer.
   */
  found: (version: string) => string
  /** The download, with its percentage when the server sent one. */
  downloading: (version: string, percent: number | null) => string
  /** Downloaded and verified: the next start runs it. */
  ready: (version: string) => string
  /** The install failed; `version` is null when no release was ever named. */
  failed: (version: string | null, message: string) => string
  /**
   * The one reason the line spells itself: a retry asked the release server and
   * found nothing newer to install. The plugin's own messages stay as they
   * arrive (D7), so this is the only failure that needs a word of ours.
   */
  noNewerVersion: string
  /** The three answers the line offers. */
  install: string
  retry: string
  dismiss: string
}

/** English: the words the line already drew. */
const ENGLISH: UpdateCopy = {
  found: (version) => `Version ${version} is available. The app will close while it installs.`,
  downloading: (version, percent) =>
    percent === null
      ? `Downloading version ${version}.`
      : `Downloading version ${version}: ${percent}%.`,
  ready: (version) => `Version ${version} is installed. Open the app again to use it.`,
  failed: (version, message) =>
    `The update${version === null ? '' : ` to version ${version}`} could not be installed: ${message}`,
  noNewerVersion: 'no newer version was found',
  install: 'Install update',
  retry: 'Try again',
  dismiss: 'Not now',
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH: typeof ENGLISH = {
  found: (version) =>
    `La versión ${version} está disponible. La app se cerrará mientras se instala.`,
  downloading: (version, percent) =>
    percent === null
      ? `Descargando la versión ${version}.`
      : `Descargando la versión ${version}: ${percent}%.`,
  ready: (version) => `La versión ${version} está instalada. Abre la app de nuevo para usarla.`,
  failed: (version, message) =>
    `No se pudo instalar la actualización${version === null ? '' : ` a la versión ${version}`}: ${message}`,
  noNewerVersion: 'no se encontró una versión más nueva',
  install: 'Instalar actualización',
  retry: 'Reintentar',
  dismiss: 'Ahora no',
}

/** The line's own words in `language`, the whole dictionary at once (D4). */
export function updateCopy(language: LanguageView): typeof ENGLISH {
  return language === 'es' ? SPANISH : ENGLISH
}
