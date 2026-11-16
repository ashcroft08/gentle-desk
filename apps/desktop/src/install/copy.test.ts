// The missing-shell screen's own words (0.2.0, S3 and D4): what the screen says
// before it asks, its two buttons, its three install states and the wait it
// counts live in ONE language-keyed dictionary. This file pins the two columns;
// the words a DRAWN screen shows are pinned in `copy.flip.test.ts`.
//
// The English column is quoted literally - it is what the screen has drawn since
// shell-choice S4 - so a change of copy is a change here. The COMMAND itself and
// npm's own failure text are not words of ours: the command stays as it must be
// typed, and npm's output arrives exactly as it was written (D7).
import { describe, expect, it } from 'vitest'
import { installCopy } from './copy'

/** The English words, exactly as the screen drew them before 0.2.0. */
const ENGLISH = {
  title: 'gentle-shell is not installed',
  why: 'This app talks to your models through gentle-shell, and it was not found on this machine. Without it nothing here can work.',
  commandIntro: 'Installing it runs this command:',
  lookedIn:
    'It looked for gentle-shell in the folders of PATH and in the npm global folder (%APPDATA%\\npm). GENTLE_SHELL_BIN can point at it where it lives.',
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
}

/** The Spanish words: neutral Spanish, no voseo, no regionalisms, accents kept. */
const SPANISH = {
  title: 'gentle-shell no está instalado',
  why: 'Esta app habla con tus modelos a través de gentle-shell, y no se encontró en esta máquina. Sin él, nada de aquí puede funcionar.',
  commandIntro: 'Instalarlo ejecuta este comando:',
  lookedIn:
    'Buscó gentle-shell en las carpetas de PATH y en la carpeta global de npm (%APPDATA%\\npm). GENTLE_SHELL_BIN puede apuntar a donde esté.',
  installs:
    'Esto instala solo el programa gentle-shell, unos 250 MB, en la carpeta global de npm. No crea un lugar para chats ni para inicios de sesión.',
  home: 'Dónde viven tus chats e inicios de sesión es una elección aparte, que viene después: tu propio pi, o un espacio propio.',
  warning: 'La instalación cambia este equipo: necesita Node y npm, y puede tardar unos minutos.',
  install: 'Instalar gentle-shell',
  checkAgain: 'Comprobar de nuevo',
  running: 'Instalando gentle-shell. Esto puede tardar unos minutos.',
  done: 'gentle-shell se instaló. Comprueba de nuevo para continuar.',
  failedTitle: 'La instalación no terminó',
  failedHint:
    'Si no se encontró npm, instala Node.js (nodejs.org) y luego pulsa Instalar de nuevo.',
  progress: 'Instalando gentle-shell',
}

/** The plain words of one language. */
const WORDS = Object.keys(ENGLISH) as (keyof typeof ENGLISH)[]

/** Nothing on the screen is spelled the same in both languages. */
const SAME_IN_BOTH = new Set<keyof typeof ENGLISH>([])

/** The words of one language, as a template reads them. */
function words(language: 'en' | 'es'): Record<string, string> {
  const copy = installCopy(language)
  return Object.fromEntries(WORDS.map((word) => [word, copy[word]]))
}

describe('the install copy', () => {
  it('draws the English words the screen already had', () => {
    expect(words('en')).toEqual(ENGLISH)
  })

  it('draws Spanish for the same shape', () => {
    expect(words('es')).toEqual(SPANISH)
  })

  it('translates every word: nothing is left in English', () => {
    const english = installCopy('en')
    const spanish = installCopy('es')

    expect(Object.keys(spanish).sort()).toEqual(Object.keys(english).sort())
    for (const word of WORDS) {
      if (SAME_IN_BOTH.has(word)) {
        expect(spanish[word], word).toBe(english[word])
        continue
      }
      expect(spanish[word], word).not.toBe(english[word])
    }
  })

  it('counts the wait in the language of the screen', () => {
    expect(installCopy('en').elapsed(5)).toBe('5 s elapsed.')
    expect(installCopy('en').elapsed(60)).toBe('1 min elapsed.')
    expect(installCopy('en').elapsed(125)).toBe('2 min 5 s elapsed.')

    expect(installCopy('es').elapsed(5)).toBe('5 s de espera.')
    expect(installCopy('es').elapsed(60)).toBe('1 min de espera.')
    expect(installCopy('es').elapsed(125)).toBe('2 min 5 s de espera.')
  })
})
