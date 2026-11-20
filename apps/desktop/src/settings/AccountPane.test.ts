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
import { accountCopy } from './account-copy'

/** The pane's own words in English, read through the dictionary (0.2.0, D4). */
const copy = accountCopy('en')

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

/** One ask for the dialog's load, as the pane sends it. */
function ask(): () => void {
  return vi.fn()
}

function draw(view: AuthView | null = auth(), onRefresh: () => void = ask()): HTMLElement {
  const { container } = render(AccountPane, { props: { auth: view, onRefresh } })
  return container as HTMLElement
}

describe('the Account pane', () => {
  it('lists one row per provider, with what its own check said', () => {
    const pane = draw()
    const rows = within(pane).getAllByRole('listitem')

    expect(rows.map((row) => row.querySelector('.provider-name')?.textContent)).toEqual([
      'anthropic',
      'commandcode',
    ])
    expect(rows[0].textContent).toContain(copy.connected)
    expect(rows[0].querySelector('.tag')?.classList.contains('ok')).toBe(true)
    expect(rows[1].textContent).toContain(copy.notReady)
    expect(rows[1].textContent).toContain('OAuth token expired')
  })

  it('says no provider is signed in when the store is empty', () => {
    const pane = draw(auth({ providers: [] }))
    expect(within(pane).getByText(copy.noProviders)).toBeTruthy()
  })

  it('says why the store could not be read, and that checking again is safe', () => {
    const pane = draw(auth({ status: { type: 'Failed', data: 'permission denied' } }))

    expect(within(pane).getByText(copy.failed('permission denied'))).toBeTruthy()
    expect(within(pane).getByText(copy.retrySafe)).toBeTruthy()
    expect(within(pane).getByRole('button', { name: copy.checkAgain })).toBeTruthy()
  })

  it('says it is reading while the first answer is on its way', () => {
    expect(within(draw(null)).getByText(copy.reading)).toBeTruthy()
    expect(
      within(draw(auth({ status: { type: 'Unknown' } }))).getByText(copy.reading),
    ).toBeTruthy()
  })

  it('asks for one load when the user checks again', async () => {
    const again = ask()
    const pane = draw(auth(), again)

    await fireEvent.click(within(pane).getByRole('button', { name: copy.checkAgain }))

    expect(again).toHaveBeenCalledTimes(1)
    // Not an intent: the one read path of the dialog is its load
    // (settings-snapshot).
    expect(handlerArgs('dispatch_intent')).toEqual([])
  })

  it('reads nothing when it appears, and asks for a load when the desk is focused', async () => {
    const again = ask()
    draw(auth(), again)

    // Appearing costs nothing: the dialog's one load already answered for this
    // pane, so the quiet "Reading..." is not what makes it fill in.
    expect(again).not.toHaveBeenCalled()

    // The terminal window closes and the desk takes the focus back: one ask.
    window.dispatchEvent(new Event('focus'))
    await vi.waitFor(() => expect(again).toHaveBeenCalledTimes(1))
  })

  // `pi auth login|logout` need a provider the desk does not have, so signing in
  // or out is pi's own /login and /logout INSIDE the shell: one button opens the
  // shell, and there is no second button that would open the very same window.
  it('opens the shell in a terminal window with one button, and says what to type', async () => {
    const pane = draw()

    await fireEvent.click(within(pane).getByRole('button', { name: copy.openShell }))
    expect(handlerArgs('open_terminal')).toEqual([{ action: 'Login' }])
    expect(within(pane).queryByRole('button', { name: /sign out in terminal/i })).toBeNull()
    expect(pane.textContent).toContain('/login')
    expect(pane.textContent).toContain('/logout')
  })
})
