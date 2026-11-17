// The helpers view's pure rules and its drawn words: every helper status draws a
// chat mark (the approved mapping), the counts line is the one iced's card
// spells, the thread's words are iced's own - and since 0.2.0 (S3, D4) every word
// the view owns is written in BOTH languages, with a panel drawn from the
// dictionary changing language in the same tick (S2).
//
// The English column is quoted literally: it is what the panel drew before
// 0.2.0, so a change of copy is a change here.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen, within } from '@testing-library/svelte'
import { tick } from 'svelte'
import type { HelperRowView_Serialize as HelperRow, HelperStatusView } from '../contract/bindings'
import type { MarkKind } from '../icons/shapes'
import { setLanguage } from '../i18n/language.svelte'
import { helperRow, helpersPanel } from '../test-support/helpers'
import { resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import HelpersPanel from './HelpersPanel.svelte'
import { DOT, helpersCopy, markOfStatus, threadHeading } from './copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

const MARKS: [HelperStatusView, MarkKind][] = [
  ['Running', 'Working'],
  ['Queued', 'Queued'],
  ['Waiting', 'Queued'],
  ['Done', 'NewReply'],
  ['Failed', 'Ended'],
  ['Cancelled', 'Ended'],
]

/** The English dictionary, exactly as the panel drew it before 0.2.0. */
const ENGLISH = {
  listLabel: 'Helpers',
  threadLabel: 'Helper thread',
  noHelpers: 'No helpers for this chat',
  back: 'Back to helpers',
  noActivity: 'No activity yet',
  stop: 'Stop',
  stopped: 'Stopped',
}

/** The Spanish dictionary: neutral Spanish, no voseo and no regionalisms. */
const SPANISH = {
  listLabel: 'Ayudantes',
  threadLabel: 'Hilo del ayudante',
  noHelpers: 'Este chat no tiene ayudantes',
  back: 'Volver a los ayudantes',
  noActivity: 'Todavía no hay actividad',
  stop: 'Detener',
  stopped: 'Detenido',
}

/** The words of one language, as the panel and the card draw them. */
const WORDS = [
  'listLabel',
  'threadLabel',
  'noHelpers',
  'back',
  'noActivity',
  'stop',
  'stopped',
] as const

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
})

describe('markOfStatus', () => {
  it.each(MARKS)('draws %s with the %s chat mark', (status, mark) => {
    expect(markOfStatus(status)).toBe(mark)
  })
})

describe('cardMeta', () => {
  const english = helpersCopy('en')
  const spanish = helpersCopy('es')

  it('spells the turns and the tool calls the way iced does', () => {
    expect(english.cardMeta(2, 1)).toBe(`2 turns ${DOT} 1 tool calls`)
  })

  it('separates with the middle dot iced uses, never an icon', () => {
    expect(DOT).toBe('\u00b7')
  })

  it('spells the counts in Spanish, marking the singular the English column leaves bare', () => {
    expect(spanish.cardMeta(2, 1)).toBe(`2 turnos ${DOT} 1 llamada a herramienta`)
    expect(spanish.cardMeta(4, 3)).toBe(`4 turnos ${DOT} 3 llamadas a herramienta`)
  })
})

