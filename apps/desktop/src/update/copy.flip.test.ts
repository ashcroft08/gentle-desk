// An update line DRAWN from its dictionary (0.2.0, S2, S3): one local language
// write redraws every word the line owns in the same tick - no push, no restart
// (S2) - while the version the updater reported keeps reading exactly as it was
// sent (D7).
//
// The English words are asserted literally: they are what the line has drawn
// since 0.1.9. The two columns themselves are pinned in `copy.test.ts`.
import '../test-support/dom-env'
import { afterEach, describe, expect, it } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import { tick } from 'svelte'
import { setLanguage } from '../i18n/language.svelte'
import UpdateNotice from './UpdateNotice.svelte'
import type { UpdateState } from './state'

afterEach(cleanup)

/** Draws the line in one state; the three actions do nothing here. */
function draw(state: UpdateState): void {
  render(UpdateNotice, {
    props: { state, onInstall: () => {}, onRetry: () => {}, onDismiss: () => {} },
  })
}

describe('a drawn update line', () => {
  it('flips the offered version and its two buttons in the same tick', async () => {
    setLanguage('en')
    draw({ kind: 'available', version: '0.2.0' })

    expect(
      screen.getByText('Version 0.2.0 is available. The app will close while it installs.'),
    ).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Install update' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Not now' })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(
      screen.getByText('La versión 0.2.0 está disponible. La app se cerrará mientras se instala.'),
    ).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Instalar actualización' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Ahora no' })).toBeTruthy()
    // D7: the version is the updater's, so it reads the same afterwards.
    expect(screen.getByText(/0\.2\.0/)).toBeTruthy()

    setLanguage('en')
  })

  it('flips the failure and its retry, leaving the updater message alone', async () => {
    const message = 'signature check failed'

    setLanguage('en')
    draw({ kind: 'failed', version: '0.2.0', message })
    expect(
      screen.getByText(`The update to version 0.2.0 could not be installed: ${message}`),
    ).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Try again' })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(
      screen.getByText(`No se pudo instalar la actualización a la versión 0.2.0: ${message}`),
    ).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Reintentar' })).toBeTruthy()
    // D7: the updater's own message is drawn as it arrived.
    expect(screen.getByText(/signature check failed/)).toBeTruthy()

    setLanguage('en')
  })
})
