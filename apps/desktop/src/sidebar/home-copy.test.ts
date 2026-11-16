// The footer home setting's own words (shell-choice S5, 0.2.0 S3/D4): the group's
// name, its two answers and the line that offers the restart live in one
// language-keyed dictionary, so a control drawn from it changes language in the
// same tick (S2).
//
// The English column is quoted literally: it is what the setting has drawn since
// shell-choice S5, so a change of copy is a change here.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import { tick } from 'svelte'
import type { ShellView_Serialize as ShellView } from '../contract/bindings'
import { setLanguage } from '../i18n/language.svelte'
import { resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import HomeSetting from './HomeSetting.svelte'
import { homeCopy } from './home-copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The English words, exactly as the setting drew them before 0.2.0. */
const ENGLISH = {
  group: 'Home',
  link: 'My pi',
  separate: 'Separate',
  restartNote: 'Restart to use this home.',
  restartNow: 'Restart now',
}

/** The Spanish words: neutral Spanish, accents allowed. */
const SPANISH = {
  group: 'Home',
  link: 'Mi pi',
  separate: 'Separado',
  restartNote: 'Reinicia para usar este home.',
  restartNow: 'Reiniciar ahora',
}

/** The words of one language, as the control draws them. */
const WORDS = Object.keys(ENGLISH) as (keyof typeof ENGLISH)[]

/**
 * "Home" is the shell's own idea and the word the Settings section already uses
 * in Spanish ("Home del shell"), so it is the one word shared on purpose.
 */
const SAME_IN_BOTH = new Set<keyof typeof ENGLISH>(['group'])

/** A shell on the user's own pi, with a home change waiting for a restart. */
function shell(over: Partial<ShellView> = {}): ShellView {
  return {
    pi_path: 'C:/Users/someone/.pi/agent',
    choice: 'link',
    ask: false,
    launcher_found: true,
    install: { type: 'Idle' },
    restart_needed: true,
    ...over,
  }
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
})

describe('the home setting copy', () => {
  it('draws the English words the setting already had', () => {
    const english = homeCopy('en')
    expect(Object.fromEntries(WORDS.map((word) => [word, english[word]]))).toEqual(ENGLISH)
  })

  it('draws Spanish for the same shape', () => {
    const spanish = homeCopy('es')
    expect(Object.fromEntries(WORDS.map((word) => [word, spanish[word]]))).toEqual(SPANISH)
  })

  it('translates every word it is ours to translate', () => {
    const english = homeCopy('en')
    const spanish = homeCopy('es')

    for (const word of WORDS) {
      if (SAME_IN_BOTH.has(word)) {
        expect(spanish[word], word).toBe(english[word])
        continue
      }
      expect(spanish[word], word).not.toBe(english[word])
    }
  })
})

describe('a drawn home setting', () => {
  it('flips its own words in the same tick, with no push and no restart', async () => {
    setLanguage('en')
    render(HomeSetting, { props: { shell: shell() } })

    expect(screen.getByRole('group', { name: 'Home' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'My pi' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Separate' })).toBeTruthy()
    expect(screen.getByText('Restart to use this home.')).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Restart now' })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(screen.getByRole('group', { name: 'Home' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Mi pi' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Separado' })).toBeTruthy()
    expect(screen.getByText('Reinicia para usar este home.')).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Reiniciar ahora' })).toBeTruthy()

    setLanguage('en')
  })
})
