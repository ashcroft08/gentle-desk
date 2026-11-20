// A WINDOW a test can size (welcome-screen T2's follow-up): the welcome picks the
// rose's size from `window.matchMedia`, so a test says how tall (and how wide) the
// window is and then moves it, and every listener the component registered is
// called as the browser would call it. Only the two queries the view asks are
// understood - `(min-width: Npx)` and `(min-height: Npx)` - and any other query
// (reduced motion, a pointer) answers false, which is what an unremarkable window
// is. Nothing here decides anything: it reports a size and nothing else.
import { vi } from 'vitest'

type Listener = () => void

/** One window's size the queries are answered against. */
type Size = { width: number; height: number }

/** Does one `(min-width|min-height: Npx)` query hold at `size`? */
function matches(query: string, size: Size): boolean {
  const min = /\(min-(width|height):\s*(\d+)px\)/.exec(query)
  if (min === null) return false
  return (min[1] === 'width' ? size.width : size.height) >= Number(min[2])
}

/** A stubbed `matchMedia` window: `resize` moves it and tells every listener. */
export interface MediaWindow {
  /** Moves the window, calling the listener of every list that was handed out. */
  resize(width: number, height: number): void
}

/**
 * Stubs `matchMedia` for a `width` x `height` px window, through `vi.stubGlobal`
 * so the test's cleanup puts the previous one back. The returned `resize` is how a
 * test says the user dragged the window's corner.
 */
export function mediaWindow(width: number, height: number): MediaWindow {
  let size: Size = { width, height }
  const lists: { query: string; listeners: Set<Listener> }[] = []

  const media = (query: string): MediaQueryList => {
    const listeners = new Set<Listener>()
    lists.push({ query, listeners })
    const list = {
      media: query,
      onchange: null,
      addListener: (listener: Listener) => listeners.add(listener),
      removeListener: (listener: Listener) => listeners.delete(listener),
      addEventListener: (type: string, listener: Listener) => {
        if (type === 'change') listeners.add(listener)
      },
      removeEventListener: (type: string, listener: Listener) => {
        if (type === 'change') listeners.delete(listener)
      },
      dispatchEvent: () => false,
    }
    Object.defineProperty(list, 'matches', { get: () => matches(query, size) })
    return list as unknown as MediaQueryList
  }

  vi.stubGlobal('matchMedia', media)

  return {
    resize(nextWidth: number, nextHeight: number): void {
      size = { width: nextWidth, height: nextHeight }
      for (const list of lists) for (const listener of [...list.listeners]) listener()
    },
  }
}
