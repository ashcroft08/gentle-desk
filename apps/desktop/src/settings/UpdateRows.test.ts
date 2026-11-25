// The Updates card's three rows (0.8.1 settings-polish S6, S7, S8, D9, D10, D14),
// drawn from the rules `cardOf` works out: one piece each, the version it runs and
// the one published when there is something newer, its OWN action, and the one
// sentence a failed read or a skip is said with. No test here reaches a registry:
// the rows arrive from the check the dialog's one load made.
import '../test-support/dom-env'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, within } from '@testing-library/svelte'
import type {
  HomeChoiceView,
  UpdatesView_Serialize as UpdatesView,
} from '../contract/bindings'
import UpdateRows from './UpdateRows.svelte'
import { cardOf, type RowAction } from './updates'
import { updatesCopy } from './updates-copy'

const copy = updatesCopy('en')

afterEach(cleanup)

/** The check the user's machine answers: pi and Gentle Shell both behind. */
function updates(over: Partial<UpdatesView> = {}): UpdatesView {
  return {
    status: 'Ready',
    desk: { current: '0.1.12', latest: '0.1.12', update_available: false },
    pi: { current: '1.5.2', latest: '2.0.5', update_available: true },
    shell: { current: '4.8.0', latest: '4.9.2', update_available: true },

    packages: { available: false, behind: [], unknown: false, truncated: false, checked: null },
    ...over,
  }
}

/** One draw of the rows of a check, with the clicks it can receive recorded. */
function draw(
  view: UpdatesView = updates(),
  blocked = false,
  home: HomeChoiceView | null = null,
  shellLatest: string | null = null,
) {
  const onAction = vi.fn<(action: RowAction) => void>()
  const onSkip = vi.fn<(version: string) => void>()
  const onUnskip = vi.fn<() => void>()
  const { container } = render(UpdateRows, {
    props: {
      rows: cardOf(view, copy).rows,
      copy,
      blocked,
      hintId: 'updates-gate',
      home,
      shellLatest,
      onAction,
      onSkip,
      onUnskip,
    },
  })
  return { pane: container as HTMLElement, onAction, onSkip, onUnskip }
}

/** One row's element, by the key it is drawn under. */
function row(pane: HTMLElement, key: 'app' | 'pi' | 'shell'): HTMLElement {
  const found = pane.querySelector(`[data-row="${key}"]`)
  if (!(found instanceof HTMLElement)) throw new Error(`no ${key} row was drawn`)
  return found
}

