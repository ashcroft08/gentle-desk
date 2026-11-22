// An Updates pane DRAWN from its dictionary (0.2.0, S3; 0.8.1 S8): choosing
// Español redraws every word the pane owns in the same tick the language changes -
// no push, no restart (S2) - while the versions the check reported keep arriving
// exactly as they were written (D7) and the age of the last check is the house's
// own in both languages (D11).
//
// The English words are quoted literally: they are what the pane draws, so a change
// of copy is a change here. The two columns themselves are pinned in
// `updates-copy.test.ts`.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, within } from '@testing-library/svelte'
import { tick } from 'svelte'
import type {
  PackagesView_Serialize as PackagesView,
  UpdatesView_Serialize as UpdatesView,
} from '../contract/bindings'
import { resetTauriMock, setHandler } from '../test-support/tauri-mock'
import { setLanguage } from '../i18n/language.svelte'
import UpdatesPane from './UpdatesPane.svelte'
import { updatesCopy } from './updates-copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** Three minutes after the check, so the age is the house's own `3m`/`3m`. */
const CHECKED_MS = 1_700_000_000_000
const NOW_MS = CHECKED_MS + 3 * 60_000

// The testing library only cleans up on its own with vitest globals on.
afterEach(() => {
  cleanup()
  vi.restoreAllMocks()
})

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => null)
  setHandler('open_terminal', () => null)
  vi.spyOn(Date, 'now').mockReturnValue(NOW_MS)
})

/** The check the user's machine really answers: pi and Gentle Shell both behind. */
function updates(over: Partial<UpdatesView> = {}): UpdatesView {
  return {
    status: 'Ready',
    desk: { current: '0.1.11', latest: '0.1.12', update_available: true },
    pi: { current: '1.5.2', latest: '2.0.5', update_available: true },
    shell: { current: '4.8.0', latest: '4.9.2', update_available: true },

    packages: { available: false, behind: [], unknown: false, truncated: false, checked: null },
    checked_ms: CHECKED_MS,
    ...over,
  }
}

function draw(view: UpdatesView | null = updates(), openChats = 0): HTMLElement {
  const { container } = render(UpdatesPane, { props: { updates: view, openChats } })
  return container as HTMLElement
}

/** The Extensions block's slice: unasked unless a test says otherwise. */
function packages(over: Partial<PackagesView> = {}): PackagesView {
  return { available: false, behind: [], unknown: false, truncated: false, checked: null, ...over }
}

/** The pane's extensions block, the group no version drives (D14). */
function ext(pane: HTMLElement): HTMLElement {
  const found = pane.querySelector('.card.ext')
  if (!(found instanceof HTMLElement)) throw new Error('the pane drew no extensions block')
  return found
}

