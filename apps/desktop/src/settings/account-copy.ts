// The Account pane's words (0.1.12, S1, T3, L11): provider NAMES and what the
// shell's own check said about each one. No account identity is shown, because
// the shell does not expose one (L11), and no credential ever reaches the page.
//
// 0.2.0 (S3, D4): those words are written in BOTH languages here - one dictionary
// per language and a builder that answers the whole object, so a parametrized line
// (the failure reason after its colon) stays a function and no string-keyed lookup
// is introduced. The English column is what the pane has drawn since 0.1.12,
// unchanged; the Spanish one is neutral - no voseo, no regionalisms - and keeps
// its accents.
//
// What does NOT move: the provider names and the reason each check reported, which
// are the shell's own words and arrive exactly as written (D7).
import type { LanguageView } from '../contract/bindings'

/** The Account pane's own words for one language. */
export interface AccountCopy {
  /** The two states the pane draws before it has an answer. */
  reading: string
  failed: (reason: string) => string
  retrySafe: string
  noProviders: string
  /** The one check button, and the one terminal hand-off. */
  checkAgain: string
  openShell: string
  note: string
  /** One provider's check, in the words the pane draws (the state, then its reason). */
  connected: string
  unverified: string
  notReady: string
  invalid: string
  error: string
}

/** English: the words the pane already drew. */
const ENGLISH: AccountCopy = {
  reading: 'Reading the sign-ins...',
  failed: (reason) => `Could not read the sign-ins: ${reason}`,
  retrySafe: 'Checking again is safe.',
  noProviders: 'No providers are signed in.',
  checkAgain: 'Check again',
  openShell: 'Open shell to sign in or out',
  note: 'The shell opens in a terminal window: type /login to sign in or /logout to sign out, then press Check again here.',
  connected: 'Connected',
  unverified: 'Signed in (not checkable)',
  notReady: 'Not ready',
  invalid: 'Sign-in is not valid',
  error: 'Could not check',
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH: typeof ENGLISH = {
  reading: 'Leyendo los inicios de sesión...',
  failed: (reason) => `No se pudieron leer los inicios de sesión: ${reason}`,
  retrySafe: 'Volver a comprobar es seguro.',
  noProviders: 'No hay proveedores con sesión iniciada.',
  checkAgain: 'Comprobar de nuevo',
  openShell: 'Abrir la shell para iniciar o cerrar sesión',
  note: 'La shell se abre en una ventana de terminal: escribe /login para iniciar sesión o /logout para cerrarla, y luego pulsa Comprobar de nuevo aquí.',
  connected: 'Conectado',
  unverified: 'Sesión iniciada (no verificable)',
  notReady: 'No está listo',
  invalid: 'La sesión no es válida',
  error: 'No se pudo comprobar',
}

/** The pane's own words in `language`, the whole dictionary at once (D4). */
export function accountCopy(language: LanguageView): typeof ENGLISH {
  return language === 'es' ? SPANISH : ENGLISH
}
