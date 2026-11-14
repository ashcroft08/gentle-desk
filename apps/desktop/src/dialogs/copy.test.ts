// The dialog words (0.2.0, S3 and D4): the five button labels the iced shell
// drew live in a language-keyed dictionary, a pending dialog drawn from it
// changes language in the same tick (S2), and the extension's own title, message
// and options keep arriving exactly as the extension wrote them (D7).
//
// The English column is quoted literally - it is what iced drew - so a change of
// copy is a change here.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import { tick } from 'svelte'
import { setLanguage } from '../i18n/language.svelte'
import {
  confirmDialog,
  inputDialog,
  resetTauriMock,
  setHandler,
  viewState,
} from '../test-support/tauri-mock'
import { dialogsCopy } from './copy'
import Dialogs from './Dialogs.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The English dictionary, exactly as iced drew it. */
const ENGLISH = { yes: 'Yes', no: 'No', cancel: 'Cancel', submit: 'Submit' }

/** The Spanish dictionary: neutral Spanish, no voseo and no regionalisms. */
const SPANISH = { yes: 'Sí', no: 'No', cancel: 'Cancelar', submit: 'Enviar' }

/** The one word both languages spell the same: Spanish also says "No". */
const SAME_IN_BOTH = new Set<'yes' | 'no' | 'cancel' | 'submit'>(['no'])

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
})

describe('the dialog copy', () => {
  it('draws the English words iced already drew', () => {
    expect(dialogsCopy('en')).toEqual(ENGLISH)
  })

  it('draws Spanish for the same shape', () => {
    expect(dialogsCopy('es')).toEqual(SPANISH)
  })

  it('translates every word: no entry is left in English', () => {
    const english = dialogsCopy('en')
    const spanish = dialogsCopy('es')

    expect(Object.keys(spanish).sort()).toEqual(Object.keys(english).sort())
    for (const entry of ['yes', 'no', 'cancel', 'submit'] as const) {
      if (SAME_IN_BOTH.has(entry)) {
        expect(spanish[entry], entry).toBe(english[entry])
        continue
      }
      expect(spanish[entry], entry).not.toBe(english[entry])
    }
  })
})

describe('a drawn dialog', () => {
  it('flips the buttons it owns in the same tick, and keeps the extension title', async () => {
    setLanguage('en')
    render(Dialogs, { props: { dialog: confirmDialog('d1', 'Allow this command?') } })
    const modal = await screen.findByRole('dialog')
    expect(screen.getByRole('button', { name: 'Yes' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Cancel' })).toBeTruthy()

    // No push and no restart: the buttons read the same reactive language the
    // Settings rail sets.
    setLanguage('es')
    await tick()

    expect(screen.getByRole('button', { name: 'Sí' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'No' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Cancelar' })).toBeTruthy()
    // D7: the title is the extension's own words, shown as they arrived.
    expect(screen.getByRole('dialog', { name: 'Allow this command?' })).toBe(modal)

    setLanguage('en')
  })

  it('submits an input in the language of the dialog', async () => {
    setLanguage('en')
    render(Dialogs, { props: { dialog: inputDialog('d2') } })
    await screen.findByRole('dialog')
    expect(screen.getByRole('button', { name: 'Submit' })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(screen.getByRole('button', { name: 'Enviar' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Cancelar' })).toBeTruthy()

    setLanguage('en')
  })
})
