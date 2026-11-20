// View A, the resume screen (welcome-screen T2, S2, S3, R1, R3, R4, R6): the rose,
// the title, the workspace worked in last with its ONE action, its recent chats,
// and the other workspaces as quiet links. Every word of it is the view's own,
// both languages drawn from the one dictionary, and every action is one intent.
//
// The three states the sheet draws are here: a workspace with no chat yet says
// so and keeps the action alone; a folder that is gone disables the action with
// Rust's own reason (verbatim, never reworded) and links it to the button; and
// with no configured workspace only the old empty-state line is left. A null
// slice means a chat is open, so the screen draws nothing at all.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte'
import type { UiIntent } from '../contract/bindings'
import { handlerArgs, resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import { welcomeChat, welcomeLink, welcomeView, welcomeWorkspace } from '../test-support/welcome'
import { setLanguage } from '../i18n/language.svelte'
import Home from './Home.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

const MIN = 60_000
const DAY = 24 * 60 * MIN
const HINT = 'Choose a chat in the workspace tree, or start a new session.'

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
  setLanguage('en')
})

/** The intents the screen sent, without the language write every test makes. */
function intents(): UiIntent[] {
  return handlerArgs('dispatch_intent')
    .map((args) => args.intent as UiIntent)
    .filter((intent) => intent.type !== 'SetLanguage')
}

