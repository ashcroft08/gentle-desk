// The thread's own words (0.2.0, S3 and D4): the scroll region's name, the two
// author lines a screen reader hears, Copy, the live status, the model-error
// title and the two activity lines live in ONE language-keyed dictionary. This
// file pins the two columns; the words a DRAWN thread shows are pinned in
// `copy.flip.test.ts`.
//
// The English column is quoted literally - it is what the thread has drawn since
// the iced view - so a change of copy is a change here. A message's text and the
// provider's error text are not words of this module: they travel through it
// exactly as they arrived (D7).
//
// chat-transcript T2b adds the card's and the reasoning row's words to the same
// two columns, the counting fold button included; composer-dock T3c adds the
// session mode control's, which are pinned in a block of their own because they
// are builders (the label, the wait and the refusal all name a mode).
import { describe, expect, it } from 'vitest'
import { threadCopy } from './copy'

/** The English words, exactly as the thread drew them before 0.2.0. */
const ENGLISH = {
  threadLabel: 'Chat thread',
  youSaid: 'You said',
  assistantSaid: 'Assistant said',
  copy: 'Copy',
  replying: 'Assistant is replying',
  modelErrorTitle: 'Model error',
  opening: 'Opening chat',
  working: 'Working',
  toolDone: 'Done',
  toolRunning: 'Running',
  toolFailed: 'Failed',
  thought: 'Thought',
  expand: 'Expand',
  collapse: 'Collapse',
  showLess: 'Show less',
  cutShort: 'output cut at the bound',
  droppedField: 'a payload field was lost',
}

/** The Spanish words: neutral Spanish, no voseo, no regionalisms, accents kept. */
const SPANISH = {
  threadLabel: 'Hilo del chat',
  youSaid: 'Tú dijiste',
  assistantSaid: 'El asistente dijo',
  copy: 'Copiar',
  replying: 'El asistente está respondiendo',
  modelErrorTitle: 'Error del modelo',
  opening: 'Abriendo el chat',
  working: 'Trabajando',
  toolDone: 'Hecho',
  toolRunning: 'Ejecutando',
  toolFailed: 'Falló',
  thought: 'Pensamiento',
  expand: 'Expandir',
  collapse: 'Contraer',
  showLess: 'Mostrar menos',
  cutShort: 'salida cortada en el límite',
  droppedField: 'se perdió un campo del payload',
}

/** The plain words of one language. */
const WORDS = Object.keys(ENGLISH) as (keyof typeof ENGLISH)[]

/** Nothing in the thread is spelled the same in both languages. */
const SAME_IN_BOTH = new Set<keyof typeof ENGLISH>([])

/** The words of one language, as a template reads them. */
function words(language: 'en' | 'es'): Record<string, string> {
  const copy = threadCopy(language)
  return Object.fromEntries(WORDS.map((word) => [word, copy[word]]))
}

describe('the thread copy', () => {
  it('draws the English words the thread already had', () => {
    expect(words('en')).toEqual(ENGLISH)
  })

  it('draws Spanish for the same shape', () => {
    expect(words('es')).toEqual(SPANISH)
  })

  it('translates every word: nothing is left in English', () => {
    const english = threadCopy('en')
    const spanish = threadCopy('es')

    expect(Object.keys(spanish).sort()).toEqual(Object.keys(english).sort())
    for (const word of WORDS) {
      if (SAME_IN_BOTH.has(word)) {
        expect(spanish[word], word).toBe(english[word])
        continue
      }
      expect(spanish[word], word).not.toBe(english[word])
    }
  })

  it('counts the folded lines in both languages', () => {
    expect(threadCopy('en').earlierLines(14)).toBe('14 earlier lines')
    expect(threadCopy('es').earlierLines(14)).toBe('14 líneas antes')
  })

  it('pins the session mode control in both columns', () => {
    const en = threadCopy('en')
    const es = threadCopy('es')

    expect(en.modeLabel('Super YOLO')).toBe('Session mode: Super YOLO')
    expect(es.modeLabel('Super YOLO')).toBe('Modo de la sesión: Super YOLO')
    expect(en.modeCaption).toBe("This session's mode \u00b7 the default does not change")
    expect(es.modeCaption).toBe('Modo de esta sesión \u00b7 el default no cambia')
    expect(en.modeConfirmTag).toBe('asks to confirm')
    expect(es.modeConfirmTag).toBe('pide confirmación')
    expect(en.modeYoloWhy).toBe('Asks no permission questions.')
    expect(es.modeYoloWhy).toBe('Sin preguntas de permiso.')
    expect(en.modeSuperWhy).toBe('Asks nothing; private paths stay blocked.')
    expect(es.modeSuperWhy).toBe('Sin preguntas; las rutas privadas siguen bloqueadas.')
    expect(en.modeAsk('YOLO')).toBe('Switch this session to YOLO?')
    expect(es.modeAsk('YOLO')).toBe('¿Pasar esta sesión a YOLO?')
    expect(en.modeGo('YOLO')).toBe('Switch to YOLO')
    expect(es.modeGo('YOLO')).toBe('Pasar a YOLO')
    expect(en.modeKept('Normal')).toBe('Still on Normal.')
    expect(es.modeKept('Normal')).toBe('Sigue en Normal.')
    expect(en.modeRefused).toBe('Could not change the mode')
    expect(es.modeRefused).toBe('No se pudo cambiar el modo')
    expect(en.cancel).toBe('Cancel')
    expect(es.cancel).toBe('Cancelar')
    // The effect sentence is shared, and only Super YOLO adds the paths clause.
    expect(en.modeEffect).toBe('The agent stops asking for permission')
    expect(es.modeEffect).toBe('El agente deja de pedir permiso')
    // YOLO says the same first sentence and drops the private-paths clause.
    expect(en.modeEffectYolo).toBe(' to run commands and edit files. It applies to this session only.')
    expect(en.modeEffectSuper).toBe(
      ' to run commands and edit files. It applies to this session only; private paths stay blocked.',
    )
    expect(es.modeEffectYolo).toBe(' para ejecutar comandos y editar archivos. Vale sólo para esta sesión.')
    expect(es.modeEffectSuper).toBe(
      ' para ejecutar comandos y editar archivos. Vale sólo para esta sesión; las rutas privadas siguen bloqueadas.',
    )
  })
})
