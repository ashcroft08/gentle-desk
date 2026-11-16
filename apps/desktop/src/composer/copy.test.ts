// The composer's own words (0.2.0, S3 and D4): the box's name and placeholder,
// the keys hint, the two buttons, the sidebar toggle, the two disabled reasons,
// the drop hint and the three model-chip states live in ONE language-keyed
// dictionary. This file pins the two columns; the words a DRAWN composer shows
// are pinned in `copy.flip.test.ts`.
//
// The English column is quoted literally - it is what the artboard drew - so a
// change of copy is a change here. The model NAMES, the thinking level NAMES the
// child sends and an image's file name are not words of this module: they travel
// through it exactly as they arrived (D7).
import { describe, expect, it } from 'vitest'
import { composerCopy } from './copy'

/** The English words, exactly as the composer drew them before 0.2.0. */
const ENGLISH = {
  messageLabel: 'Message',
  messagePlaceholder: 'Tell Gentle what you need...',
  keysHint: 'Enter to send \u00b7 Shift+Enter for a new line \u00b7 Esc to stop',
  send: 'Send',
  stop: 'Stop',
  openingReason: 'Send is available once the chat is ready.',
  dropHint: 'Drop the images here to attach them.',
  removeImage: 'Remove',
  searchPlaceholder: 'Search models...',
  nothingMatches: 'Nothing matches.',
  sidebarToggle: 'Toggle sidebar',
  endedReason: "This chat's process exited. Restart it to keep talking.",
  modelLabel: 'Model',
  levelLabel: 'Thinking level',
  modelsLoading: 'Loading models...',
  modelsEmpty: 'No models are available for this chat.',
  modelsUnavailable: 'Models unavailable:',
  contextHidden: 'Context usage is not known right now',
}

/** The Spanish words: neutral Spanish, no voseo, no regionalisms, accents kept. */
const SPANISH = {
  messageLabel: 'Mensaje',
  messagePlaceholder: 'Cuéntale a Gentle lo que necesitas...',
  keysHint: 'Enter para enviar \u00b7 Shift+Enter para una línea nueva \u00b7 Esc para detener',
  send: 'Enviar',
  stop: 'Detener',
  openingReason: 'Enviar estará disponible cuando el chat esté listo.',
  dropHint: 'Suelta las imágenes aquí para adjuntarlas.',
  removeImage: 'Quitar',
  searchPlaceholder: 'Buscar modelos...',
  nothingMatches: 'No hay coincidencias.',
  sidebarToggle: 'Mostrar u ocultar la barra lateral',
  endedReason: 'El proceso de este chat terminó. Reinícialo para seguir hablando.',
  modelLabel: 'Modelo',
  levelLabel: 'Nivel de pensamiento',
  modelsLoading: 'Cargando modelos...',
  modelsEmpty: 'No hay modelos disponibles para este chat.',
  modelsUnavailable: 'Modelos no disponibles:',
  contextHidden: 'El uso del contexto no se conoce ahora mismo',
}

/** The plain words of one language. */
const WORDS = Object.keys(ENGLISH) as (keyof typeof ENGLISH)[]

/** Nothing in the composer is spelled the same in both languages. */
const SAME_IN_BOTH = new Set<keyof typeof ENGLISH>([])

/** The words of one language, as a template reads them. */
function words(language: 'en' | 'es'): Record<string, string> {
  const copy = composerCopy(language)
  return Object.fromEntries(WORDS.map((word) => [word, copy[word]]))
}

describe('the composer copy', () => {
  it('draws the English words the composer already had', () => {
    expect(words('en')).toEqual(ENGLISH)
  })

  it('draws Spanish for the same shape', () => {
    expect(words('es')).toEqual(SPANISH)
  })

  it('translates every word: nothing is left in English', () => {
    const english = composerCopy('en')
    const spanish = composerCopy('es')

    expect(Object.keys(spanish).sort()).toEqual(Object.keys(english).sort())
    for (const word of WORDS) {
      if (SAME_IN_BOTH.has(word)) {
        expect(spanish[word], word).toBe(english[word])
        continue
      }
      expect(spanish[word], word).not.toBe(english[word])
    }
  })

  it('names the context reading in the language of the moment', () => {
    expect(composerCopy('en').contextLabel(35)).toBe(
      "Context used: 35 percent of the model's window",
    )
    expect(composerCopy('es').contextLabel(35)).toBe(
      'Contexto usado: 35 por ciento de la ventana del modelo',
    )
  })
})
