// The Account pane (0.1.12, S1, T3, L11): one row per provider the shell's auth
// store lists, its own check's answer, and the three buttons that hand the work
// to a terminal. Names only - the pane never carries a key, a token or an
// account identity, so no test here expects one.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, within } from '@testing-library/svelte'
import type { AuthView_Serialize as AuthView } from '../contract/bindings'
import { handlerArgs, resetTauriMock, setHandler } from '../test-support/tauri-mock'
import AccountPane from './AccountPane.svelte'
import {
  CHECK_AGAIN,
  CONNECTED,
  FAILED,
  LOGIN,
  LOGOUT,
  NOT_READY,
  NO_PROVIDERS,
  READING,
  RETRY_SAFE,
} from './account-copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => null)
  setHandler('open_terminal', () => null)
})

function auth(over: Partial<AuthView> = {}): AuthView {
  return {
    status: { type: 'Ready' },
    providers: [
      { name: 'anthropic', status: 'Ready', reason: null },
      { name: 'commandcode', status: 'NotReady', reason: 'OAuth token expired' },
    ],
    ...over,
  }
}

function draw(view: AuthView | null = auth()): HTMLElement {
  const { container } = render(AccountPane, { props: { auth: view } })
  return container as HTMLElement
}

function intents(): unknown[] {
  return handlerArgs('dispatch_intent').map((args) => args.intent)
}

describe('the Account pane', () => {
  it('lists one row per provider, with what its own check said', () => {
    const pane = draw()
    const rows = within(pane).getAllByRole('listitem')

    expect(rows.map((row) => row.querySelector('.provider-name')?.textContent)).toEqual([
      'anthropic',
      'commandcode',
    ])
    expect(rows[0].textContent).toContain(CONNECTED)
    expect(rows[0].querySelector('.tag')?.classList.contains('ok')).toBe(true)
    expect(rows[1].textContent).toContain(NOT_READY)
    expect(rows[1].textContent).toContain('OAuth token expired')
  })

  it('says no provider is signed in when the store is empty', () => {
    const pane = draw(auth({ providers: [] }))
    expect(within(pane).getByText(NO_PROVIDERS)).toBeTruthy()
  })

  it('says why the store could not be read, and that checking again is safe', () => {
    const pane = draw(auth({ status: { type: 'Failed', data: 'permission denied' } }))

    expect(within(pane).getByText(FAILED('permission denied'))).toBeTruthy()
    expect(within(pane).getByText(RETRY_SAFE)).toBeTruthy()
    expect(within(pane).getByRole('button', { name: CHECK_AGAIN })).toBeTruthy()
  })

  it('says it is reading while the first answer is on its way', () => {
    expect(within(draw(null)).getByText(READING)).toBeTruthy()
    expect(
      within(draw(auth({ status: { type: 'Unknown' } }))).getByText(READING),
    ).toBeTruthy()
  })

  it('checks again when the user asks', async () => {
    const pane = draw()

    await fireEvent.click(within(pane).getByRole('button', { name: CHECK_AGAIN }))
    expect(intents()).toEqual([{ type: 'RefreshAuth' }])
  })

  it('opens login and logout in a terminal window', async () => {
    const pane = draw()

    await fireEvent.click(within(pane).getByRole('button', { name: LOGIN }))
    await fireEvent.click(within(pane).getByRole('button', { name: LOGOUT }))
    expect(handlerArgs('open_terminal')).toEqual([{ action: 'Login' }, { action: 'Logout' }])
  })
})