describe('the thread words', () => {
  const english = helpersCopy('en')
  const spanish = helpersCopy('es')

  it('spells the dropped line the way iced does, singular or not', () => {
    expect(english.droppedLine(3)).toBe('3 earlier items were dropped')
    expect(english.droppedLine(1)).toBe('1 earlier items were dropped')
  })

  it('spells the dropped line in Spanish, either number', () => {
    expect(spanish.droppedLine(3)).toBe('Se descartaron 3 elementos anteriores')
    expect(spanish.droppedLine(1)).toBe('Se descartó 1 elemento anterior')
  })

  it('names the helper by its agent and task, or by the task alone', () => {
    expect(threadHeading('explore', 'Inventory of sidebar modules')).toBe(
      'explore - Inventory of sidebar modules',
    )
    expect(threadHeading('', 'Inventory of sidebar modules')).toBe('Inventory of sidebar modules')
    expect(threadHeading('  ', 'Only a task')).toBe('Only a task')
  })

  it('draws each tool status with iced own word', () => {
    expect(english.toolStatus('Running')).toBe('running')
    expect(english.toolStatus('Failed')).toBe('failed')
    expect(english.toolStatus('Ok')).toBe('ok')
  })

  it('draws each tool status in Spanish, never the English word', () => {
    expect(spanish.toolStatus('Running')).toBe('en ejecución')
    expect(spanish.toolStatus('Failed')).toBe('falló')
    expect(spanish.toolStatus('Ok')).toBe('correcto')
  })

  it('names the stop control for the helper it would stop', () => {
    expect(english.stopNamed('explore - Inventory of sidebar modules')).toBe(
      'Stop explore - Inventory of sidebar modules',
    )
    expect(english.alreadyStopped('explore - Inventory of sidebar modules')).toBe(
      'explore - Inventory of sidebar modules already stopped',
    )
  })

  it('names the stop control in Spanish, for the helper it would stop', () => {
    expect(spanish.stopNamed('explore - Inventory of sidebar modules')).toBe(
      'Detener explore - Inventory of sidebar modules',
    )
    expect(spanish.alreadyStopped('explore - Inventory of sidebar modules')).toBe(
      'explore - Inventory of sidebar modules ya está detenido',
    )
  })
})

describe('the helpers copy', () => {
  it('draws the English words the panel already had', () => {
    const english = helpersCopy('en')
    expect(Object.fromEntries(WORDS.map((word) => [word, english[word]]))).toEqual(ENGLISH)
  })

  it('draws Spanish for the same shape', () => {
    const spanish = helpersCopy('es')
    expect(Object.fromEntries(WORDS.map((word) => [word, spanish[word]]))).toEqual(SPANISH)
  })

  it('translates every word and every line: nothing is left in English', () => {
    const english = helpersCopy('en')
    const spanish = helpersCopy('es')

    for (const word of WORDS) {
      expect(spanish[word], word).not.toBe(english[word])
    }
    // The parametrized entries answer a different sentence in each language.
    expect(spanish.cardMeta(2, 1)).not.toBe(english.cardMeta(2, 1))
    expect(spanish.droppedLine(3)).not.toBe(english.droppedLine(3))
    for (const status of ['Running', 'Failed', 'Ok'] as const) {
      expect(spanish.toolStatus(status), status).not.toBe(english.toolStatus(status))
    }
    expect(spanish.stopNamed('h')).not.toBe(english.stopNamed('h'))
    expect(spanish.alreadyStopped('h')).not.toBe(english.alreadyStopped('h'))
  })
})

describe('a drawn panel', () => {
  /** The panel's own words in `language`, for a chat with no helpers at all. */
  function draw(language: 'en' | 'es'): void {
    setLanguage(language)
    const helpers: HelperRow[] = []
    render(HelpersPanel, { props: { helpers: helpersPanel(helpers) } })
  }

  it('flips its own words in the same tick, with no push and no restart', async () => {
    draw('en')
    expect(screen.getByText('No helpers for this chat')).toBeTruthy()

    // No push and no restart: the panel reads the same reactive language the
    // Settings rail sets.
    setLanguage('es')
    await tick()

    expect(screen.getByText('Este chat no tiene ayudantes')).toBeTruthy()
    setLanguage('en')
  })

  it('names the list in the language of the panel', async () => {
    setLanguage('en')
    render(HelpersPanel, { props: { helpers: helpersPanel([helperRow('h1')]) } })
    expect(screen.getByRole('listbox', { name: 'Helpers' })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(screen.getByRole('listbox', { name: 'Ayudantes' })).toBeTruthy()
    expect(within(screen.getByRole('listbox')).getAllByRole('option')).toHaveLength(1)
    setLanguage('en')
  })
})