describe('the Updates rows', () => {
  it('draws each piece with its version and its own action, and names the shell in full', () => {
    const { pane } = draw()

    expect([...pane.querySelectorAll('.urows > .urow')]).toHaveLength(3)
    // D10: the row is the product's name, not the terminal's short one.
    expect(row(pane, 'shell').querySelector('.headline')?.textContent).toBe('Gentle Shell')
    expect(row(pane, 'app').querySelector('.mono')?.textContent).toBe('0.1.12')
    expect(row(pane, 'pi').querySelector('.mono')?.textContent).toBe('1.5.2 \u2192 2.0.5')
    expect(row(pane, 'shell').querySelector('.mono')?.textContent).toBe('4.8.0 \u2192 4.9.2')

    // The app's own release is the banner's job: its row offers no button.
    expect(within(row(pane, 'app')).queryByRole('button')).toBeNull()
    expect(within(row(pane, 'pi')).getByRole('button', { name: copy.update })).toBeTruthy()
    // S7: the shell's button names the product and runs its own upgrade.
    expect(within(row(pane, 'shell')).getByRole('button', { name: copy.updateShell })).toBeTruthy()
  })

  it('asks for the action of the row that was pressed, never one for the whole card', async () => {
    const { pane, onAction } = draw()

    await fireEvent.click(within(row(pane, 'shell')).getByRole('button', { name: copy.updateShell }))
    expect(onAction).toHaveBeenCalledWith('upgrade')

    await fireEvent.click(within(row(pane, 'pi')).getByRole('button', { name: copy.update }))
    expect(onAction).toHaveBeenLastCalledWith('update')
  })

  it('draws the sentence of the row that could not be read, and nowhere else', () => {
    const { pane } = draw(
      updates({
        pi: { current: '2.0.5', latest: '2.0.5', update_available: false },
        shell: {
          current: '4.8.0',
          latest: null,
          update_available: null,
          reason: 'NoAnswer',
        },
      }),
    )

    expect(pane.querySelectorAll('.reason')).toHaveLength(1)
    expect(row(pane, 'shell').querySelector('.reason')?.textContent).toBe(
      'Could not check the latest gentle-shell version on npm: the registry did not answer.',
    )
    expect(row(pane, 'shell').querySelector('.mono')?.textContent).toBe('4.8.0')
  })

  it('carries the one skip on the row it belongs to, and the way back on the skipped one', async () => {
    const { pane, onSkip } = draw()

    // pi 2.0.5 and Gentle Shell 4.9.2 are behind: 4.9.2 is the greatest, so the
    // skip sits on the shell's row (D9).
    expect(within(row(pane, 'app')).queryByRole('button', { name: copy.skipVersion })).toBeNull()
    expect(within(row(pane, 'pi')).queryByRole('button', { name: copy.skipVersion })).toBeNull()
    const skip = within(row(pane, 'shell')).getByRole('button', { name: copy.skipVersion })
    // The cheap one wears no accent: the terminal buttons are the loud ones.
    expect(skip.classList.contains('accent')).toBe(false)
    await fireEvent.click(skip)
    expect(onSkip).toHaveBeenCalledWith('4.9.2')

    const skipped = draw(
      updates({
        pi: { current: '2.0.5', latest: '2.0.5', update_available: false },
        skipped: '4.9.2',
      }),
    )
    const stop = within(row(skipped.pane, 'shell')).getByRole('button', {
      name: copy.stopSkipping,
    })
    await fireEvent.click(stop)
    expect(skipped.onUnskip).toHaveBeenCalledTimes(1)
    expect(skipped.onSkip).not.toHaveBeenCalled()
    // What the skip means, on the row that carries it. The row also names the
    // command its button runs (updates-by-home S4), so the sentence is the `.reason`
    // that is not that line.
    expect(row(skipped.pane, 'shell').querySelector('.reason:not([data-command])')?.textContent).toBe(
      copy.skippedNote('4.9.2'),
    )
  })

  it('keeps every button dead while a chat is open, and points at the reason', () => {
    const { pane, onAction, onSkip } = draw(updates(), true)

    const buttons = [...pane.querySelectorAll('button')] as HTMLButtonElement[]
    expect(buttons).toHaveLength(3)
    for (const button of buttons) {
      expect(button.disabled).toBe(true)
      expect(button.getAttribute('aria-describedby')).toBe('updates-gate')
    }
    expect(onAction).not.toHaveBeenCalled()
    expect(onSkip).not.toHaveBeenCalled()
  })

  it('draws the command of each action it offers, under it, for the home it was given', () => {
    // updates-by-home S4: the row says the exact line its button runs, so the home
    // choice is never a surprise. These are Rust's table's literals, byte for byte.
    const separado = draw()
    expect(row(separado.pane, 'pi').querySelector('[data-command]')?.textContent).toBe(
      'gentle-shell update',
    )
    expect(row(separado.pane, 'shell').querySelector('[data-command]')?.textContent).toBe(
      'gentle-shell upgrade',
    )
    // The app's own release installs itself from the banner: no button, no line.
    expect(row(separado.pane, 'app').querySelector('[data-command]')).toBeNull()

    const miPi = draw(updates(), false, 'link', '4.9.2')
    expect(row(miPi.pane, 'pi').querySelector('[data-command]')?.textContent).toBe('pi update')
    expect(row(miPi.pane, 'shell').querySelector('[data-command]')?.textContent).toBe(
      'pi install npm:gentle-pi@4.9.2',
    )
  })

  it('says nothing is installed when Mi pi has no measured stable version', () => {
    // S2's rule, said before the press: no guessed `@latest`, and not the other
    // home's upgrade either.
    const { pane } = draw(updates(), false, 'link', '4.9.0-main.abc123')
    const shell = row(pane, 'shell')

    expect(shell.querySelector('[data-command]')?.textContent).toBe(copy.noStableLatest)
    expect(shell.textContent).not.toContain('pi install')
    expect(shell.textContent).not.toContain('gentle-shell upgrade')
  })
})
