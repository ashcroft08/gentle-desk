// The welcome question's words, after the gentle-shell welcome screen the user
// pointed to (shell-choice S2). It says nothing about what is inside pi: the app
// only checked that the folder exists, so the only fact it names is the path.
//
// 0.2.0 (S3, D4): those words are written in BOTH languages here - one dictionary
// per language and a builder that answers the whole object, so the two lists of
// points stay arrays and no string-keyed lookup is introduced. The English column
// is what the screen has drawn since shell-choice S2, unchanged; the Spanish one is
// neutral - no voseo, no regionalisms - and keeps its accents.
//
// What does NOT move: the FOLDER the machine reported, which is a path and is drawn
// exactly as it was found (D7).
import type { LanguageView } from '../contract/bindings'

/** The screen's own words for one language. */
export interface WelcomeCopy {
  title: string
  intro: string
  foundTitle: string
  /** The recommended answer: its title, its badge, its points and its path. */
  linkTitle: string
  linkBadge: string
  linkPoints: string[]
  linkPath: string
  /** The other answer, drawn with the same four parts. */
  separateTitle: string
  separatePoints: string[]
  separatePath: string
  /** Shown from the click until the app starts again with the chosen home. */
  applying: string
  skipNote: string
}

/** English: the words the screen already drew. */
const ENGLISH: WelcomeCopy = {
  title: 'Welcome to gentle shell',
  intro: 'Everything you need is already inside this app. One question before you start.',
  foundTitle: 'We found pi on this machine',
  linkTitle: 'Use my pi setup',
  linkBadge: 'RECOMMENDED',
  linkPoints: [
    'Your sign-ins, local models and chats appear here',
    'Chats you start here also show up in your terminal',
    'Your pi settings are never edited',
  ],
  linkPath: 'links to ~/.pi/agent',
  separateTitle: 'Keep it separate',
  separatePoints: [
    'A clean space with its own sign-ins and chats',
    'Nothing on this machine is touched',
    'You can change this later from Settings',
  ],
  separatePath: 'creates ~/.gentle-shell/agent',
  applying: 'Applying your choice. The app will start again in a moment.',
  skipNote:
    'No pi on the machine? This screen is skipped and gentle shell starts in its own space.',
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH: typeof ENGLISH = {
  title: 'Te damos la bienvenida a gentle shell',
  intro: 'Todo lo que necesitas ya está dentro de esta app. Una pregunta antes de empezar.',
  foundTitle: 'Encontramos pi en esta máquina',
  linkTitle: 'Usar mi configuración de pi',
  linkBadge: 'RECOMENDADO',
  linkPoints: [
    'Tus inicios de sesión, modelos locales y chats aparecen aquí',
    'Los chats que inicies aquí también aparecen en tu terminal',
    'Tu configuración de pi nunca se modifica',
  ],
  linkPath: 'enlaza con ~/.pi/agent',
  separateTitle: 'Mantenerlo separado',
  separatePoints: [
    'Un espacio limpio con sus propios inicios de sesión y chats',
    'No se toca nada de esta máquina',
    // The dialog's own Spanish name, so one word has one owner (settings/copy.ts).
    'Puedes cambiarlo más tarde en Ajustes',
  ],
  separatePath: 'crea ~/.gentle-shell/agent',
  applying: 'Aplicando tu elección. La app se reiniciará en un momento.',
  skipNote:
    '¿No hay pi en la máquina? Esta pantalla se omite y gentle shell arranca en su propio espacio.',
}

/** The screen's own words in `language`, the whole dictionary at once (D4). */
export function welcomeCopy(language: LanguageView): typeof ENGLISH {
  return language === 'es' ? SPANISH : ENGLISH
}
