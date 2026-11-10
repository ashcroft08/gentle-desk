// Each of the four extension dialogs as the user meets it: the fields it draws,
// the control that takes the focus, and the one answer its primary control
// sends. The answers leave through the generated bindings, so what is asserted
// is the intent Rust would receive.

import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte'
import {
  confirmDialog,
  editorDialog,
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

/** The intents the dialog sent, oldest first. */
function intents(): unknown[] {
  return handlerArgs('dispatch_intent').map((args) => args.intent)
}

/** Mounts the host with one pending dialog and waits for the modal. */
async function open(dialog: DialogView_Serialize): Promise<HTMLElement> {
  render(Dialogs, { props: { dialog } })
  return screen.findByRole('dialog')
}

/** Waits for `element` to hold the focus: the dialog focuses after mounting. */
async function expectFocus(element: HTMLElement): Promise<void> {
  await waitFor(() => expect(document.activeElement).toBe(element))
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
})

describe('a confirm dialog', () => {
  it('draws its title and message, and starts on the confirming button', async () => {
    const modal = await open(confirmDialog('d1', 'Allow this command?', 'Run `ls`?'))
    await expectFocus(screen.getByRole('button', { name: 'Yes' }))

    expect(modal.getAttribute('aria-modal')).toBe('true')
    // The title is the modal's accessible name, not only its first line.
    expect(screen.getByRole('dialog', { name: 'Allow this command?' })).toBe(modal)
    expect(screen.getByText('Run `ls`?')).toBeTruthy()
    expect(screen.getByText('Run `ls`?')).toBeTruthy()
    expect(screen.getByRole('button', { name: 'No' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Cancel' })).toBeTruthy()
  })

  it('answers yes with a confirmation', async () => {
    await open(confirmDialog('d1'))

    await fireEvent.click(screen.getByRole('button', { name: 'Yes' }))

    expect(intents()).toEqual([
      { type: 'AnswerDialog', data: { id: 'd1', answer: { type: 'Confirmed', data: { confirmed: true } } } },
    ])
  })

  it('answers no with a refusal, never a value', async () => {
    await open(confirmDialog('d1'))

    await fireEvent.click(screen.getByRole('button', { name: 'No' }))

    expect(intents()).toEqual([
      { type: 'AnswerDialog', data: { id: 'd1', answer: { type: 'Confirmed', data: { confirmed: false } } } },
    ])
  })
})

describe('a select dialog', () => {
  it('draws one button per option, in order, and starts on the first', async () => {
    await open(selectDialog('d2', ['a', 'b']))
    const options = screen.getAllByRole('button', { name: /^[ab]$/ })
    await expectFocus(options[0])

    expect(screen.getByRole('dialog', { name: 'Pick a letter' })).toBeTruthy()
    expect(options.map((option) => option.textContent)).toEqual(['a', 'b'])
  })

  it('answers the highlighted option, not the one the mouse never touched', async () => {
    await open(selectDialog('d2', ['a', 'b']))
    const options = screen.getAllByRole('button', { name: /^[ab]$/ })

    await fireEvent.keyDown(options[0], { key: 'ArrowDown' })
    await expectFocus(options[1])
    await fireEvent.keyDown(options[1], { key: 'Enter' })

    expect(intents()).toEqual([
      { type: 'AnswerDialog', data: { id: 'd2', answer: { type: 'Value', data: { value: 'b' } } } },
    ])
  })
})

describe('an input dialog', () => {
  it('draws its title and hint, and starts in the field', async () => {
    await open(inputDialog('d3', 'name', 'Your name'))
    const field = screen.getByRole('textbox', { name: 'Your name' })
    await expectFocus(field)

    expect(screen.getByRole('dialog', { name: 'Your name' })).toBeTruthy()
    expect(field.getAttribute('placeholder')).toBe('name')
  })

  it('answers the typed text on Enter', async () => {
    await open(inputDialog('d3', 'name', 'Your name'))
    const field = screen.getByRole('textbox', { name: 'Your name' })

    await fireEvent.input(field, { target: { value: 'Ada' } })
    await fireEvent.keyDown(field, { key: 'Enter' })

    expect(intents()).toEqual([
      { type: 'AnswerDialog', data: { id: 'd3', answer: { type: 'Value', data: { value: 'Ada' } } } },
    ])
  })
})

describe('an editor dialog', () => {
  it('draws its title and prefill, and starts in the box', async () => {
    await open(editorDialog('d4', 'a note', 'Edit the note'))
    const box = screen.getByRole('textbox', { name: 'Edit the note' })
    await expectFocus(box)

    expect(screen.getByRole('dialog', { name: 'Edit the note' })).toBeTruthy()
    expect((box as HTMLTextAreaElement).value).toBe('a note')
  })

  it('keeps Enter for a newline and submits on Ctrl+Enter', async () => {
    await open(editorDialog('d4', 'a note', 'Edit the note'))
    const box = screen.getByRole('textbox', { name: 'Edit the note' })

    await fireEvent.input(box, { target: { value: 'a note\nsecond line' } })
    await fireEvent.keyDown(box, { key: 'Enter' })
    expect(intents()).toEqual([])

    await fireEvent.keyDown(box, { key: 'Enter', ctrlKey: true })
    expect(intents()).toEqual([
      {
        type: 'AnswerDialog',
        data: { id: 'd4', answer: { type: 'Value', data: { value: 'a note\nsecond line' } } },
      },
    ])
  })
})
