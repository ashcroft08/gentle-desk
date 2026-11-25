// The General pane (0.9.0 general S2, S5; the approved artboard
// `design/settings-redesign/08-general.html`, sheets G1 and G2): one row per tool
// the app and the shell depend on, its state worded apart from the other, the
// version only when the check read one, and the fix hint ONLY when the tool is
// missing or unreadable. The bottom row is the last check and the one button.
//
// Everything here is drawn from what the pane is HANDED - the environment check
// of the dialog's one read - so this file renders the component with its props
// and never mounts the dialog: what the dialog wires is
// SettingsDialog.snapshot.test.ts's. Nothing in this pane reads the machine, so
// no test here expects a command.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, within } from '@testing-library/svelte'
import type { GeneralView_Serialize as GeneralView } from '../contract/bindings'
import { resetTauriMock } from '../test-support/tauri-mock'
import { setLanguage } from '../i18n/language.svelte'
import { generalCopy } from './general-copy'
import GeneralPane from './GeneralPane.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** Everything installed, the state the artboard's G1 draws. */
function installed(over: Partial<GeneralView> = {}): GeneralView {
  return {
    tools: [
      { tool: 'GentleShell', state: 'Installed', version: '4.9.0' },
      { tool: 'Pi', state: 'Installed', version: '1.6.0' },
      { tool: 'Node', state: 'Installed', version: 'v24.11.1' },
      { tool: 'Git', state: 'Installed', version: '2.51.0' },
    ],
    checked_ms: Date.now() - 3 * 60_000,
    ...over,
  }
}

// The testing library only cleans up on its own with vitest globals on.
afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setLanguage('en')
})

/** The pane drawn with the answer the check gave and the dialog's own signals. */
function draw(
  view: GeneralView | null,
  over: { refreshing?: boolean; onRefresh?: () => void } = {},
): HTMLElement {
  const { container } = render(GeneralPane, {
    props: {
      general: view,
      refreshing: over.refreshing ?? false,
      onRefresh: over.onRefresh ?? (() => {}),
    },
  })
  return container as HTMLElement
}

/** The rows the pane draws, top to bottom. */
function rows(pane: HTMLElement): HTMLElement[] {
  return [...pane.querySelectorAll<HTMLElement>('.tool')]
}

/** One row's own parts, as the artboard's classes name them. */
function name(row: HTMLElement): string {
  return row.querySelector('.tname')?.textContent?.trim() ?? ''
}

function tag(row: HTMLElement): { text: string; tone: string } {
  const found = row.querySelector('.tag')
  const tone = ['ok', 'warn', 'bad'].find((one) => found?.classList.contains(one)) ?? ''
  return { text: found?.textContent?.trim() ?? '', tone }
}

function version(row: HTMLElement): string | null {
  const found = row.querySelector('.tver')
  return found === null ? null : (found.textContent?.trim() ?? '')
}

function why(row: HTMLElement): string | null {
  const found = row.querySelector('.twhy')
  return found === null ? null : (found.textContent?.trim() ?? '')
}