describe('view A, the resume screen', () => {
  it('draws the rose, the title and the last workspace with its one action', () => {
    const last = welcomeWorkspace({ label: 'Curriculum', last_activity_ms: Date.now() - 21 * MIN })
    const recents = [welcomeChat('C:\\sessions\\alpha.jsonl', { modified_ms: Date.now() - 21 * MIN })]
    render(Home, { props: { welcome: welcomeView({ last, recents }) } })

    expect(screen.getByRole('img', { name: 'The gentle-shell rose' })).toBeTruthy()
    // The page's h1 is the header's; this title is the view's own h2.
    expect(screen.getByRole('heading', { level: 2, name: 'Pick up where you left off' })).toBeTruthy()
    expect(screen.getByText('Curriculum')).toBeTruthy()
    expect(screen.getByText('· last activity 21m ago')).toBeTruthy()
    expect(screen.getByRole('button', { name: 'New session in Curriculum' })).toBeTruthy()
  })

  it('starts a new chat in the last workspace, naming its path', async () => {
    const last = welcomeWorkspace({ path: 'C:\\dev\\curriculum', label: 'Curriculum' })
    render(Home, { props: { welcome: welcomeView({ last }) } })

    const action = screen.getByRole('button', { name: 'New session in Curriculum' })
    expect((action as HTMLButtonElement).disabled).toBe(false)
    await fireEvent.click(action)

    expect(intents()).toEqual([
      { type: 'NewChatInWorkspace', data: { path: 'C:\\dev\\curriculum' } },
    ])
  })

  it('lists the recent chats with their age and opens one by its RAW session path', async () => {
    const last = welcomeWorkspace({ label: 'Curriculum', last_activity_ms: Date.now() - 21 * MIN })
    const recents = [
      welcomeChat('C:\\sessions\\alpha.jsonl', {
        title: 'corré ls en la raíz del repo',
        modified_ms: Date.now() - 21 * MIN,
      }),
      welcomeChat('C:\\sessions\\beta.jsonl', {
        title: 'que puedo borrar de mi carpeta',
        modified_ms: Date.now() - 2 * DAY,
      }),
    ]
    render(Home, { props: { welcome: welcomeView({ last, recents }) } })

    expect(
      screen.getByRole('heading', { level: 3, name: 'Recent chats in Curriculum' }),
    ).toBeTruthy()
    const row = screen.getByRole('button', { name: /corré ls en la raíz del repo/ })
    expect(row.textContent).toContain('21m')
    expect(
      screen.getByRole('button', { name: /que puedo borrar de mi carpeta/ }).textContent,
    ).toContain('2d')

    await fireEvent.click(row)

    expect(intents()).toEqual([
      { type: 'OpenChat', data: { chat_id: 'C:\\sessions\\alpha.jsonl' } },
    ])
  })

  it('says a workspace holds no chat yet, with no list and the action alone', async () => {
    const last = welcomeWorkspace({ label: 'gentle-desk', chat_count: 0, last_activity_ms: null })
    render(Home, { props: { welcome: welcomeView({ last, recents: [] }) } })

    expect(screen.getByText('no chats yet')).toBeTruthy()
    expect(screen.queryByRole('list')).toBeNull()
    expect(screen.queryByRole('heading', { level: 3 })).toBeNull()
    expect(screen.queryByText(/last activity/)).toBeNull()

    // The action stays, and it still starts the chat in that empty workspace.
    await fireEvent.click(screen.getByRole('button', { name: 'New session in gentle-desk' }))
    expect(intents()).toEqual([
      { type: 'NewChatInWorkspace', data: { path: 'C:\\dev\\gentle-desk' } },
    ])
  })

  it('disables the action for a folder that is gone, printing Rust’s reason verbatim', async () => {
    const last = welcomeWorkspace({
      path: 'C:\\dev\\tmp.cblyG8CFAR',
      label: 'tmp.cblyG8CFAR',
      available: false,
      reason: 'This folder no longer exists.',
    })
    render(Home, { props: { welcome: welcomeView({ last }) } })

    const action = screen.getByRole('button', { name: 'New session in tmp.cblyG8CFAR' })
    // A disabled control cannot be activated at all: the browser refuses the
    // click, so the intent can never be sent for a folder that is gone.
    expect((action as HTMLButtonElement).disabled).toBe(true)

    const described = document.getElementById(action.getAttribute('aria-describedby') ?? '')
    expect(described).not.toBeNull()
    expect(described?.textContent).toContain('This folder no longer exists.')
    expect(described?.textContent).toContain('Pick another workspace in the sidebar.')
    // No other action is invented for a folder that cannot host a chat.
    expect(screen.getAllByRole('button')).toHaveLength(1)
  })

  it('keeps the old empty-state line when no workspace is configured', () => {
    render(Home, { props: { welcome: welcomeView({ last: null }) } })

    expect(screen.getByRole('img', { name: 'The gentle-shell rose' })).toBeTruthy()
    expect(screen.getByText(HINT)).toBeTruthy()
    expect(screen.queryByRole('button')).toBeNull()
    expect(screen.queryByRole('heading')).toBeNull()
  })

  it('draws nothing at all when a chat is open, which is what a null slice means', () => {
    const { container } = render(Home, { props: { welcome: null } })

    expect(container.textContent?.trim()).toBe('')
    expect(container.querySelector('canvas')).toBeNull()
    expect(container.querySelector('button')).toBeNull()
  })

  it('offers the other workspaces as quiet links, without repeating the last one', async () => {
    const last = welcomeWorkspace({ label: 'gentle-desk' })
    const others = [welcomeLink('Proyecto'), welcomeLink('Kallari_Pagina_web')]
    render(Home, { props: { welcome: welcomeView({ last, others }) } })

    expect(screen.getByText('Other workspaces:')).toBeTruthy()
    // Rust leaves the last workspace out of the quiet links, and so does the view.
    expect(screen.queryByRole('button', { name: 'gentle-desk' })).toBeNull()

    await fireEvent.click(screen.getByRole('button', { name: 'Proyecto' }))
    expect(intents()).toEqual([
      { type: 'NewChatInWorkspace', data: { path: 'C:\\dev\\Proyecto' } },
    ])
  })

  it('disables a quiet link whose folder is gone, so it starts nothing', async () => {
    const others = [welcomeLink('gone', { path: 'C:\\dev\\gone', available: false })]
    render(Home, { props: { welcome: welcomeView({ others }) } })

    const quiet = screen.getByRole('button', { name: 'gone' })
    // Disabled is the guard: a browser refuses to click it, and Rust would NOT
    // refuse the intent (a quiet link that is not available must not offer one).
    expect((quiet as HTMLButtonElement).disabled).toBe(true)

    // The available links are the ones that can dispatch, as the test above shows.
    expect(intents()).toEqual([])
  })

  it('draws its own words in Spanish', () => {
    setLanguage('es')
    const last = welcomeWorkspace({ label: 'Curriculum', last_activity_ms: Date.now() - 21 * MIN })
    render(Home, {
      props: {
        welcome: welcomeView({
          last,
          recents: [welcomeChat('C:\\s\\a.jsonl', { title: 'una charla' })],
        }),
      },
    })

    expect(screen.getByRole('img', { name: 'La rosa de gentle shell' })).toBeTruthy()
    expect(screen.getByRole('heading', { level: 2, name: 'Sigue donde lo dejaste' })).toBeTruthy()
    expect(screen.getByText('· última actividad hace 21m')).toBeTruthy()
    expect(screen.getByRole('button', { name: 'Nueva sesión en Curriculum' })).toBeTruthy()
    expect(screen.getByRole('heading', { level: 3, name: 'Chats recientes de Curriculum' })).toBeTruthy()
    expect(screen.queryByText('Pick up where you left off')).toBeNull()
  })
})
