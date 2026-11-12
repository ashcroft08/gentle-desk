// The updater check the window runs BY ITSELF when it opens, pinned before the
// Settings dialog grew its own Check now button (settings-dialog S6: the
// automatic check is unchanged). It is the same one plugin call the button
// makes - `update/plugin`'s seam - asked here once, with the answer drawn in the
// update line the app has always drawn, and never installed by itself.
//
// The plugin's `check` is the module seam mocked here, so no test reaches the
// network: what it answers is the whole of what the app sees.
import './test-support/dom-env'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import { resetTauriMock, viewState } from './test-support/tauri-mock'
import { serveSidebar } from './test-support/sidebar'
import App from './App.svelte'

const { check } = vi.hoisted(() => ({ check: vi.fn() }))

vi.mock('@tauri-apps/api/core', () => import('./test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('./test-support/tauri-mock').then((m) => m.eventModule()))
vi.mock('@tauri-apps/plugin-updater', () => ({ check }))

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  check.mockReset()
  // `update/plugin` asks the updater only in the app, so the test says it is one
  // (the marker `@tauri-apps/api` leaves inside the webview); no Tauri is built.
  const globals = globalThis as unknown as Record<string, unknown>
  globals.__TAURI_INTERNALS__ = {}
})

afterEach(() => {
  delete (globalThis as unknown as Record<string, unknown>).__TAURI_INTERNALS__
})

/** The update line, or null while it draws nothing: an idle app says nothing. */
function updateLine(): HTMLElement | null {
  return document.querySelector('.update')
}

describe('the update check of a run that opens', () => {
  it('asks the release server once when the window opens', async () => {
    check.mockResolvedValue(null)
    serveSidebar({ view: viewState() })
    render(App)

    await vi.waitFor(() => expect(check).toHaveBeenCalledTimes(1))
    // Nothing newer: the line stays away, and the window says nothing (S2).
    expect(updateLine()).toBeNull()
  })

  it('offers the install of the notice when a newer release exists, without pressing it', async () => {
    const install = vi.fn()
    check.mockResolvedValue({ version: '0.2.0', downloadAndInstall: install })
    serveSidebar({ view: viewState() })
    render(App)

    expect(await screen.findByRole('button', { name: 'Install update' })).toBeTruthy()
    expect(document.body.textContent).toContain('0.2.0')
    // The check only offers: nothing installs until the user presses it.
    expect(install).not.toHaveBeenCalled()
  })

  it('says nothing when the check itself fails', async () => {
    check.mockRejectedValue(new Error('no network'))
    serveSidebar({ view: viewState() })
    render(App)

    await vi.waitFor(() => expect(check).toHaveBeenCalledTimes(1))
    // The rejection is answered by the seam itself, so no line and no error text.
    await Promise.resolve()
    expect(updateLine()).toBeNull()
    expect(document.body.textContent).not.toContain('no network')
  })
})
