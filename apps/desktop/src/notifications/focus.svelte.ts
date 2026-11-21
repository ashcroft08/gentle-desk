// Whether this window is the one in front (S1): the single fact the alerting
// path reads to stay silent while the maintainer is already looking. The
// browser's own events are the only input - `focus`, `blur` and
// `visibilitychange` - and a hidden or minimized window counts as NOT focused
// even when WebView2 still calls it focused, so a minimized desk never tones.
//
// One store per owner: `dispose()` removes every listener, so a component that
// unmounts leaves nothing behind on `window` or `document`.

/** Whether the window is in front right now, read reactively, plus teardown. */
export type WindowFocus = {
  /** `true` only while the window is visible and holds focus. */
  readonly focused: boolean
  /** Removes the listeners; the value it holds stops changing after this. */
  dispose(): void
}

/** What the machine says right now, before any event arrives. */
function reading(): boolean {
  return !document.hidden && document.hasFocus()
}

/**
 * Starts following the window. The starting value is the machine's own answer,
 * so a store opened while the app is already in front is right from the first
 * read and needs no event to become so.
 */
export function windowFocused(): WindowFocus {
  let focused = $state(reading())

  // Gaining focus never counts while the window is hidden.
  const onFocus = (): void => {
    focused = !document.hidden
  }
  const onBlur = (): void => {
    focused = false
  }
  // Hiding drops focus; showing again asks the document, which is where the
  // truth is once the window is back (a restored window may or may not hold it).
  const onVisibility = (): void => {
    focused = reading()
  }

  window.addEventListener('focus', onFocus)
  window.addEventListener('blur', onBlur)
  document.addEventListener('visibilitychange', onVisibility)

  return {
    get focused(): boolean {
      return focused
    },
    dispose(): void {
      window.removeEventListener('focus', onFocus)
      window.removeEventListener('blur', onBlur)
      document.removeEventListener('visibilitychange', onVisibility)
    },
  }
}
