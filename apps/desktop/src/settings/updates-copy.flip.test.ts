// An Updates pane DRAWN from its dictionary (0.2.0, S3): choosing Español redraws
// every word the pane owns in the same tick the language changes - no push, no
// restart (S2) - while the versions the check reported keep arriving exactly as
// they were written (D7).
//
// The English words are asserted literally: they are what the pane has drawn since
// 0.1.12, so a change of copy is a change here. The two columns themselves are
// pinned in `updates-copy.test.ts`.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, within } from '@testing-library/svelte'
import { tick } from 'svelte'
import type { UpdatesView_Serialize as UpdatesView } from '../contract/bindings'
import { resetTauriMock, setHandler } from '../test-support/tauri-mock'
import { setLanguage } from '../i18n/language.svelte'
import UpdatesPane from './UpdatesPane.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => null)
  setHandler('open_terminal', () => null)
})

/** The check the user's machine really answers: pi 1.5.2 against 2.0.5. */
function updates(over: Partial<UpdatesView> = {}): UpdatesView {
  return {
    status: 'Ready',
    desk: { current: '0.1.11', latest: '0.1.12', update_available: true },
    pi: { current: '1.5.2', latest: '2.0.5', update_available: true },
    shell: { current: '4.7.0', latest: null, update_available: null },
    ...over,
  }
}

function draw(view: UpdatesView | null = updates(), openChats = 0): HTMLElement {
  const { container } = render(UpdatesPane, { props: { updates: view, openChats } })
  return container as HTMLElement
}

describe('a drawn Updates pane', () => {
  it('flips its tag, its buttons and the headlines, and never a version', async () => {
    setLanguage('en')
    const pane = draw()

    expect(pane.querySelector('.tag')?.textContent?.trim()).toBe('Update available')
    expect(pane.textContent).toContain('This app 0.1.11 \u2192 0.1.12')
    expect(pane.textContent).toContain('pi 1.5.2 \u2192 2.0.5')
    expect(pane.textContent).toContain('This app 0.1.11 \u00b7 pi 1.5.2 \u00b7 shell 4.7.0')
    expect(within(pane).getByRole('button', { name: 'Check again' })).toBeTruthy()
    expect(within(pane).getByRole('button', { name: 'Update' })).toBeTruthy()
    expect(within(pane).getByRole('button', { name: 'Update extensions' })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(pane.querySelector('.tag')?.textContent?.trim()).toBe('Actualización disponible')
    expect(pane.textContent).toContain('Esta app 0.1.11 \u2192 0.1.12')
    expect(pane.textContent).toContain('pi 1.5.2 \u2192 2.0.5')
    // D7: the versions are the check's, so only the two labels we own change.
    expect(pane.textContent).toContain('Esta app 0.1.11 \u00b7 pi 1.5.2 \u00b7 shell 4.7.0')
    expect(within(pane).getByRole('button', { name: 'Comprobar de nuevo' })).toBeTruthy()
    expect(within(pane).getByRole('button', { name: 'Actualizar' })).toBeTruthy()
    expect(within(pane).getByRole('button', { name: 'Actualizar extensiones' })).toBeTruthy()

    setLanguage('en')
  })

  it('flips the up-to-date tag and the notes under the card', async () => {
    setLanguage('en')
    const pane = draw(
      updates({
        desk: { current: '0.1.12', latest: '0.1.12', update_available: false },
        pi: { current: '2.0.5', latest: '2.0.5', update_available: false },
      }),
    )
    expect(pane.querySelector('.tag')?.textContent?.trim()).toBe('Everything is up to date')
    expect(pane.textContent).toContain("Update extensions opens a terminal window")

    setLanguage('es')
    await tick()

    expect(pane.querySelector('.tag')?.textContent?.trim()).toBe('Todo está al día')
    expect(pane.textContent).toContain('Actualizar extensiones abre una ventana de terminal')

    setLanguage('en')
  })

  it('flips the app-only note: the banner installs this app, not a terminal', async () => {
    setLanguage('en')
    const only = updates({
      pi: { current: '2.0.5', latest: '2.0.5', update_available: false },
      shell: { current: '4.8.0', latest: '4.8.0', update_available: false },
    })
    const pane = draw(only)
    expect(pane.textContent).toContain('This app updates itself: use the update banner at the top.')
    expect(within(pane).queryByRole('button', { name: 'Update' })).toBeNull()

    setLanguage('es')
    await tick()

    expect(pane.textContent).toContain('Esta app se actualiza sola')

    setLanguage('en')
  })

  it('flips the chats-closed gate and keeps it counting the same chats', async () => {
    setLanguage('en')
    const pane = draw(updates(), 2)
    expect(pane.textContent).toContain('2 chats are open')
    expect((within(pane).getByRole('button', { name: 'Update' }) as HTMLButtonElement).disabled).toBe(
      true,
    )

    setLanguage('es')
    await tick()

    expect(pane.textContent).toContain('2 chats están abiertos')
    expect(
      (within(pane).getByRole('button', { name: 'Actualizar' }) as HTMLButtonElement).disabled,
    ).toBe(true)

    setLanguage('en')
  })

  it('flips the two states that are not a card', async () => {
    setLanguage('en')
    const notChecked = draw(updates({ status: 'Unknown' }))
    expect(within(notChecked).getByText('Not checked yet.')).toBeTruthy()
    const checking = draw(updates({ status: 'Checking' }))
    expect(within(checking).getByText('Checking...')).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(within(notChecked).getByText('Todavía sin comprobar.')).toBeTruthy()
    expect(within(checking).getByText('Comprobando...')).toBeTruthy()

    setLanguage('en')
  })
})
