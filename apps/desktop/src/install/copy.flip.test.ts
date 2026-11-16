// A missing-shell screen DRAWN from its dictionary (0.2.0, S2, S3): one local
// language write redraws everything it owns - the title, the five explanations,
// both buttons, the progress name and the wait it counts - in the same tick, with
// no push and no restart (S2). The command it asks the user to run and npm's own
// failure text are not ours, so they read the same afterwards (D7).
//
// The English words are asserted literally: they are what the screen has drawn
// since shell-choice S4. The two columns themselves are pinned in `copy.test.ts`.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import { tick } from 'svelte'
import type { InstallView } from '../contract/bindings'
import { resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import { setLanguage } from '../i18n/language.svelte'
import MissingShell from './MissingShell.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The command itself: what must be typed, in both languages (D7). */
const COMMAND = 'npm install -g gentle-pi'
/** npm's own last words on a failed install, shown exactly as it wrote them (D7). */
const NPM_SAID = 'npm ERR! code EACCES'

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
})

describe('a drawn install screen', () => {
  it('flips its title, its explanations and its buttons in the same tick', async () => {
    setLanguage('en')
    render(MissingShell, { props: { install: { type: 'Idle' } as InstallView } })

    expect(screen.getByRole('heading', { name: 'gentle-shell is not installed' })).toBeTruthy()
    expect(screen.getByText(COMMAND)).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Install gentle-shell' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Check again' })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(screen.getByRole('heading', { name: 'gentle-shell no está instalado' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Instalar gentle-shell' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Comprobar de nuevo' })).toBeTruthy()
    // D7: the command is typed as it must be, so neither language rewrites it.
    expect(screen.getByText(COMMAND)).toBeTruthy()

    setLanguage('en')
  })

  it('names the progress bar and says the failure in the language of the screen', async () => {
    setLanguage('en')
    render(MissingShell, { props: { install: { type: 'Running' } as InstallView } })

    expect(screen.getByRole('progressbar', { name: 'Installing gentle-shell' })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(screen.getByRole('progressbar', { name: 'Instalando gentle-shell' })).toBeTruthy()

    cleanup()
    render(MissingShell, {
      props: { install: { type: 'Failed', data: { message: NPM_SAID } } as InstallView },
    })

    expect(screen.getByText('La instalación no terminó')).toBeTruthy()
    // D7: npm's own output is drawn in a pre, exactly as it arrived.
    expect(screen.getByText(NPM_SAID)).toBeTruthy()

    setLanguage('en')
    await tick()

    expect(screen.getByText('The install did not finish')).toBeTruthy()
    expect(screen.getByText(NPM_SAID)).toBeTruthy()
  })
})
