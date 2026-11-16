// The words of the missing gentle-shell screen (shell-choice S4). It says what
// will run and what it changes before it asks, because the permission only
// means something if the user knows what they are allowing. It also says what
// the command installs - the program, not a home - and that where chats and
// sign-ins live is the question after it, so the install is not read as "all of
// gentle-shell".
//
// 0.2.0 (S3, D4): those words are written in BOTH languages here - one dictionary
// per language and a builder that answers the whole object, so the parametrized
// wait stays a function and no string-keyed lookup is introduced. The English
// column is what the screen has drawn since shell-choice S4, unchanged; the Spanish
// one is neutral - no voseo, no regionalisms - and keeps its accents.
//
// What does NOT move: the COMMAND, which stays the exact line the user must type
// in either language, and npm's own output on a failure, which arrives exactly as
// it was written (D7).
import type { LanguageView } from '../contract/bindings'

/** The screen's own words for one language. */
export interface InstallCopy {
  title: string
  /** Why the screen exists, where it looked, and what the command installs. */
  why: string
  lookedIn: string
  commandIntro: string
  installs: string
  home: string
  warning: string
  /** The two answers the screen offers. */
  install: string
  checkAgain: string
  /** The three states the install already has. */
  running: string
  done: string
  failedTitle: string
  failedHint: string
  /** The progress bar's accessible name while npm works. */
  progress: string
  /**
   * How long the wait has lasted, counted here because npm reports nothing
   * while it works: a number the screen can stand behind, where a percentage
   * would be a guess. Spoken the way the rest of the copy is, in a minute and
   * its seconds.
   */
  elapsed: (seconds: number) => string
}

/** English: the words the screen already drew. */
const ENGLISH: InstallCopy = {
  title: 'gentle-shell is not installed',
  why: 'This app talks to your models through gentle-shell, and it was not found on this machine. Without it nothing here can work.',
  lookedIn:
    'It looked for gentle-shell in the folders of PATH and in the npm global folder (%APPDATA%\\npm). GENTLE_SHELL_BIN can point at it where it lives.',
  commandIntro: 'Installing it runs this command:',
  installs:
    'This installs the gentle-shell program only, about 250 MB, in the npm global folder. It makes no place for chats or sign-ins.',
  home: 'Where your chats and sign-ins live is a separate choice, made next: your own pi, or a space of its own.',
  warning: 'The install changes this computer: it needs Node and npm, and it can take a few minutes.',
  install: 'Install gentle-shell',
  checkAgain: 'Check again',
  running: 'Installing gentle-shell. This can take a few minutes.',
  done: 'gentle-shell was installed. Check again to continue.',
  failedTitle: 'The install did not finish',
  failedHint: 'If npm could not be found, install Node.js (nodejs.org), then press Install again.',
  progress: 'Installing gentle-shell',
  elapsed: (seconds) => {
    const minutes = Math.floor(seconds / 60)
    const rest = seconds % 60
    if (minutes === 0) return `${rest} s elapsed.`
    if (rest === 0) return `${minutes} min elapsed.`
    return `${minutes} min ${rest} s elapsed.`
  },
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH: typeof ENGLISH = {
  title: 'gentle-shell no está instalado',
  why: 'Esta app habla con tus modelos a través de gentle-shell, y no se encontró en esta máquina. Sin él, nada de aquí puede funcionar.',
  lookedIn:
    'Buscó gentle-shell en las carpetas de PATH y en la carpeta global de npm (%APPDATA%\\npm). GENTLE_SHELL_BIN puede apuntar a donde esté.',
  commandIntro: 'Instalarlo ejecuta este comando:',
  installs:
    'Esto instala solo el programa gentle-shell, unos 250 MB, en la carpeta global de npm. No crea un lugar para chats ni para inicios de sesión.',
  home: 'Dónde viven tus chats e inicios de sesión es una elección aparte, que viene después: tu propio pi, o un espacio propio.',
  warning: 'La instalación cambia este equipo: necesita Node y npm, y puede tardar unos minutos.',
  install: 'Instalar gentle-shell',
  checkAgain: 'Comprobar de nuevo',
  running: 'Instalando gentle-shell. Esto puede tardar unos minutos.',
  done: 'gentle-shell se instaló. Comprueba de nuevo para continuar.',
  failedTitle: 'La instalación no terminó',
  failedHint: 'Si no se encontró npm, instala Node.js (nodejs.org) y luego pulsa Instalar de nuevo.',
  progress: 'Instalando gentle-shell',
  // "de espera" does not agree with the count, so one sentence serves every length.
  elapsed: (seconds) => {
    const minutes = Math.floor(seconds / 60)
    const rest = seconds % 60
    if (minutes === 0) return `${rest} s de espera.`
    if (rest === 0) return `${minutes} min de espera.`
    return `${minutes} min ${rest} s de espera.`
  },
}

/** The screen's own words in `language`, the whole dictionary at once (D4). */
export function installCopy(language: LanguageView): typeof ENGLISH {
  return language === 'es' ? SPANISH : ENGLISH
}

/** The one command the button runs, exactly as it must be typed in either language. */
export const COMMAND = 'npm install -g gentle-pi'
