// The screen for a machine with no gentle-shell (shell-choice S4). It explains
// that the app cannot work without it and asks for permission: nothing is
// installed until the user presses the install button, and the screen follows
// the install it started (running, done, failed) and offers to look again.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import type { InstallView } from '../contract/bindings'
import { handlerArgs, resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import MissingShell from './MissingShell.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
})

function intents(): unknown[] {
  return handlerArgs('dispatch_intent').map((args) => (args as { intent: unknown }).intent)
}

function draw(install: InstallView = { type: 'Idle' }) {
  return render(MissingShell, { props: { install } })
}

describe('the missing gentle-shell screen', () => {
  it('says gentle-shell is required and shows the exact command it would run', () => {
    draw()

    expect(screen.getByRole('heading', { name: 'gentle-shell is not installed' })).toBeTruthy()
    expect(screen.getByText('npm install -g gentle-pi')).toBeTruthy()
  })

  it('installs nothing until the user presses the button', () => {
    draw()

    expect(intents()).toEqual([])
  })

  it('asks for the install once, however many times the button is pressed', async () => {
    draw()
    const button = screen.getByRole('button', { name: 'Install gentle-shell' })

    await fireEvent.click(button)
    await fireEvent.click(button)

    await vi.waitFor(() => expect(intents()).toEqual([{ type: 'InstallShell' }]))
  })

  it('shows progress while it runs and takes both buttons away', () => {
    draw({ type: 'Running' })

    expect(screen.getByRole('status').textContent).toContain('Installing gentle-shell')
    for (const name of ['Install gentle-shell', 'Check again']) {
      expect((screen.getByRole('button', { name }) as HTMLButtonElement).disabled).toBe(true)
    }
  })

  it('says what went wrong and lets the user try again', () => {
    draw({ type: 'Failed', data: { message: 'the install ended with exit code 1\nnpm ERR! 404' } })

    expect(screen.getByRole('alert').textContent).toContain('npm ERR! 404')
    expect((screen.getByRole('button', { name: 'Install gentle-shell' }) as HTMLButtonElement).disabled).toBe(
      false,
    )
  })

  it('tells the user what to do when npm itself could not run', () => {
    draw({ type: 'Failed', data: { message: 'could not start npm.cmd: not found' } })

    expect(screen.getByRole('alert').textContent).toContain('install Node.js')
  })

  it('says it finished and offers to look again instead of installing again', () => {
    draw({ type: 'Done' })

    expect(screen.getByRole('status').textContent).toContain('gentle-shell was installed')
    expect(screen.queryByRole('button', { name: 'Install gentle-shell' })).toBeNull()
    expect(screen.getByRole('button', { name: 'Check again' })).toBeTruthy()
  })

  it('looks for gentle-shell again when asked', async () => {
    draw()

    await fireEvent.click(screen.getByRole('button', { name: 'Check again' }))

    await vi.waitFor(() => expect(intents()).toEqual([{ type: 'CheckShellAgain' }]))
  })
})
