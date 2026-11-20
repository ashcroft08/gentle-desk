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
  keysHintWorking:
    'Enter is off while Gentle works \u00b7 Shift+Enter for a new line \u00b7 Esc to stop',
  send: 'Send',
  stop: 'Stop',
  openingReason: 'Send is available once the chat is ready.',
  dropHint: 'Drop to attach',
  removeImage: 'Remove',
  removeFile: 'Remove',
  attachFiles: 'Attach files',
  attachedFiles: 'Attached files',
  outsideWorkspace: 'outside the workspace',
  outsideWarning:
    "Some files are outside this chat's folder; the agent may not be able to open them.",
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
  keysHintWorking:
    'Enter no envía mientras Gentle trabaja \u00b7 Shift+Enter para una línea nueva \u00b7 Esc para detener',
  send: 'Enviar',
  stop: 'Detener',
  openingReason: 'Enviar estará disponible cuando el chat esté listo.',
  dropHint: 'Suelta para adjuntar',
  removeImage: 'Quitar',
  removeFile: 'Quitar',
  attachFiles: 'Adjuntar archivos',
  attachedFiles: 'Archivos adjuntos',
  outsideWorkspace: 'fuera del workspace',
  outsideWarning:
    'Hay archivos fuera de la carpeta de este chat; el agente puede no poder abrirlos.',
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

  it('says why a file was not attached, in the language of the moment', () => {
    // The folder's line is the approved artboard's own words (sheet D).
    expect(composerCopy('es').attachFailed('assets', 'IsDirectory')).toBe(
      'No se pudo adjuntar assets: es una carpeta, no un archivo.',
    )
    expect(composerCopy('en').attachFailed('assets', 'IsDirectory')).toBe(
      'Could not attach assets: it is a folder, not a file.',
    )
    expect(composerCopy('en').attachFailed('gone.md', 'NotFound')).toBe(
      'Could not attach gone.md: the file does not exist.',
    )
    expect(composerCopy('es').attachFailed('gone.md', 'NotFound')).toBe(
      'No se pudo adjuntar gone.md: el archivo no existe.',
    )
    expect(composerCopy('en').attachFailed('locked.md', 'Unreadable')).toBe(
      'Could not attach locked.md: the file could not be read.',
    )
    expect(composerCopy('es').attachFailed('locked.md', 'Unreadable')).toBe(
      'No se pudo adjuntar locked.md: no se pudo leer el archivo.',
    )
  })

  it('counts the chips a collapsed row hides, in the language of the moment', () => {
    expect(composerCopy('en').showMore(9)).toBe('+9 more')
    expect(composerCopy('es').showMore(9)).toBe('+9 más')
    expect(composerCopy('en').showMoreLabel(9)).toBe('Show the 9 remaining files')
    expect(composerCopy('es').showMoreLabel(9)).toBe('Ver los 9 archivos restantes')
  })
})
