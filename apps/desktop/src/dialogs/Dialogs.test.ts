// The dialog host's own rules, which the four kinds share: Escape and Cancel
// cancel, one answer per dialog however many times it is clicked, the typed text
// stays here and never travels, and the queue moves one dialog at a time.

import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte'
import {
  confirmDialog,
  handlerArgs,
  inputDialog,
  resetTauriMock,
  selectDialog,
  setHandler,
  viewState,
} from '../test-support/tauri-mock'
import type { DialogView_Serialize } from '../contract/bindings'
import Dialogs from './Dialogs.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

const CANCELLED = { type: 'AnswerDialog', data: { id: 'd1', answer: { type: 'Cancelled' } } }

/** The intents the dialog sent, oldest first. */
function intents(): unknown[] {
  return handlerArgs('dispatch_intent').map((args) => args.intent)
}

/** Opens the host on one dialog, and answers the promise when it is drawn. */
async function open(dialog: DialogView_Serialize | null): Promise<void> {
  render(Dialogs, { props: { dialog } })
  if (dialog !== null) await screen.findByRole('dialog')
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
})

describe('cancelling', () => {
  it('sends a cancelled answer on Escape', async () => {
    await open(confirmDialog('d1'))

    await fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Escape' })

    expect(intents()).toEqual([CANCELLED])
  })

  it('sends a cancelled answer on the Cancel button', async () => {
    await open(confirmDialog('d1'))

    await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }))

    expect(intents()).toEqual([CANCELLED])
  })

  it('sends one cancelled answer when Escape and Cancel both arrive', async () => {
    await open(confirmDialog('d1'))

    await fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Escape' })
    await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }))

    expect(intents()).toEqual([CANCELLED])
  })
})

describe('answering once', () => {
  it('sends one answer for a double click on Submit', async () => {
    await open(inputDialog('d1'))
    const field = screen.getByRole('textbox')
    await fireEvent.input(field, { target: { value: 'Ada' } })
    const submit = screen.getByRole('button', { name: 'Submit' })

    await fireEvent.click(submit)
    await fireEvent.click(submit)

    expect(intents()).toEqual([
      { type: 'AnswerDialog', data: { id: 'd1', answer: { type: 'Value', data: { value: 'Ada' } } } },
    ])
  })

  it('answers again once Rust sends the next dialog', async () => {
    const { rerender } = render(Dialogs, { props: { dialog: selectDialog('d2', ['a']) } })
    await screen.findByRole('dialog')
    await fireEvent.click(screen.getByRole('button', { name: 'a' }))

    await rerender({ dialog: confirmDialog('d3') })

    await fireEvent.click(await screen.findByRole('button', { name: 'No' }))
    expect(intents()).toHaveLength(2)
  })
})

describe('the typed text', () => {
  it('starts empty again when a new dialog arrives', async () => {
    const { rerender } = render(Dialogs, { props: { dialog: inputDialog('d1') } })
    const first = screen.getByRole('textbox')
    await fireEvent.input(first, { target: { value: 'half typed' } })
    expect((first as HTMLInputElement).value).toBe('half typed')

    await rerender({ dialog: inputDialog('d2') })

    const second = (await screen.findByRole('textbox')) as HTMLInputElement
    expect(second.value).toBe('')
  })

  it('never travels before the user submits it', async () => {
    const { rerender } = render(Dialogs, { props: { dialog: inputDialog('d1') } })
    const field = screen.getByRole('textbox')

    await fireEvent.input(field, { target: { value: 'half typed' } })
    await rerender({ dialog: inputDialog('d1') })

    expect(intents()).toEqual([])
    expect(viewState()).not.toHaveProperty('typed')
  })
})

describe('nothing pending', () => {
  it('draws no dialog at all', async () => {
    await open(null)

    expect(screen.queryByRole('dialog')).toBeNull()
    expect(screen.queryByRole('button')).toBeNull()
  })
})

describe('the focus', () => {
  it('stays inside the modal, never in the composer behind it', async () => {
    render(Dialogs, { props: { dialog: confirmDialog('d1') } })
    await screen.findByRole('dialog')

    await waitFor(() => expect(document.activeElement).not.toBe(document.body))
    const composer = document.createElement('textarea')
    document.body.appendChild(composer)
    expect(document.activeElement).not.toBe(composer)
    composer.remove()
  })
})
