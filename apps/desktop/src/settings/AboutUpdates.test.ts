// The Updates part of the About and updates section (settings-dialog S6): one
// button runs the check the window's startup check runs, and what it answers is
// drawn in place - checking, the version it found, up to date, or why it failed.
// It reports and never installs: the install stays the update notice's own flow.
//
// The check is injected (`checkUpdate`), so no test reaches the network and the
// result lands when the test says so; `App.update.test.ts` pins the automatic
// startup check itself unchanged.
import '../test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte'
import type { ShellView_Serialize as ShellView } from '../contract/bindings'
import { resetTauriMock } from '../test-support/tauri-mock'
import type { CheckResult } from '../update/plugin'
import SettingsDialog from './SettingsDialog.svelte'

/** The words the section draws, quoted here so a change of copy is a change here. */
const ABOUT = 'About and updates'
const VERSION = '0.1.9'
const CHECK_NOW = 'Check now'
const CHECKING = 'Checking for updates...'
const UP_TO_DATE = 'You are up to date.'
const FOUND = 'Version 0.2.0 is available.'
const FAILED = 'The update check failed: no network'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
})

function shell(): ShellView {
  return {
    pi_path: 'C:/Users/someone/.pi/agent',
    choice: 'isolated',
    ask: false,
    launcher_found: true,
    install: { type: 'Idle' },
    restart_needed: false,
  }
}

/** The dialog drawn on its own, with the check the test answers. */
async function draw(checkUpdate: () => Promise<CheckResult>): Promise<HTMLElement> {
  render(SettingsDialog, {
    props: { open: true, theme: 'system' as const, shell: shell(), version: VERSION, checkUpdate },
  })
  const dialog = await screen.findByRole('dialog')
  return within(dialog).getByRole('region', { name: ABOUT })
}

describe('the Updates check', () => {
  it('draws no result until the button is pressed', async () => {
    const checkUpdate = vi.fn(async (): Promise<CheckResult> => ({ kind: 'current' }))
    const about = await draw(checkUpdate)

    expect(within(about).queryByRole('status')).toBeNull()
    expect(checkUpdate).not.toHaveBeenCalled()
  })

  it('runs the check once and moves from checking to the version it found', async () => {
    let answer!: (result: CheckResult) => void
    const checkUpdate = vi.fn(
      () =>
        new Promise<CheckResult>((resolve) => {
          answer = resolve
        }),
    )
    const about = await draw(checkUpdate)

    await fireEvent.click(within(about).getByRole('button', { name: CHECK_NOW }))

    expect(within(about).getByRole('status').textContent).toContain(CHECKING)
    expect(checkUpdate).toHaveBeenCalledTimes(1)
    // A second press while the check runs asks nothing more.
    await fireEvent.click(within(about).getByRole('button', { name: CHECK_NOW }))
    expect(checkUpdate).toHaveBeenCalledTimes(1)

    answer({ kind: 'available', version: '0.2.0' })

    await vi.waitFor(() =>
      expect(within(about).getByRole('status').textContent).toContain(FOUND),
    )
    // It reports; it never installs by itself (S6): the one button stays the check.
    expect(within(about).queryByRole('button', { name: 'Install update' })).toBeNull()
    expect(within(about).getByRole('button', { name: CHECK_NOW })).toBeTruthy()
  })

  it('says it is up to date when the check finds nothing newer', async () => {
    const about = await draw(async () => ({ kind: 'current' }))

    await fireEvent.click(within(about).getByRole('button', { name: CHECK_NOW }))

    await vi.waitFor(() =>
      expect(within(about).getByRole('status').textContent).toContain(UP_TO_DATE),
    )
  })

  it('says why the check failed, and lets the user ask again', async () => {
    const checkUpdate = vi.fn(
      async (): Promise<CheckResult> => ({ kind: 'failed', message: 'no network' }),
    )
    const about = await draw(checkUpdate)

    await fireEvent.click(within(about).getByRole('button', { name: CHECK_NOW }))
    await vi.waitFor(() =>
      expect(within(about).getByRole('status').textContent).toContain(FAILED),
    )

    await fireEvent.click(within(about).getByRole('button', { name: CHECK_NOW }))
    await vi.waitFor(() => expect(checkUpdate).toHaveBeenCalledTimes(2))
  })
})
