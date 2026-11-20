// The two selects of the composer's row (design/composer-dock, the maintainer's
// comment mf3kk990): each one wears its OWN glyph and a caret, so both read as
// selects instead of a colour dot and a word. The glyph is decoration; the
// accessible name and the popover still carry the meaning.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import type { ModelChipView } from '../contract/bindings'
import { resetTauriMock, setHandler } from '../test-support/tauri-mock'
import Composer from './Composer.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))
vi.mock('@tauri-apps/api/webview', () => import('../test-support/webview').then((m) => m.webviewModule()))

const READY: ModelChipView = {
  status: { type: 'Ready' },
  models: [{ provider: 'prov', model_id: 'm1', name: 'Model One', reasoning: true }],
  current: { provider: 'prov', model_id: 'm1', name: 'Model One', reasoning: true },
  thinking_level: 'high',
  thinking_levels: ['off', 'medium', 'high'],
}

function draw() {
  return render(Composer, { props: { chat: 'c1', chip: READY } })
}

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => Promise.resolve(null))
})

afterEach(cleanup)

describe('the row’s two selects', () => {
  it('wears its own glyph on each select, never the same one twice', () => {
    draw()

    const model = screen.getByRole('button', { name: 'Model' })
    const level = screen.getByRole('button', { name: 'Thinking level' })

    expect(model.querySelector('[data-glyph="model"]')).toBeTruthy()
    expect(level.querySelector('[data-glyph="effort"]')).toBeTruthy()
    expect(model.querySelector('[data-glyph="effort"]')).toBeNull()
  })

  it('draws a caret on both, so both read as selects', () => {
    draw()

    for (const name of ['Model', 'Thinking level']) {
      const pill = screen.getByRole('button', { name })
      // The caret is the app's own icon; the glyph is the select's own mark.
      expect(pill.querySelector('svg.icon')).toBeTruthy()
      expect(pill.getAttribute('aria-haspopup')).toBe('listbox')
    }
  })
})
