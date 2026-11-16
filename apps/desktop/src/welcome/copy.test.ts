// The welcome question's own words (0.2.0, S3 and D4): the screen's title and
// intro, the found line, both answers with their points, the two paths they name,
// the applying line and the skip note live in ONE language-keyed dictionary. This
// file pins the two columns; the words a DRAWN screen shows are pinned in
// `copy.flip.test.ts`.
//
// The English column is quoted literally - it is what the screen has drawn since
// shell-choice S2 - so a change of copy is a change here. The FOLDER the machine
// reported is not a word of this module: it is drawn exactly as it was found (D7).
import { describe, expect, it } from 'vitest'
import { welcomeCopy } from './copy'

/** The English words, exactly as the screen drew them before 0.2.0. */
const ENGLISH = {
  title: 'Welcome to gentle shell',
  intro: 'Everything you need is already inside this app. One question before you start.',
  foundTitle: 'We found pi on this machine',
  linkTitle: 'Use my pi setup',
  linkBadge: 'RECOMMENDED',
  linkPath: 'links to ~/.pi/agent',
  separateTitle: 'Keep it separate',
  separatePath: 'creates ~/.gentle-shell/agent',
  applying: 'Applying your choice. The app will start again in a moment.',
  skipNote:
    'No pi on the machine? This screen is skipped and gentle shell starts in its own space.',
}

/** The Spanish words: neutral Spanish, no voseo, no regionalisms, accents kept. */
const SPANISH = {
  title: 'Te damos la bienvenida a gentle shell',
  intro: 'Todo lo que necesitas ya está dentro de esta app. Una pregunta antes de empezar.',
  foundTitle: 'Encontramos pi en esta máquina',
  linkTitle: 'Usar mi configuración de pi',
  linkBadge: 'RECOMENDADO',
  linkPath: 'enlaza con ~/.pi/agent',
  separateTitle: 'Mantenerlo separado',
  separatePath: 'crea ~/.gentle-shell/agent',
  applying: 'Aplicando tu elección. La app se reiniciará en un momento.',
  skipNote:
    '¿No hay pi en la máquina? Esta pantalla se omite y gentle shell arranca en su propio espacio.',
}

/** The English points under each answer, in the order the screen draws them. */
const LINK_POINTS_EN = [
  'Your sign-ins, local models and chats appear here',
  'Chats you start here also show up in your terminal',
  'Your pi settings are never edited',
]
const LINK_POINTS_ES = [
  'Tus inicios de sesión, modelos locales y chats aparecen aquí',
  'Los chats que inicies aquí también aparecen en tu terminal',
  'Tu configuración de pi nunca se modifica',
]
const SEPARATE_POINTS_EN = [
  'A clean space with its own sign-ins and chats',
  'Nothing on this machine is touched',
  'You can change this later from Settings',
]
const SEPARATE_POINTS_ES = [
  'Un espacio limpio con sus propios inicios de sesión y chats',
  'No se toca nada de esta máquina',
  'Puedes cambiarlo más tarde en Ajustes',
]

/** The plain words of one language. */
const WORDS = Object.keys(ENGLISH) as (keyof typeof ENGLISH)[]

/** Nothing on the screen is spelled the same in both languages. */
const SAME_IN_BOTH = new Set<keyof typeof ENGLISH>([])

/** The words of one language, as a template reads them. */
function words(language: 'en' | 'es'): Record<string, string> {
  const copy = welcomeCopy(language)
  return Object.fromEntries(WORDS.map((word) => [word, copy[word]]))
}

describe('the welcome copy', () => {
  it('draws the English words the screen already had', () => {
    expect(words('en')).toEqual(ENGLISH)
  })

  it('draws Spanish for the same shape', () => {
    expect(words('es')).toEqual(SPANISH)
  })

  it('translates every word: nothing is left in English', () => {
    const english = welcomeCopy('en')
    const spanish = welcomeCopy('es')

    expect(Object.keys(spanish).sort()).toEqual(Object.keys(english).sort())
    for (const word of WORDS) {
      if (SAME_IN_BOTH.has(word)) {
        expect(spanish[word], word).toBe(english[word])
        continue
      }
      expect(spanish[word], word).not.toBe(english[word])
    }
  })

  it('translates all three points of each answer, in the order they are drawn', () => {
    expect(welcomeCopy('en').linkPoints).toEqual(LINK_POINTS_EN)
    expect(welcomeCopy('es').linkPoints).toEqual(LINK_POINTS_ES)
    expect(welcomeCopy('en').separatePoints).toEqual(SEPARATE_POINTS_EN)
    expect(welcomeCopy('es').separatePoints).toEqual(SEPARATE_POINTS_ES)
  })
})