describe('a drawn General pane', () => {
  it('draws one named row per tool, in the order the check reported', () => {
    const pane = draw(installed())

    expect(rows(pane).map(name)).toEqual(['Gentle Shell', 'pi', 'node', 'git'])
    expect(rows(pane).map(tag)).toEqual([
      { text: 'Installed', tone: 'ok' },
      { text: 'Installed', tone: 'ok' },
      { text: 'Installed', tone: 'ok' },
      { text: 'Installed', tone: 'ok' },
    ])
    // The version only when the check read one, and never a hint on a good row.
    expect(rows(pane).map(version)).toEqual(['4.9.0', '1.6.0', 'v24.11.1', '2.51.0'])
    expect(rows(pane).map(why)).toEqual([null, null, null, null])
  })

  // S5: the two facts are drawn with two different words AND two different
  // sentences, and the sentence names the exact command the app knows.
  it('words missing and unreadable apart, each with its install command in code', () => {
    const pane = draw({
      tools: [
        { tool: 'Node', state: 'Missing' },
        { tool: 'Git', state: 'Unreadable' },
      ],
      checked_ms: null,
    })
    const [node, git] = rows(pane)

    expect(tag(node)).toEqual({ text: 'Missing', tone: 'warn' })
    expect(why(node)).toBe(
      'Install it with winget install OpenJS.NodeJS.LTS and check again. The app uses it for the usage script.',
    )
    expect(node.querySelector('.twhy code')?.textContent).toBe('winget install OpenJS.NodeJS.LTS')
    // No version was read, so none is drawn: never an empty line and never a guess.
    expect(version(node)).toBeNull()

    expect(tag(git)).toEqual({ text: 'Could not read', tone: 'bad' })
    // The sentence names the tool in code AND the installer in code: two
    // commands, in the order the artboard's G2 card draws them.
    expect([...git.querySelectorAll('.twhy code')].map((one) => one.textContent)).toEqual([
      'git',
      'winget install Git.Git',
    ])
    // S6: the sentence about git never blocking the app is drawn with its row.
    expect(why(git)).toContain('Without git the app still works: only the branch chips are missing.')
    expect(version(git)).toBeNull()
  })

  // G2: before the check answers - and for a payload that carries no row at all -
  // the pane says it is checking. It never invents an "Installed" row.
  it('says it is checking while it has nothing to show, and never invents a state', () => {
    for (const nothing of [null, { tools: [], checked_ms: null }]) {
      const pane = draw(nothing)
      const drawn = rows(pane)

      expect(drawn).toHaveLength(1)
      expect(name(drawn[0])).toBe('Gentle Shell')
      expect(tag(drawn[0])).toEqual({ text: 'Checking...', tone: 'warn' })
      expect(version(drawn[0])).toBe('—')
      expect(why(drawn[0])).toBeNull()
      // Never "Installed", and never a fix hint it has no fact for.
      expect(pane.textContent).not.toContain('Installed')
      expect(within(pane).getByText('Last checked: never')).toBeTruthy()
      cleanup()
    }
  })

  // The one button of the card: dead while THIS open's read is out (no answer
  // yet) and dead while a load the user asked for is out. A payload that answered
  // with no row at all leaves it alive - no read is in flight, and a press is
  // harmless - so the state is never a dead end.
  it('is dead while a read is out, and alive for an answer that carried no row', () => {
    const waiting = within(draw(null)).getByRole('button', { name: 'Check again' })
    expect((waiting as HTMLButtonElement).disabled).toBe(true)
    cleanup()

    const answered = within(draw({ tools: [], checked_ms: null })).getByRole('button', {
      name: 'Check again',
    })
    expect((answered as HTMLButtonElement).disabled).toBe(false)
  })

  it('words the last check with the house age, and says never before one', () => {
    const now = Date.now()
    expect(draw(installed({ checked_ms: null })).textContent).toContain('Last checked: never')
    expect(draw(installed({ checked_ms: now })).textContent).toContain('Last checked: just now')
    expect(draw(installed({ checked_ms: now - 2 * 3_600_000 })).textContent).toContain(
      'Last checked: 2h ago',
    )
  })

  // S4: the pane has no read of its own. Its one button asks the dialog for the
  // same one load the rest of the card asks for, and it is dead while that load
  // is out, so a burst of clicks cannot become a burst of checks.
  it('asks the dialog for the one load, once per press, and is dead while it is out', async () => {
    const onRefresh = vi.fn()
    const pane = draw(installed(), { onRefresh })
    const button = within(pane).getByRole('button', { name: 'Check again' }) as HTMLButtonElement

    expect(button.disabled).toBe(false)
    await fireEvent.click(button)
    expect(onRefresh).toHaveBeenCalledTimes(1)

    cleanup()
    const busy = draw(installed(), { onRefresh, refreshing: true })
    const dead = within(busy).getByRole('button', { name: 'Check again' }) as HTMLButtonElement
    expect(dead.disabled).toBe(true)
    await fireEvent.click(dead)
    expect(onRefresh).toHaveBeenCalledTimes(1)
  })

  it('keeps the order the check reported, whatever the rows are', () => {
    // The wire carries the rows in the contract's own order and the pane draws
    // them as they arrive: it never re-sorts the check's answer.
    const pane = draw({
      tools: [
        { tool: 'Git', state: 'Installed', version: '2.51.0' },
        { tool: 'Node', state: 'Missing' },
      ],
      checked_ms: null,
    })

    expect(rows(pane).map(name)).toEqual(['git', 'node'])
    expect(generalCopy('en').state.Missing).toBe(tag(rows(pane)[1]).text)
  })
})
