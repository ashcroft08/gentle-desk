// The two mappings of the drop boundary, as plain data: a drag tells the
// composer it can take the files, and a drop hands over the PATHS. Choosing here
// keeps the rule testable without a window, a webview or a file.

import { describe, expect, it } from 'vitest'
import { dropAction, watchDrops } from './drop'

describe('dropAction', () => {
  it('says a drag can be taken while files are over the window', () => {
    expect(dropAction({ type: 'enter', paths: ['C:\\shots\\a.png'] })).toEqual({ kind: 'over' })
    expect(dropAction({ type: 'over' })).toEqual({ kind: 'over' })
  })

  it('hands over the paths of a drop, and nothing else', () => {
    // The list is what Rust reads: no bytes, no names, no File objects.
    expect(dropAction({ type: 'drop', paths: ['C:\\shots\\a.png', 'C:\\shots\\b.png'] })).toEqual({
      kind: 'drop',
      paths: ['C:\\shots\\a.png', 'C:\\shots\\b.png'],
    })
  })

  it('clears the hint when the drag leaves or a drop names no file', () => {
    expect(dropAction({ type: 'leave' })).toEqual({ kind: 'clear' })
    expect(dropAction({ type: 'drop' })).toEqual({ kind: 'drop', paths: [] })
  })
})

describe('watchDrops', () => {
  it('is a no-op where there is no Tauri to listen to', async () => {
    // A plain browser (`npm run dev`) and every test that renders the composer
    // without a webview reach this line: it must answer a working unsubscribe
    // instead of throwing, or the composer cannot even be drawn.
    const globals = globalThis as unknown as Record<string, unknown>
    const saved = globals.__TAURI_INTERNALS__
    delete globals.__TAURI_INTERNALS__
    try {
      const stop = await watchDrops(() => {
        throw new Error('no event may arrive without Tauri')
      })
      expect(typeof stop).toBe('function')
      expect(() => stop()).not.toThrow()
    } finally {
      if (saved !== undefined) globals.__TAURI_INTERNALS__ = saved
    }
  })
})
