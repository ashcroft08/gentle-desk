// The Extensions block of the Updates pane (packages-status T2, S1, S3; T6, S6):
// the one group no version of this app drives, drawn from pi's own check. Its
// states are five, and none of them is a guess: the list pi reported as behind,
// the listing nobody has checked yet, the answered listing with nothing behind,
// the read that could not run, and the one nothing asked. The block's words are
// `updates-copy.test.ts`'s and its place in the pane is `UpdatesPane.test.ts`'s.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, within } from '@testing-library/svelte'
import type { PackagesView_Serialize as PackagesView } from '../contract/bindings'
import { resetTauriMock } from '../test-support/tauri-mock'
import ExtensionsBlock from './ExtensionsBlock.svelte'
import { updatesCopy } from './updates-copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The block's own words in English, read through the dictionary (0.2.0, D4). */
const copy = updatesCopy('en')

/** The Extensions block's slice: unasked unless a test says otherwise. */
function packages(over: Partial<PackagesView> = {}): PackagesView {
  return {
    available: false,
    behind: [],
    unknown: false,
    truncated: false,
    unchecked: false,
    checked: null,
    ...over,
  }
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

/** The block, over one slice, with everything a test does not name at its usual value. */
function draw(
  packagesView: PackagesView | null = packages(),
  over: { checking?: boolean; blocked?: boolean; error?: string | null } = {},
): HTMLElement {
  const { container } = render(ExtensionsBlock, {
    props: {
      packages: packagesView,
      checking: over.checking ?? false,
      blocked: over.blocked ?? false,
      hintId: 'updates-gate',
      line: 'pi update --extensions',
      error: over.error ?? null,
      onExtensions: () => {},
      copy,
    },
  })
  return container as HTMLElement
}

/** The quiet rows of behind packages the block drew. */
function rows(block: HTMLElement): HTMLElement[] {
  return [...block.querySelectorAll('.urow')] as HTMLElement[]
}

describe('the Extensions block', () => {
  it('counts and names every extension pi reports as behind, each on its own row', () => {
    const block = draw(
      packages({
        available: true,
        behind: [{ name: 'pi-hud', current: '0.3.1' }, { name: 'pi-tools' }],
      }),
    )

    expect(within(block).getByText(copy.packagesBehind(2))).toBeTruthy()
    expect(rows(block).map((each) => each.querySelector('.headline')?.textContent)).toEqual([
      'pi-hud',
      'pi-tools',
    ])
    expect(rows(block).map((each) => each.querySelector('.mono')?.textContent ?? null)).toEqual([
      '0.3.1',
      null,
    ])
    // A finished check has nothing more to say, and no row offers a button.
    expect(block.textContent).not.toContain(copy.packagesUnchecked)
    expect(block.querySelectorAll('.urow button')).toHaveLength(0)
  })

  it('says the extensions have not been checked yet when the listing exists but pi has not run', () => {
    // S6, T6: the defect this state fixes - `available: true` with nothing behind
    // used to read "Las extensiones están al día" although pi had checked nothing.
    const block = draw(packages({ available: true, unchecked: true }))

    expect(within(block).getByText(copy.packagesUnchecked)).toBeTruthy()
    expect(rows(block)).toHaveLength(0)
    expect(block.textContent).not.toContain(copy.packagesUpToDate)
    // Nothing failed, so it is not the failed read's clause either.
    expect(block.textContent).not.toContain(copy.packagesUnreadable)
    expect(block.textContent).not.toContain(copy.extensionsNote)
  })

  it('draws the behind list AND the line below it when only some are checked', () => {
    // One package is behind and another is still `unknown`: the list can never
    // read as the whole story, so the same line sits under it.
    const block = draw(
      packages({
        available: true,
        unchecked: true,
        behind: [{ name: 'pi-hud', current: '0.3.1' }],
      }),
    )
    const list = rows(block)
    const line = within(block).getByText(copy.packagesUnchecked)

    expect(within(block).getByText(copy.packagesBehind(1))).toBeTruthy()
    expect(list.map((each) => each.querySelector('.headline')?.textContent)).toEqual(['pi-hud'])
    // Below the list, in the pane's own quiet words.
    expect(list[0].compareDocumentPosition(line) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
  })

  it('says the extensions are up to date only for the listing pi checked', () => {
    const block = draw(packages({ available: true, unchecked: false }))

    expect(within(block).getByText(copy.packagesUpToDate)).toBeTruthy()
    expect(block.textContent).not.toContain(copy.packagesUnchecked)
    expect(block.textContent).not.toContain(copy.extensionsNote)
  })

  it('keeps the two honest exceptions: a read that could not run, and one nothing asked', () => {
    const unreadable = draw(packages({ unknown: true }))
    expect(within(unreadable).getByText(copy.packagesUnreadable)).toBeTruthy()
    expect(unreadable.textContent).not.toContain(copy.packagesUnchecked)
    expect(rows(unreadable)).toHaveLength(0)

    const unasked = draw(packages())
    expect(unasked.textContent).toContain(copy.extensionsNote)
    expect(within(unasked).getByText(copy.packagesNeedsSession)).toBeTruthy()
    expect(unasked.textContent).not.toContain(copy.packagesUnchecked)
    expect(rows(unasked)).toHaveLength(0)
  })

  it("drops the state line while the check it asked for is out, and keeps the button", () => {
    // S7: one press asks the snapshot load AND pi's own check; until the pushed
    // answer lands the block says it is checking, in the pane's own words.
    const block = draw(packages({ available: true, unchecked: true }), { checking: true })

    expect(within(block).getByText(copy.checking)).toBeTruthy()
    expect(within(block).queryByText(copy.packagesUnchecked)).toBeNull()
    expect(within(block).queryByText(copy.packagesUpToDate)).toBeNull()
    expect(within(block).getByRole('button', { name: copy.updateExtensions })).toBeTruthy()
  })

  it('keeps Update extensions as it was: the one action, its command and its refusal', async () => {
    const pressed: string[] = []
    const { container } = render(ExtensionsBlock, {
      props: {
        packages: packages({ available: true, unchecked: true }),
        checking: false,
        blocked: false,
        hintId: 'updates-gate',
        line: 'pi update --extensions',
        error: 'the extensions window could not open',
        onExtensions: () => pressed.push('extensions'),
        copy,
      },
    })
    const block = container as HTMLElement
    const button = within(block).getByRole('button', { name: copy.updateExtensions })

    expect((button as HTMLButtonElement).disabled).toBe(false)
    await fireEvent.click(button)
    expect(pressed).toEqual(['extensions'])
    // The exact line the button runs, and the terminal's own refusal under it.
    expect(block.querySelector('[data-command]')?.textContent).toBe('pi update --extensions')
    expect(within(block).getByRole('status').textContent).toBe(
      'the extensions window could not open',
    )
  })

  it('keeps the chats-closed gate on that one button, with the reason it points at', () => {
    const block = draw(packages({ available: true, unchecked: true }), { blocked: true })
    const button = within(block).getByRole('button', {
      name: copy.updateExtensions,
    }) as HTMLButtonElement

    expect(button.disabled).toBe(true)
    expect(button.getAttribute('aria-describedby')).toBe('updates-gate')
  })
})