describe('a drawn Updates pane', () => {
  it('flips its tag, its row names, its buttons and its last check, and never a version', async () => {
    setLanguage('en')
    const pane = draw()

    expect(pane.querySelector('.tag')?.textContent?.trim()).toBe('Update available')
    expect(pane.textContent).toContain('This app')
    expect(pane.textContent).toContain('Gentle Shell')
    expect(pane.textContent).toContain('0.1.11 \u2192 0.1.12')
    expect(pane.textContent).toContain('1.5.2 \u2192 2.0.5')
    expect(pane.textContent).toContain('4.8.0 \u2192 4.9.2')
    expect(pane.textContent).toContain('Last checked: 3m ago')
    expect(within(pane).getByRole('button', { name: 'Check again' })).toBeTruthy()
    expect(within(pane).getByRole('button', { name: 'Update' })).toBeTruthy()
    expect(within(pane).getByRole('button', { name: 'Update Gentle Shell' })).toBeTruthy()
    expect(within(pane).getByRole('button', { name: 'Update extensions' })).toBeTruthy()
    expect(within(pane).getByRole('button', { name: 'Skip this version' })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(pane.querySelector('.tag')?.textContent?.trim()).toBe('Actualización disponible')
    expect(pane.textContent).toContain('Esta app')
    // D10: the product's name reads the same in both languages.
    expect(pane.textContent).toContain('Gentle Shell')
    // D7: the versions are the check's, so only the words around them change.
    expect(pane.textContent).toContain('0.1.11 \u2192 0.1.12')
    expect(pane.textContent).toContain('1.5.2 \u2192 2.0.5')
    expect(pane.textContent).toContain('4.8.0 \u2192 4.9.2')
    // D11: the age is `ago`'s word in both, and the sentence around it flips.
    expect(pane.textContent).toContain('Última comprobación: hace 3m')
    expect(within(pane).getByRole('button', { name: 'Comprobar de nuevo' })).toBeTruthy()
    expect(within(pane).getByRole('button', { name: 'Actualizar' })).toBeTruthy()
    expect(within(pane).getByRole('button', { name: 'Actualizar Gentle Shell' })).toBeTruthy()
    expect(within(pane).getByRole('button', { name: 'Actualizar extensiones' })).toBeTruthy()
    expect(within(pane).getByRole('button', { name: 'Omitir esta versión' })).toBeTruthy()

    setLanguage('en')
  })

  it('flips the up-to-date tag and the extensions block', async () => {
    setLanguage('en')
    const pane = draw(
      updates({
        desk: { current: '0.1.12', latest: '0.1.12', update_available: false },
        pi: { current: '2.0.5', latest: '2.0.5', update_available: false },
        shell: { current: '4.9.2', latest: '4.9.2', update_available: false },
      }),
    )
    expect(pane.querySelector('.tag')?.textContent?.trim()).toBe('Everything is up to date')
    // Nothing is behind, so no row carries a sentence and the terminal note is gone.
    expect(pane.querySelectorAll('.reason')).toHaveLength(0)
    expect(pane.querySelector('.note')).toBeNull()
    expect(pane.textContent).toContain('There is no way to tell which ones are behind')

    setLanguage('es')
    await tick()

    expect(pane.querySelector('.tag')?.textContent?.trim()).toBe('Todo está al día')
    expect(pane.textContent).toContain('No se puede saber cuáles están desactualizadas')

    setLanguage('en')
  })

  it("flips the app row's own sentence when the app is the one behind", async () => {
    setLanguage('en')
    const pane = draw(
      updates({
        pi: { current: '2.0.5', latest: '2.0.5', update_available: false },
        shell: { current: '4.9.2', latest: '4.9.2', update_available: false },
      }),
    )
    expect(pane.textContent).toContain('This app updates itself: use the update banner at the top.')

    setLanguage('es')
    await tick()

    expect(pane.textContent).toContain('Esta app se actualiza sola')

    setLanguage('en')
  })

  it('flips the sentence of a row that could not be read (S8, D12)', async () => {
    setLanguage('en')
    const pane = draw(
      updates({
        desk: { current: '0.1.12', latest: '0.1.12', update_available: false },
        pi: { current: '2.0.5', latest: '2.0.5', update_available: false },
        shell: {
          current: '4.8.0',
          latest: null,
          update_available: null,
          reason: 'Offline',
        },
      }),
    )
    expect(pane.querySelector('.tag')?.textContent?.trim()).toBe('Not checked')
    expect(pane.textContent).toContain(
      'Could not check the latest gentle-shell version on npm: offline.',
    )

    setLanguage('es')
    await tick()

    expect(pane.querySelector('.tag')?.textContent?.trim()).toBe('Sin comprobar')
    expect(pane.textContent).toContain(
      'No se pudo comprobar la última versión de gentle-shell en npm: sin conexión.',
    )
    // The installed version is the check's, and it is still there.
    expect(pane.textContent).toContain('4.8.0')

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

  it('flips the two states that are not a card, and the last check they keep', async () => {
    setLanguage('en')
    const checking = draw(updates({ status: 'Checking' }))
    expect(within(checking).getByText('Checking...')).toBeTruthy()
    expect(checking.textContent).toContain('Last checked: 3m ago')
    // D13: nothing has been checked yet, and the one line that says it is the age.
    const never = draw(updates({ status: 'Unknown', checked_ms: undefined }))
    expect(never.textContent).toContain('Last checked: never')

    setLanguage('es')
    await tick()

    expect(within(checking).getByText('Comprobando...')).toBeTruthy()
    expect(checking.textContent).toContain('Última comprobación: hace 3m')
    expect(never.textContent).toContain('Última comprobación: nunca')

    setLanguage('en')
  })

  it('flips every extension block state, and never a name or a version pi reported', async () => {
    setLanguage('en')
    const behind = draw(
      updates({
        packages: packages({
          available: true,
          behind: [{ name: 'pi-hud', current: '0.3.1' }, { name: 'pi-tools' }],
        }),
      }),
    )
    const upToDate = draw(updates({ packages: packages({ available: true }) }))
    const unreadable = draw(updates({ packages: packages({ unknown: true }) }))
    const unasked = draw(updates())

    expect(ext(behind).textContent).toContain('2 extensions have an update available')
    expect(ext(behind).textContent).toContain('pi-hud')
    expect(ext(upToDate).textContent).toContain('The extensions are up to date.')
    expect(ext(unreadable).textContent).toContain(
      'Could not check the extensions: pi did not answer.',
    )
    expect(ext(unasked).textContent).toContain('There is no way to tell which ones are behind')
    expect(ext(unasked).textContent).toContain('You can check it by opening a session.')

    setLanguage('es')
    await tick()

    expect(ext(behind).textContent).toContain('2 extensiones con actualización disponible')
    // D7: the names and the versions are pi's own, so they read the same in both.
    expect(ext(behind).textContent).toContain('pi-hud')
    expect(ext(behind).textContent).toContain('0.3.1')
    expect(ext(upToDate).textContent).toContain('Las extensiones están al día.')
    expect(ext(unreadable).textContent).toContain(
      'No se pudo revisar las extensiones: pi no respondió.',
    )
    expect(ext(unasked).textContent).toContain('No se puede saber cuáles están desactualizadas')
    expect(ext(unasked).textContent).toContain('Se puede revisar al abrir una sesión.')

    setLanguage('en')
  })
})

/** The dictionary is only read to keep the pin honest about what it compares. */
void updatesCopy
