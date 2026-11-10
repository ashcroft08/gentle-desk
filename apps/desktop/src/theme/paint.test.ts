// The page's theme (01a's footer picker and the states of `theme.css`): the
// page PAINTS the theme Rust owns, and it paints it before the app mounts, so a
// window whose config says `light` never shows the dark default first.
import '../test-support/dom-env'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { pushEvent, resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import { paintTheme, startTheme } from './paint'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

/** The attribute the six CSS blocks select on. */
function rootTheme(): string | undefined {
  return document.documentElement.dataset.theme
}

beforeEach(() => {
  resetTauriMock()
  document.documentElement.dataset.theme = 'system'
})

describe('the page theme', () => {
  it('paints a theme on the root element', () => {
    paintTheme('Gentleman-Cute')

    expect(rootTheme()).toBe('Gentleman-Cute')
  })

  it('paints the stored theme from the first read, before anything is drawn', async () => {
    setHandler('get_view_state', () => viewState({ theme: 'light' }))

    await startTheme()

    expect(rootTheme()).toBe('light')
  })

  it('paints every pushed theme', async () => {
    setHandler('get_view_state', () => viewState({ theme: 'light' }))
    await startTheme()

    pushEvent('state-changed', viewState({ theme: 'Gentleman-Sexy' }))

    expect(rootTheme()).toBe('Gentleman-Sexy')
  })

  it('keeps the default theme when the first read fails', async () => {
    // No handler for `get_view_state`: the command rejects, as a bridge that
    // could not answer does.
    await startTheme()

    expect(rootTheme()).toBe('system')
  })

  it('stores nothing of its own: no copy survives a restart', async () => {
    setHandler('get_view_state', () => viewState({ theme: 'Gentle' }))

    // The spy sits on jsdom's own `Storage`: the runner's global `localStorage`
    // differs between Node versions, so the test never reads that global.
    const write = vi.spyOn(window.Storage.prototype, 'setItem')

    await startTheme()

    expect(write).not.toHaveBeenCalled()
    write.mockRestore()
  })
})
