// The first-start question (shell-choice S2), drawn after the gentle-shell
// welcome screen: the app found pi on this machine and asks whether to use it or
// keep gentle shell separate. Each answer is one intent; nothing is read from
// pi, so the screen names only the folder.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import { handlerArgs, resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import Welcome from './Welcome.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

const PI = 'C:/Users/someone/.pi/agent'

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
})

function intents(): unknown[] {
  return handlerArgs('dispatch_intent').map((args) => args.intent)
}

describe('the welcome question', () => {
  it('names the app, the folder it found and the one question', () => {
    render(Welcome, { props: { piPath: PI } })

    expect(screen.getByRole('heading', { name: 'Welcome to gentle shell' })).toBeTruthy()
    expect(screen.getByText('We found pi on this machine')).toBeTruthy()
    expect(screen.getByText(PI)).toBeTruthy()
  })

  it('offers the two homes, the first marked as the recommended one', () => {
    render(Welcome, { props: { piPath: PI } })

    expect(screen.getByRole('button', { name: 'Use my pi setup' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Keep it separate' })).toBeTruthy()
    expect(screen.getByText('RECOMMENDED')).toBeTruthy()
    expect(screen.getByText('links to ~/.pi/agent')).toBeTruthy()
    expect(screen.getByText('creates ~/.gentle-shell/agent')).toBeTruthy()
  })

  it('sends the link choice for Use my pi setup, once', async () => {
    render(Welcome, { props: { piPath: PI } })

    await fireEvent.click(screen.getByRole('button', { name: 'Use my pi setup' }))

    expect(intents()).toEqual([{ type: 'SetShellHome', data: { choice: 'link' } }])
  })

  it('sends the isolated choice for Keep it separate, once', async () => {
    render(Welcome, { props: { piPath: PI } })

    await fireEvent.click(screen.getByRole('button', { name: 'Keep it separate' }))

    expect(intents()).toEqual([{ type: 'SetShellHome', data: { choice: 'isolated' } }])
  })

  it('puts the keyboard on the recommended answer when it opens', () => {
    render(Welcome, { props: { piPath: PI } })

    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Use my pi setup' }))
  })

  it('says what happens with no pi, so nobody wonders where the question went', () => {
    render(Welcome, { props: { piPath: PI } })

    expect(
      screen.getByText(
        'No pi on the machine? This screen is skipped and gentle shell starts in its own space.',
      ),
    ).toBeTruthy()
  })
})
