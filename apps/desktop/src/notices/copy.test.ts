// The notice's own words (0.2.0, S3 and D4): the action's name, the dismiss and
// the word each level is spoken with live in a language-keyed dictionary, and a
// card drawn from it changes language in the same tick (S2) while the child's own
// text keeps arriving exactly as it was written (D7).
//
// The English column is quoted literally - it is what the card has drawn since
// the states board - so a change of copy is a change here.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen, within } from '@testing-library/svelte'
import { tick } from 'svelte'
import type { NoticeLevel } from '../contract/bindings'
import { setLanguage } from '../i18n/language.svelte'
import {
  plainNotice,
  resetTauriMock,
  restartNotice,
  setHandler,
  viewState,
} from '../test-support/tauri-mock'
import { noticesCopy } from './copy'
import Notice from './Notice.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The English dictionary, exactly as the card drew it before 0.2.0. */
const ENGLISH = {
  restart: 'Restart chat',
  dismiss: 'Dismiss',
  levels: { Error: 'Error', Warning: 'Warning', Info: 'Info' },
}

/** The Spanish dictionary: neutral Spanish, no voseo and no regionalisms. */
const SPANISH = {
  restart: 'Reiniciar chat',
  dismiss: 'Descartar',
  levels: { Error: 'Error', Warning: 'Advertencia', Info: 'Información' },
}

/** The three levels, as the contract names them. */
const LEVELS: NoticeLevel[] = ['Error', 'Warning', 'Info']

/** The one word both languages spell the same: Spanish also says "Error". */
const SAME_IN_BOTH = new Set<NoticeLevel>(['Error'])

/** The child's own text, which no language of ours ever rewrites (D7). */
const TEXT = 'Could not read 3 chat files.'

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
})

describe('the notice copy', () => {
  it('draws the English words the card already had', () => {
    expect(noticesCopy('en')).toEqual(ENGLISH)
  })

  it('draws Spanish for the same shape', () => {
    expect(noticesCopy('es')).toEqual(SPANISH)
  })

  it('translates every word: no entry is left in English', () => {
    const english = noticesCopy('en')
    const spanish = noticesCopy('es')

    expect(Object.keys(spanish.levels).sort()).toEqual(Object.keys(english.levels).sort())
    for (const entry of ['restart', 'dismiss'] as const) {
      expect(spanish[entry], entry).not.toBe(english[entry])
    }
    for (const level of LEVELS) {
      if (SAME_IN_BOTH.has(level)) {
        // The same word in both languages, so it is shared on purpose.
        expect(spanish.levels[level], level).toBe(english.levels[level])
        continue
      }
      expect(spanish.levels[level], `the word for ${level}`).not.toBe(english.levels[level])
    }
  })
})

describe('a drawn card', () => {
  it('flips its own words in the same tick, and leaves the child text alone', async () => {
    setLanguage('en')
    render(Notice, { props: { notice: plainNotice(1, TEXT, 'Warning') } })
    const card = screen.getByRole('status')
    expect(within(card).getByText('Warning')).toBeTruthy()
    expect(within(card).getByRole('button', { name: 'Dismiss' })).toBeTruthy()

    // No push and no restart: the card reads the same reactive language the
    // Settings rail sets, so one local write redraws it.
    setLanguage('es')
    await tick()

    expect(within(card).getByText('Advertencia')).toBeTruthy()
    expect(within(card).getByRole('button', { name: 'Descartar' })).toBeTruthy()
    // D7: the notice's text came from the child, so it is shown as it arrived.
    expect(card.textContent).toContain(TEXT)

    setLanguage('en')
  })

  it('names the one action in the language of the card', async () => {
    setLanguage('en')
    render(Notice, { props: { notice: restartNotice(2, TEXT, 'c:chat-1') } })
    const card = screen.getByRole('alert')
    expect(within(card).getByRole('button', { name: 'Restart chat' })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(within(card).getByRole('button', { name: 'Reiniciar chat' })).toBeTruthy()

    setLanguage('en')
  })
})
