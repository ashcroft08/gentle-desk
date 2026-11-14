// The Usage pane (settings-0.1.12 S6's deferred half): the subscription usage
// the shell's own sources report, as the maintainer asked for it - per provider
// its name, its plan when it has one, and one row per window with a small gauge
// and the countdown to its reset. A provider with no credentials says "no
// data"; the pane never turns that into a zero.
//
// The one read is the pane's own: it asks when it appears, because a run of the
// sources takes seconds (the cheap correct answer: one run when the user asks),
// and "Check again" asks for another. Nothing here reads a chat, so no open chat
// can hold the pane back.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, within } from '@testing-library/svelte'
import type { UsageView_Serialize as UsageView } from '../contract/bindings'
import { handlerArgs, resetTauriMock, setHandler } from '../test-support/tauri-mock'
import UsagePane from './UsagePane.svelte'
import {
  CANNOT_READ,
  CHECK_AGAIN,
  INSTALL,
  LIMIT_REACHED,
  MISSING,
  NO_DATA,
  NO_PROVIDERS,
  READING,
} from './usage-copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => null)
  setHandler('open_terminal', () => null)
})

/** The real document this machine's sources printed, as the pane receives it. */
function installed(): UsageView {
  return {
    status: { type: 'Installed' },
    providers: [
      {
        provider: 'opencode-go',
        status: 'NoData',
        plan: null,
        error: null,
        limits: [],
      },
      {
        provider: 'commandcode',
        status: 'Ok',
        plan: 'individual goat',
        error: null,
        limits: [
          {
            name: 'command code',
            limit_reached: false,
            windows: [
              { label: '5h', used_percent: 4.6771898400000005, resets_in: 'resets in 2h' },
              { label: 'week', used_percent: 88.272975324, resets_in: 'resets in 6d' },
            ],
          },
          {
            name: 'credits',
            limit_reached: true,
            windows: [
              { label: 'billing', used_percent: 44.13765581735515, resets_in: 'resets in 30d' },
            ],
          },
        ],
      },
    ],
  }
}

function draw(view: UsageView | null = installed()): HTMLElement {
  const { container } = render(UsagePane, { props: { usage: view } })
  return container as HTMLElement
}

function intents(): unknown[] {
  return handlerArgs('dispatch_intent').map((args) => args.intent)
}

/** How many usage reads the pane asked for. */
function reads(): number {
  return intents().filter((intent) => (intent as { type: string }).type === 'RefreshUsage').length
}

describe('the Usage pane', () => {
  it('draws each provider with its plan and one gauge row per window', () => {
    const pane = draw()

    const cards = [...pane.querySelectorAll('.usage-provider')]
    expect(cards.map((card) => card.querySelector('.uname')?.textContent)).toEqual([
      'opencode-go',
      'commandcode',
    ])
    expect(cards[1].textContent).toContain('Plan: individual goat')

    const rows = [...cards[1].querySelectorAll('.uwindow')]
    expect(rows.map((row) => row.querySelector('.ulabel')?.textContent)).toEqual([
      '5h',
      'week',
      'billing',
    ])
    // The gauge is the context chip's language: a bar and the rounded share.
    expect(rows[0].querySelector('.upercent')?.textContent).toBe('5%')
    expect(rows[1].querySelector('.upercent')?.textContent).toBe('88%')
    expect(rows[1].textContent).toContain('resets in 6d')
    expect(
      (rows[1].querySelector('.ubar i') as HTMLElement).style.width,
    ).toBe('88%')
    // The provider's own mark reaches the row.
    expect(cards[1].querySelector('.tag')?.textContent).toBe(LIMIT_REACHED)
  })

  it('draws a provider with no credentials as no data, never as 0%', () => {
    const pane = draw()

    const card = pane.querySelector('.usage-provider') as HTMLElement
    expect(card.textContent).toContain(NO_DATA)
    expect(card.querySelector('.upercent')).toBeNull()
    expect(card.querySelector('.ubar')).toBeNull()
  })

  it('says the sources are not installed, and installs only through the terminal', async () => {
    const pane = draw({ status: { type: 'Missing', data: 'not installed' }, providers: [] })

    expect(within(pane).getByText(MISSING)).toBeTruthy()
    await fireEvent.click(within(pane).getByRole('button', { name: INSTALL }))
    expect(handlerArgs('open_terminal')).toEqual([{ action: 'InstallUsageSources' }])
  })

  it('says why an unreadable slice could not be read, and that retrying is safe', () => {
    const pane = draw({ status: { type: 'Unreadable', data: 'permission denied' }, providers: [] })

    expect(within(pane).getByText(CANNOT_READ('permission denied'))).toBeTruthy()
    expect(within(pane).getByRole('button', { name: CHECK_AGAIN })).toBeTruthy()
  })

  it('says why a failed read failed and offers the retry', () => {
    const pane = draw({ status: { type: 'Failed', data: 'wrong schema' }, providers: [] })

    expect(within(pane).getByText(CANNOT_READ('wrong schema'))).toBeTruthy()
    expect(within(pane).getByRole('button', { name: CHECK_AGAIN })).toBeTruthy()
  })

  it('draws a quiet line while the read runs, with no spinner and no second read', async () => {
    const pane = draw({ status: { type: 'Reading' }, providers: [] })

    expect(within(pane).getByText(READING)).toBeTruthy()
    expect(within(pane).queryByRole('progressbar')).toBeNull()
    expect(within(pane).queryByRole('button', { name: CHECK_AGAIN })).toBeNull()
    await vi.waitFor(() => expect(reads()).toBe(1))
  })

  it('says so when the sources answered with no provider at all', () => {
    const pane = draw({ status: { type: 'Installed' }, providers: [] })
    expect(within(pane).getByText(NO_PROVIDERS)).toBeTruthy()
  })

  it('reads once when it appears, and again only when the user asks', async () => {
    const pane = draw()

    await vi.waitFor(() => expect(reads()).toBe(1))
    await fireEvent.click(within(pane).getByRole('button', { name: CHECK_AGAIN }))
    expect(reads()).toBe(2)
  })
})
