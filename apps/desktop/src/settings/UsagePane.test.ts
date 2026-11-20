// The Usage pane (settings-0.1.12 S6's deferred half): the subscription usage
// the shell's own sources report, as the maintainer asked for it - per provider
// its name, its plan when it has one, and one row per window with a small gauge
// and the countdown to its reset. A provider with no credentials says "no
// data"; the pane never turns that into a zero.
//
// The pane reads NOTHING (settings-snapshot): the dialog's one load already
// ran the sources, so appearing costs no run. "Check again" asks the dialog for
// another load through `onRefresh`, and `refreshing` is the quiet line that
// makes the ask visible while it is on its way.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, within } from '@testing-library/svelte'
import type { UsageView_Serialize as UsageView } from '../contract/bindings'
import { handlerArgs, resetTauriMock, setHandler } from '../test-support/tauri-mock'
import UsagePane from './UsagePane.svelte'
import { usageCopy } from './usage-copy'

/** The pane's own words in English, read through the dictionary (0.2.0, D4). */
const copy = usageCopy('en')

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

/** A load the user asked for, as the dialog would be asked for one. */
function ask(): () => void {
  return vi.fn()
}

function draw(
  view: UsageView | null = installed(),
  refreshing = false,
  onRefresh: () => void = ask(),
): HTMLElement {
  const { container } = render(UsagePane, { props: { usage: view, refreshing, onRefresh } })
  return container as HTMLElement
}

describe('the Usage pane', () => {
  it('draws each provider with its plan and one gauge row per window', () => {
    const pane = draw()

    // Only the providers with a reading are drawn (the fixture's opencode-go
    // answered no data, as a machine without that credential does), and the one
    // spending most comes first.
    const cards = [...pane.querySelectorAll('.usage-provider')]
    expect(cards.map((card) => card.querySelector('.uname')?.textContent)).toEqual([
      'commandcode',
    ])
    expect(cards[0].textContent).toContain('Plan: individual goat')

    const rows = [...cards[0].querySelectorAll('.uwindow')]
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
    expect(cards[0].querySelector('.tag')?.textContent).toBe(copy.limitReached)
  })

  it('does not list a provider with no credentials, and never draws it as 0%', () => {
    const pane = draw()

    const names = [...pane.querySelectorAll('.uname')].map((name) => name.textContent)
    expect(names).not.toContain('opencode-go')
    // And no drawn card turns a missing reading into a zero.
    for (const percent of pane.querySelectorAll('.upercent')) {
      expect(percent.textContent).not.toBe('0%')
    }
  })

  it('says the sources are not installed, and installs only through the terminal', async () => {
    const pane = draw({ status: { type: 'Missing', data: 'not installed' }, providers: [] })

    expect(within(pane).getByText(copy.missing)).toBeTruthy()
    await fireEvent.click(within(pane).getByRole('button', { name: copy.install }))
    expect(handlerArgs('open_terminal')).toEqual([{ action: 'InstallUsageSources' }])
  })

  it('says why an unreadable slice could not be read, and that retrying is safe', () => {
    const pane = draw({ status: { type: 'Unreadable', data: 'permission denied' }, providers: [] })

    expect(within(pane).getByText(copy.cannotRead('permission denied'))).toBeTruthy()
    expect(within(pane).getByRole('button', { name: copy.checkAgain })).toBeTruthy()
  })

  it('says why a failed read failed and offers the retry', () => {
    const pane = draw({ status: { type: 'Failed', data: 'wrong schema' }, providers: [] })

    expect(within(pane).getByText(copy.cannotRead('wrong schema'))).toBeTruthy()
    expect(within(pane).getByRole('button', { name: copy.checkAgain })).toBeTruthy()
  })

  it('draws a quiet line while the read runs, with no spinner and no read of its own', () => {
    const again = ask()
    const pane = draw({ status: { type: 'Reading' }, providers: [] }, false, again)

    expect(within(pane).getByText(copy.reading)).toBeTruthy()
    expect(within(pane).queryByRole('progressbar')).toBeNull()
    expect(within(pane).queryByRole('button', { name: copy.checkAgain })).toBeNull()
    // Nothing was asked for: the dialog's one load is what read this.
    expect(again).not.toHaveBeenCalled()
  })

  it("says the load is running while the user's own re-check is on its way", () => {
    // A card the last load answered, and a re-check in flight: the same quiet
    // line, so a click that takes seconds is never silent.
    const pane = draw(installed(), true)

    expect(within(pane).getByText(copy.reading)).toBeTruthy()
    expect(pane.querySelector('.usage-provider')).toBeNull()
  })

  it('says so when the sources answered with no provider at all', () => {
    const pane = draw({ status: { type: 'Installed' }, providers: [] })
    expect(within(pane).getByText(copy.noProviders)).toBeTruthy()
  })

  it('reads nothing when it appears, and asks for one load when the user asks', async () => {
    const again = ask()
    const pane = draw(installed(), false, again)
    expect(again).not.toHaveBeenCalled()

    await fireEvent.click(within(pane).getByRole('button', { name: copy.checkAgain }))

    expect(again).toHaveBeenCalledTimes(1)
    // Still no intent: the pane's one read path is the dialog's load.
    expect(handlerArgs('dispatch_intent')).toEqual([])
  })
})
