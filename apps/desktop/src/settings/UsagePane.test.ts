// The Usage pane (settings-0.1.12 S6's deferred half; settings-redesign S8, L5,
// L7): the subscription usage the shell's own sources report, as the maintainer
// asked for it - per provider its name and plan on one line, then one sub-heading
// per limit and one row per window with a small gauge and the countdown to its
// reset, all of it on ONE four-column grid declared once (D1, D2, D4). No card:
// a provider is a section of the whole column, separated from the next by a
// hairline and its own air (R1). A provider with no credentials says "no data";
// the pane never turns that into a zero, and a provider with no windows says so
// in its own sentence (D3).
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
import APP_CSS from '../app.css?raw'
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
  it('draws each provider as a full-column section named by its heading (S8, D1)', () => {
    const pane = draw()

    // Only the providers with a reading are drawn (the fixture's opencode-go
    // answered no data, as a machine without that credential does), and the one
    // spending most comes first.
    const sections = [...pane.querySelectorAll('section.uprov')]
    expect(sections).toHaveLength(1)

    const head = sections[0].querySelector('.uprov-head') as HTMLElement
    const name = head.querySelector('h4.uname') as HTMLElement
    expect(name.textContent).toBe('commandcode')
    // The section is NAMED by that heading, never by a bare glyph or a plan.
    const labelled = sections[0].getAttribute('aria-labelledby')
    expect(labelled).toBe(name.id)
    expect(document.getElementById(labelled as string)).toBe(name)
    // The plan sits at the right edge of the same line (D1).
    expect(head.querySelector('.uplan')?.textContent).toBe('Plan: individual goat')
  })

  it('draws one row per window on the four columns, with the gauge and the share', () => {
    const pane = draw()

    const rows = [...pane.querySelectorAll('.uwindow')]
    expect(rows.map((row) => row.querySelector('.ulabel')?.textContent)).toEqual([
      '5h',
      'week',
      'billing',
    ])
    // The gauge is the context chip's language: a bar and the rounded share.
    expect(rows[0].querySelector('.upercent')?.textContent).toBe('5%')
    expect(rows[1].querySelector('.upercent')?.textContent).toBe('88%')
    expect(rows[1].textContent).toContain('resets in 6d')
    expect((rows[1].querySelector('.ubar i') as HTMLElement).style.width).toBe('88%')
    // Four cells per row, in the order the grid declares them.
    expect([...rows[0].children].map((cell) => cell.className.split(' ')[0])).toEqual([
      'ulabel',
      'ubar',
      'upercent',
      'ureset',
    ])
  })

  it('names each limit as a quiet sub-heading and marks the one it reached (D2)', () => {
    const pane = draw()

    expect(
      [...pane.querySelectorAll('.ulimit-head')].map((head) =>
        (head.querySelector('.ulimit-name') as HTMLElement).textContent,
      ),
    ).toEqual(['command code', 'credits'])
    // The provider's own mark reaches the row it belongs to.
    const reached = [...pane.querySelectorAll('.ulimit.reached')]
    expect(reached).toHaveLength(1)
    expect(reached[0].querySelector('.ulimit-head .tag')?.textContent).toBe(copy.limitReached)
  })

  it('leaves no card anywhere: the grid is the whole column', () => {
    const pane = draw()

    expect(pane.querySelector('.usecs')).toBeTruthy()
    expect(pane.querySelector('.card')).toBeNull()
    expect(pane.querySelector('.usage-provider')).toBeNull()
    expect(pane.querySelector('.ucards')).toBeNull()
  })

  it('does not list a provider with no credentials, and never draws it as 0%', () => {
    const pane = draw()

    const names = [...pane.querySelectorAll('.uname')].map((name) => name.textContent)
    expect(names).not.toContain('opencode-go')
    // And no drawn row turns a missing reading into a zero.
    for (const percent of pane.querySelectorAll('.upercent')) {
      expect(percent.textContent).not.toBe('0%')
    }
  })

  // D3: a provider that reported no window has its own sentence, and never the
  // one the pane says when the sources named no provider at all.
  it('says a provider has no windows instead of borrowing the no-provider line', () => {
    const pane = draw({
      status: { type: 'Installed' },
      providers: [
        {
          provider: 'claude-bridge',
          status: 'Ok',
          plan: 'pro',
          error: null,
          limits: [],
        },
      ],
    })

    expect(within(pane).getByText(copy.noWindows)).toBeTruthy()
    expect(within(pane).queryByText(copy.noProviders)).toBeNull()
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
    // A reading the last load answered, and a re-check in flight: the same quiet
    // line, so a click that takes seconds is never silent.
    const pane = draw(installed(), true)

    expect(within(pane).getByText(copy.reading)).toBeTruthy()
    expect(pane.querySelector('.uprov')).toBeNull()
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

/** The sheet without its comments: a selector list is never prose. */
const SHEET = APP_CSS.replace(/\/\*[\s\S]*?\*\//g, '')

/** One rule's DECLARATIONS, by its exact selector (spacing normalized). */
function declarations(selector: string): string {
  const exact = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&').replace(/ /g, '\\s+')
  const pattern = new RegExp(`(?:^|\\n)\\s*${exact}\\s*\\{`)
  const start = pattern.exec(SHEET)
  if (start === null) throw new Error(`no rule for ${selector}`)
  const open = start.index + start[0].length
  return SHEET.slice(open, SHEET.indexOf('}', open))
}

// The stylesheet the markup above depends on (R1, R3, R4; D1, D2, D4). Every
// assertion reads a DECLARATION out of app.css, never the raw text, which
// explains itself in comments and would satisfy a substring search with the
// declaration gone. What a stylesheet read cannot prove - how the four columns
// LOOK in a real window - is reported as unverified.
describe("the Usage pane's stylesheet", () => {
  it('declares the four columns ONCE and hangs every row on them (R3, D4)', () => {
    expect([...SHEET.matchAll(/--ucols\s*:/g)]).toHaveLength(1)
    expect(declarations('.settings-pane .usecs')).toMatch(
      /--ucols:\s*96px\s+minmax\(0,\s*1fr\)\s+46px\s+112px\s*;/,
    )
    expect(declarations('.settings-pane .uwindow')).toMatch(
      /grid-template-columns:\s*var\(--ucols\)\s*;/,
    )
    // The list is the pane's whole column, so no row can shrink the grid.
    expect(declarations('.settings-pane .usecs')).toMatch(/width:\s*100%\s*;/)
  })

  it('puts the reset column at a fixed left edge instead of a trailing auto margin', () => {
    // `margin-left: auto` is what made the widest countdown start further left.
    expect(declarations('.settings-pane .ureset')).not.toMatch(/margin-left/)
    expect(declarations('.settings-pane .upercent')).toMatch(/text-align:\s*right\s*;/)
    expect(declarations('.settings-pane .ubar')).not.toMatch(/width:\s*\d+px\s*;/)
  })

  it('separates two providers with one hairline and no box (R1)', () => {
    expect(declarations('.settings-pane .uprov + .uprov')).toMatch(
      /border-top:\s*1px solid var\(--border\)\s*;/,
    )
    for (const selector of ['.settings-pane .uprov', '.settings-pane .uprov-head']) {
      const rule = declarations(selector)
      expect(rule, selector).not.toMatch(/(^|[\s;])border(-[a-z]+)?\s*:/)
      expect(rule, selector).not.toMatch(/(^|[\s;])background(-[a-z]+)?\s*:/)
      expect(rule, selector).not.toMatch(/border-radius/)
    }
    // The two classes the card world used are gone from the sheet.
    expect(SHEET).not.toMatch(/\.ucards|\.usage-provider/)
  })

  it('draws a limit name as the quiet sub-heading of the house (R4, D2)', () => {
    const head = declarations('.settings-pane .ulimit-head')
    expect(head).toMatch(/font-size:\s*11px\s*;/)
    expect(head).toMatch(/font-weight:\s*600\s*;/)
    expect(head).toMatch(/text-transform:\s*uppercase\s*;/)
    expect(head).toMatch(/letter-spacing:\s*\.?0?\.05em\s*;/)
    expect(head).toMatch(/color:\s*var\(--faint\)\s*;/)
  })

  it('puts the plan at the right edge of the provider line (D1)', () => {
    expect(declarations('.settings-pane .uprov-head')).toMatch(
      /grid-template-columns:\s*minmax\(0,\s*1fr\)\s+auto\s*;/,
    )
    expect(declarations('.settings-pane .uplan')).toMatch(/text-align:\s*right\s*;/)
  })
})
