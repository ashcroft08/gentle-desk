// The Profiles pane (0.1.12, S1, T2, L11): the orchestrator switch of the chats
// opened from now on, then the ACTIVE profile read-only - its name, its two
// routes and one row per agent - and the one way out of the pane, the terminal.
// One test per rule: the drawn profile, the switch's intent, the empty state, the
// failed state with its safe retry, the refresh on open and on focus, and the
// terminal button.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, within } from '@testing-library/svelte'
import type { ProfilesView_Serialize as ProfilesView } from '../contract/bindings'
import { handlerArgs, resetTauriMock, setHandler } from '../test-support/tauri-mock'
import ProfilesPane from './ProfilesPane.svelte'
import {
  ACTIVE_NOTE,
  EDIT_IN_TERMINAL,
  EDIT_NOTE,
  FAILED,
  MODE_FULL_DELEGATION,
  MODE_NOTE,
  MODE_ORGANIC,
  NO_PROFILES,
  READING,
  RETRY_SAFE,
  TRY_AGAIN,
} from './profiles-copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => null)
  setHandler('open_terminal', () => null)
})

/** The document the user's own machine holds, trimmed to what the pane draws. */
function profiles(over: Partial<ProfilesView> = {}): ProfilesView {
  return {
    status: { type: 'Ready' },
    active: 'Mine',
    mode: 'organic',
    profiles: [
      {
        name: 'Mine',
        active: true,
        orchestrator: { role: 'orchestrator', model: 'prov/big', thinking: 'medium', tier: null },
        roles: [
          { role: 'fallback', model: 'prov/fb', thinking: 'low', tier: null },
          { role: 'worker', model: 'prov/small', thinking: 'high', tier: null },
        ],
        mode: 'organic',
      },
    ],
    ...over,
  }
}

function draw(view: ProfilesView | null = profiles()): HTMLElement {
  const { container } = render(ProfilesPane, { props: { profiles: view } })
  return container as HTMLElement
}

/** The intents the pane sent, oldest first. */
function intents(): unknown[] {
  return handlerArgs('dispatch_intent').map((args) => args.intent)
}

/** The mode intents only: the pane also reads the file once per open. */
function modes(): unknown[] {
  return intents().filter((intent) => (intent as { type: string }).type === 'SetOrchestratorMode')
}

/** How many times the pane asked for a fresh read of the file. */
function reads(): number {
  return intents().filter((intent) => (intent as { type: string }).type === 'RefreshProfiles')
    .length
}

describe('the Profiles pane', () => {
  it('draws the active profile: its name, its two routes and one row per agent', () => {
    const pane = draw()

    expect(within(pane).getByText('Mine')).toBeTruthy()
    expect(within(pane).getByText(ACTIVE_NOTE)).toBeTruthy()
    // The reserved orchestrator route, then the fallback chain's model.
    expect(pane.querySelector('.pair.orchestrator')?.textContent).toContain('prov/big')
    expect(pane.querySelector('.pair.orchestrator')?.textContent).toContain('medium')
    expect(pane.querySelector('.pair.fallback')?.textContent).toContain('prov/fb')
    // One table row per other role, with its model and its effort.
    const rows = within(pane).getAllByRole('row').slice(1)
    expect(
      rows.map((row) => [...row.querySelectorAll('td')].map((cell) => cell.textContent?.trim())),
    ).toEqual([
      ['fallback', 'prov/fb', 'low'],
      ['worker', 'prov/small', 'high'],
    ])
    expect(within(pane).getByRole('columnheader', { name: 'Effort' })).toBeTruthy()
  })

  it('sends the orchestrator mode the user picks, and says it is not stored', async () => {
    const pane = draw()

    const organic = within(pane).getByRole('radio', { name: MODE_ORGANIC })
    const full = within(pane).getByRole('radio', { name: MODE_FULL_DELEGATION })
    expect(organic.getAttribute('aria-checked')).toBe('true')
    expect(full.getAttribute('aria-checked')).toBe('false')
    expect(within(pane).getByText(MODE_NOTE)).toBeTruthy()

    await fireEvent.click(full)
    expect(modes()).toEqual([{ type: 'SetOrchestratorMode', data: { mode: 'full-delegation' } }])

    // The push is what moves the switch: a click sends, it never guesses.
    expect(full.getAttribute('aria-checked')).toBe('false')
  })

  it('says there are no profiles in a quiet line, never a broken pane', () => {
    const pane = draw(profiles({ active: null, profiles: [] }))

    expect(within(pane).getByText(NO_PROFILES)).toBeTruthy()
    // The switch stays usable: the mode is the session's, not the file's.
    expect(within(pane).getByRole('radiogroup')).toBeTruthy()
  })

  it('says why the file could not be read, and that reading again is safe', async () => {
    const reason = 'the file is not a profiles document'
    const pane = draw(profiles({ status: { type: 'Failed', data: reason }, active: null, profiles: [] }))

    expect(within(pane).getByText(FAILED(reason))).toBeTruthy()
    expect(within(pane).getByText(RETRY_SAFE)).toBeTruthy()
    const before = reads()
    await fireEvent.click(within(pane).getByRole('button', { name: TRY_AGAIN }))
    expect(reads()).toBe(before + 1)
  })

  it('reads the file when the pane opens and again when the desk is focused', async () => {
    draw()
    await vi.waitFor(() => expect(intents()).toEqual([{ type: 'RefreshProfiles' }]))

    // The terminal window closes and the desk takes the focus back: one read,
    // which is what makes the note under the button true.
    window.dispatchEvent(new Event('focus'))
    await vi.waitFor(() =>
      expect(intents()).toEqual([{ type: 'RefreshProfiles' }, { type: 'RefreshProfiles' }]),
    )
  })

  it('opens the shell profiles screen in a terminal window', async () => {
    const pane = draw()

    await fireEvent.click(within(pane).getByRole('button', { name: EDIT_IN_TERMINAL }))
    expect(handlerArgs('open_terminal')).toEqual([{ action: 'Profiles' }])
    expect(within(pane).getByText(EDIT_NOTE)).toBeTruthy()
  })

  it('says the terminal could not open, instead of a button that does nothing', async () => {
    setHandler('open_terminal', () => Promise.reject('gentle-shell could not be found'))
    const pane = draw()

    await fireEvent.click(within(pane).getByRole('button', { name: EDIT_IN_TERMINAL }))

    await vi.waitFor(() =>
      expect(within(pane).getByRole('status').textContent).toContain(
        'gentle-shell could not be found',
      ),
    )
  })

  it('says it is reading while the first answer is on its way', () => {
    const pane = draw(null)
    expect(within(pane).getByText(READING)).toBeTruthy()
  })
})
