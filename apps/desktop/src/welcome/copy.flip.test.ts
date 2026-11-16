// A welcome screen DRAWN from its dictionary (0.2.0, S2, S3): one local language
// write redraws both answers, their points and their paths in the same tick - no
// push, no restart (S2) - while the folder the machine reported keeps reading
// exactly as it was found (D7).
//
// The English words are asserted literally: they are what the screen has drawn
// since shell-choice S2. The two columns themselves are pinned in `copy.test.ts`.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import { tick } from 'svelte'
import { resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import { setLanguage } from '../i18n/language.svelte'
import Welcome from './Welcome.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The folder Rust found; a path, so it is drawn as it was found (D7). */
const PI = 'C:\\Users\\ana\\.pi\\agent'

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
})

describe('a drawn welcome screen', () => {
  it('flips its title, both answers, their points and their paths in the same tick', async () => {
    setLanguage('en')
    render(Welcome, { props: { piPath: PI } })

    expect(screen.getByRole('heading', { level: 1, name: 'Welcome to gentle shell' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Use my pi setup' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Keep it separate' })).toBeTruthy()
    expect(screen.getByText('RECOMMENDED')).toBeTruthy()
    expect(screen.getByText('Your pi settings are never edited')).toBeTruthy()
    expect(screen.getByText('links to ~/.pi/agent')).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(
      screen.getByRole('heading', { level: 1, name: 'Te damos la bienvenida a gentle shell' }),
    ).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Usar mi configuración de pi' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Mantenerlo separado' })).toBeTruthy()
    expect(screen.getByText('RECOMENDADO')).toBeTruthy()
    expect(screen.getByText('Tu configuración de pi nunca se modifica')).toBeTruthy()
    expect(screen.getByText('enlaza con ~/.pi/agent')).toBeTruthy()
    // D7: the folder is the machine's, so it reads the same afterwards.
    expect(screen.getByText(PI)).toBeTruthy()

    setLanguage('en')
  })
})
