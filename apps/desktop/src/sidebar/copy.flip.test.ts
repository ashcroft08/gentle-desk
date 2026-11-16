// A sidebar DRAWN from its dictionary (0.2.0, S3): choosing Español redraws every
// word the sidebar owns in the same tick the language changes - no push, no
// restart (S2) - while the chat title, the workspace label and the timestamps Rust
// sends keep arriving exactly as they were written (D7).
//
// The English words are asserted literally: they are what the sidebar has drawn
// since 01a and the states board, so a change of copy is a change here. The two
// columns themselves are pinned in `copy.test.ts`.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import { tick } from 'svelte'
import { activity, chatRow, resetTauriMock, setHandler, viewState, workspaceRow } from '../test-support/tauri-mock'
import { serveSidebar } from '../test-support/sidebar'
import { setLanguage } from '../i18n/language.svelte'
import Sidebar from './Sidebar.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** A chat title Rust sends: never translated, only wrapped in our words (D7). */
const TITLE = 'First chat'

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

describe('a drawn sidebar', () => {
  it('flips its own words in the same tick, with no push and no restart', async () => {
    setLanguage('en')
    serveSidebar({ view: viewState({ list_status: { type: 'Complete' }, total_sessions: 0 }) })
    render(Sidebar)

    expect(await screen.findByText('No workspaces yet')).toBeTruthy()
    expect(screen.getByRole('complementary', { name: 'Workspaces' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'New session' })).toBeTruthy()
    // The empty state's own way forward and the header's own icon: one word,
    // drawn twice.
    expect(screen.getAllByRole('button', { name: 'New workspace' })).toHaveLength(2)
    expect(screen.getByText('Preview').getAttribute('title')).toBe(
      'Preview: under active development - fixes ship fast',
    )
    // The gear's own name is the Settings dialog's word, read in this language:
    // the seam T6 recorded is closed here, so the gear is never half Spanish.
    expect(screen.getByRole('button', { name: 'Settings' })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(screen.getByText('Aún no hay espacios de trabajo')).toBeTruthy()
    expect(screen.getByRole('complementary', { name: 'Espacios de trabajo' })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Nueva sesión' })).toBeTruthy()
    expect(screen.getAllByRole('button', { name: 'Nuevo espacio de trabajo' })).toHaveLength(2)
    expect(screen.getByText('Vista previa').getAttribute('title')).toBe(
      'Vista previa: en desarrollo activo; las correcciones llegan rápido',
    )
    expect(screen.getByRole('button', { name: 'Ajustes' })).toBeTruthy()

    setLanguage('en')
  })

  it('flips the words around a chat row, and leaves the title Rust sent alone', async () => {
    setLanguage('en')
    serveSidebar({
      view: viewState({ total_sessions: 2, activity: [activity('a', 'Working')] }),
      rows: [workspaceRow('one'), chatRow('a', { title: TITLE })],
      total: 2,
    })
    render(Sidebar)

    const trigger = await screen.findByRole('button', { name: `Chat actions for ${TITLE}` })
    expect(screen.getByRole('button', { name: 'New chat in one' })).toBeTruthy()
    expect(screen.getByRole('img', { name: 'Working' })).toBeTruthy()
    const menu = await openMenu(trigger)
    expect(within(menu).getByRole('menuitem', { name: 'Rename' })).toBeTruthy()
    expect(within(menu).getByRole('menuitem', { name: 'Copy file path' })).toBeTruthy()
    expect(within(menu).getByRole('menuitem', { name: 'Close chat' })).toBeTruthy()
    expect(within(menu).getByRole('menuitem', { name: 'Delete' })).toBeTruthy()

    setLanguage('es')
    await tick()

    // D7: the title is Rust's and stays exactly as it arrived.
    expect(screen.getByText(TITLE)).toBeTruthy()
    expect(screen.getByRole('button', { name: `Acciones del chat: ${TITLE}` })).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Chat nuevo en one' })).toBeTruthy()
    // The mark's name is this app's own word for the state, so it flips too.
    expect(screen.getByRole('img', { name: 'Trabajando' })).toBeTruthy()
    const flipped = screen.getByRole('menu', { name: `Acciones del chat: ${TITLE}` })
    expect(within(flipped).getByRole('menuitem', { name: 'Renombrar' })).toBeTruthy()
    expect(within(flipped).getByRole('menuitem', { name: 'Copiar ruta del archivo' })).toBeTruthy()
    expect(within(flipped).getByRole('menuitem', { name: 'Cerrar chat' })).toBeTruthy()
    expect(within(flipped).getByRole('menuitem', { name: 'Eliminar' })).toBeTruthy()

    setLanguage('en')
  })

  it('flips the gear name while an update waits, dot and news and all', async () => {
    setLanguage('en')
    serveSidebar({
      view: viewState({
        total_sessions: 2,
        updates: {
          status: 'Ready',
          desk: { current: '0.1.14', latest: '0.1.14', update_available: false },
          pi: { current: '1.5.2', latest: '2.0.5', update_available: true },
          shell: { current: '4.7.0', latest: null, update_available: null },
        },
      }),
      rows: [workspaceRow('one'), chatRow('a', { title: TITLE })],
      total: 2,
    })
    render(Sidebar)

    // The Settings word is the dialog's (settings/copy.ts) and the news is the
    // sidebar's: one word, one owner, and the two flip together.
    expect(
      await screen.findByRole('button', { name: 'Settings - an update is available' }),
    ).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(
      screen.getByRole('button', { name: 'Ajustes - hay una actualización disponible' }),
    ).toBeTruthy()

    setLanguage('en')
  })

  it('flips the inline rename input and the confirm dialog it offers', async () => {
    setLanguage('en')
    serveSidebar({
      view: viewState({ total_sessions: 2 }),
      rows: [workspaceRow('one'), chatRow('a', { title: TITLE })],
      total: 2,
    })
    render(Sidebar)
    await openMenu(await screen.findByRole('button', { name: `Chat actions for ${TITLE}` }))
    await fireEvent.click(screen.getByRole('menuitem', { name: 'Rename' }))
    expect(screen.getByRole('textbox', { name: 'Chat name' })).toBeTruthy()

    setLanguage('es')
    await tick()

    expect(screen.getByRole('textbox', { name: 'Nombre del chat' })).toBeTruthy()
    // The confirm dialog's own words are in the same dictionary.
    await fireEvent.keyDown(screen.getByRole('textbox', { name: 'Nombre del chat' }), {
      key: 'Escape',
    })
    await openMenu(screen.getByRole('button', { name: `Acciones del chat: ${TITLE}` }))
    await fireEvent.click(screen.getByRole('menuitem', { name: 'Eliminar' }))
    const dialog = await screen.findByRole('dialog')
    expect(within(dialog).getByText('¿Eliminar este chat?')).toBeTruthy()
    // The consequence line is the dialog's own, in the same dictionary (S3).
    expect(within(dialog).getByText('chat').tagName).toBe('B')
    expect(dialog.textContent ?? '').toContain('almacén de sesiones de gentle-shell')
    expect(within(dialog).getByRole('button', { name: 'Cancelar' })).toBeTruthy()
    expect(within(dialog).getByRole('button', { name: 'Eliminar' })).toBeTruthy()

    setLanguage('en')
  })

  it('flips the removal dialog in the same tick, offer, disclosure and all', async () => {
    setLanguage('en')
    const path = 'C:\\dev\\one'
    serveSidebar({
      view: viewState({ total_sessions: 2 }),
      rows: [workspaceRow('one', { path }), chatRow('a', { title: TITLE })],
      total: 2,
    })
    setHandler('workspace_removal_offer', () => ({
      key: 'one',
      path,
      label: 'one',
      chats: 1,
      bytes: 1_800_000,
    }))
    render(Sidebar)

    await fireEvent.click(await screen.findByRole('button', { name: 'Workspace actions for one' }))
    await fireEvent.click(await screen.findByRole('menuitem', { name: 'Remove workspace...' }))
    const dialog = await screen.findByRole('dialog')
    await within(dialog).findByText('Remove this workspace?')

    expect(within(dialog).getByText('1 chat (1.7 MB)').tagName).toBe('B')
    expect(within(dialog).getByText('Show 1 chat (1.7 MB)').tagName).toBe('SUMMARY')
    expect(within(dialog).getByRole('button', { name: 'Delete 1 chat and remove' })).toBeTruthy()

    setLanguage('es')
    await tick()

    // S7: the row menu's item and every word of the dialog it opened, together.
    expect(within(dialog).getByText('¿Quitar este espacio de trabajo?')).toBeTruthy()
    expect(within(dialog).getByText('1 chat (1.7 MB)').tagName).toBe('B')
    expect(within(dialog).getByText('Mostrar 1 chat (1.7 MB)').tagName).toBe('SUMMARY')
    expect(within(dialog).getByRole('button', { name: 'Cancelar' })).toBeTruthy()
    expect(within(dialog).getByRole('button', { name: 'Eliminar 1 chat y quitar' })).toBeTruthy()
    // The workspace's own title is Rust's and stays exactly as it arrived (D7).
    expect(within(dialog).getByText(TITLE)).toBeTruthy()

    setLanguage('en')
  })
})

/** The row's actions menu, opened the way the user opens it. */
async function openMenu(trigger: HTMLElement): Promise<HTMLElement> {
  await fireEvent.click(trigger)
  return await screen.findByRole('menu')
}
