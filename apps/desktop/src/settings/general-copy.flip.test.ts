// A General pane DRAWN from its dictionary (0.9.0 general S5): choosing Español
// redraws every word the pane owns in the same tick the language changes - no
// push, no restart - while the four tool names, the versions the check read and
// the `—` of a row nobody read yet keep arriving exactly as they are.
//
// The English words are asserted literally: they are the G4 table's, so a change
// of copy is a change here. The two columns themselves are pinned in
// `general-copy.test.ts`, and the rules the pane draws by in
// `GeneralPane.test.ts`.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, within } from '@testing-library/svelte'
import { tick } from 'svelte'
import type { GeneralView_Serialize as GeneralView } from '../contract/bindings'
import { resetTauriMock } from '../test-support/tauri-mock'
import { setLanguage } from '../i18n/language.svelte'
import GeneralPane from './GeneralPane.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The two rows that carry a fix hint, and the two versions that never flip. */
function mixed(): GeneralView {
  return {
    tools: [
      { tool: 'GentleShell', state: 'Installed', version: '4.9.0' },
      { tool: 'Pi', state: 'Unreadable' },
      { tool: 'Node', state: 'Missing' },
      { tool: 'Git', state: 'Installed', version: '2.51.0' },
    ],
    checked_ms: Date.now() - 3 * 60_000,
  }
}

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

/** The pane drawn with one answer; the props are the dialog's own signals. */
function draw(): HTMLElement {
  const { container } = render(GeneralPane, {
    props: { general: mixed(), refreshing: false, onRefresh: () => {} },
  })
  return container as HTMLElement
}

describe('a drawn General pane in either language', () => {
  it('flips its states, its hints and its check row, and renames no tool', async () => {
    setLanguage('en')
    const pane = draw()

    expect(pane.textContent).toContain('Installed')
    expect(pane.textContent).toContain('Could not read')
    expect(pane.textContent).toContain('Missing')
    expect(pane.textContent).toContain('Install it with')
    expect(pane.textContent).toContain('Last checked: 3m ago')
    expect(within(pane).getByRole('button', { name: 'Check again' })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(pane.textContent).toContain('Instalado')
    expect(pane.textContent).toContain('No se pudo leer')
    expect(pane.textContent).toContain('Falta')
    expect(pane.textContent).toContain('Instálalo con')
    expect(pane.textContent).toContain('Última comprobación: hace 3m')
    expect(within(pane).getByRole('button', { name: 'Comprobar de nuevo' })).toBeTruthy()
    // The commands and the versions are facts, so they read the same twice.
    expect(pane.textContent).toContain('npm install -g gentle-pi')
    expect(pane.textContent).toContain('winget install OpenJS.NodeJS.LTS')
    expect(pane.textContent).toContain('4.9.0')
    expect(pane.textContent).toContain('2.51.0')
    // The tool names are the terminal's own words, never respelled per language.
    expect([...pane.querySelectorAll('.tname')].map((one) => one.textContent?.trim())).toEqual([
      'Gentle Shell',
      'pi',
      'node',
      'git',
    ])

    setLanguage('en')
  })

  it('flips the checking row while the answer is still on its way', async () => {
    setLanguage('en')
    const { container } = render(GeneralPane, {
      props: { general: null, refreshing: false, onRefresh: () => {} },
    })
    const pane = container as HTMLElement

    expect(within(pane).getByText('Checking...')).toBeTruthy()
    expect(pane.textContent).toContain('Last checked: never')

    setLanguage('es')
    await tick()

    expect(within(pane).getByText('Comprobando...')).toBeTruthy()
    expect(pane.textContent).toContain('Última comprobación: nunca')

    setLanguage('en')
  })
})
