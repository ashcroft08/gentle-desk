// Whether the desk is the window in front (S1): the one fact the alerting path
// reads to stay silent while the maintainer is already looking. The three facts
// it follows are the browser's own: `focus`, `blur` and `visibilitychange`, so a
// hidden or minimized window counts as not focused even while WebView2 still
// calls it focused.

import { afterEach, describe, expect, it, vi } from 'vitest'
import { windowFocused, type WindowFocus } from './focus.svelte'

/** The store a test opened, so its listeners never outlive the case. */
let open: WindowFocus | null = null

/** What the window says about being in front, before any event arrives. */
function machine(options: { focused: boolean; visible?: boolean }): void {
  const visibility = options.visible === false ? 'hidden' : 'visible'
  vi.spyOn(document, 'hasFocus').mockReturnValue(options.focused)
  Object.defineProperty(document, 'visibilityState', {
    configurable: true,
    get: () => visibility,
  })
  Object.defineProperty(document, 'hidden', {
    configurable: true,
    get: () => visibility === 'hidden',
  })
}

function store(): WindowFocus {
  open = windowFocused()
  return open
}

afterEach(() => {
  open?.dispose()
  open = null
  vi.restoreAllMocks()
  Object.defineProperty(document, 'visibilityState', { configurable: true, get: () => 'visible' })
  Object.defineProperty(document, 'hidden', { configurable: true, get: () => false })
})

describe('the window in front', () => {
  it('starts from what the document says', () => {
    machine({ focused: true })
    expect(store().focused).toBe(true)

    open?.dispose()
    machine({ focused: false })
    expect(store().focused).toBe(false)
  })

  it('starts unfocused when the window is hidden, even if it says it has focus', () => {
    machine({ focused: true, visible: false })
    expect(store().focused).toBe(false)
  })

  it('follows the window losing and gaining focus', () => {
    machine({ focused: true })
    const focus = store()

    window.dispatchEvent(new Event('blur'))
    expect(focus.focused).toBe(false)

    window.dispatchEvent(new Event('focus'))
    expect(focus.focused).toBe(true)
  })

  it('counts a hidden window as not focused and a visible one by its focus again', () => {
    machine({ focused: true })
    const focus = store()

    Object.defineProperty(document, 'visibilityState', { configurable: true, get: () => 'hidden' })
    Object.defineProperty(document, 'hidden', { configurable: true, get: () => true })
    document.dispatchEvent(new Event('visibilitychange'))
    expect(focus.focused).toBe(false)

    Object.defineProperty(document, 'visibilityState', { configurable: true, get: () => 'visible' })
    Object.defineProperty(document, 'hidden', { configurable: true, get: () => false })
    vi.spyOn(document, 'hasFocus').mockReturnValue(true)
    document.dispatchEvent(new Event('visibilitychange'))
    expect(focus.focused).toBe(true)
  })

  it('stops following the window once disposed', () => {
    machine({ focused: true })
    const focus = store()
    focus.dispose()
    open = null

    window.dispatchEvent(new Event('blur'))
    expect(focus.focused).toBe(true)
  })
})
