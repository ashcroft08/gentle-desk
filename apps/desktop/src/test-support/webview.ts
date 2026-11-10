// The stand-in for the THIRD Tauri module the web side touches:
// `@tauri-apps/api/webview`, whose drag and drop event is the only place a
// dropped file's PATH exists. It keeps the listener set the way the real webview
// does - one callback per `onDragDropEvent`, an unlisten per registration - so a
// test can drop files the way the window would and prove what the page does with
// the paths, and that a component that went away listens no more.

/** One event as the webview delivers it. */
type DragListener = (event: { payload: unknown }) => void

const listeners = new Set<DragListener>()

/** Emits one drag and drop payload to every registered listener, in order. */
export function dragEvent(payload: unknown): void {
  for (const listener of [...listeners]) listener({ payload })
}

/** How many components are listening for drops right now. */
export function dragListenerCount(): number {
  return listeners.size
}

/** Forgets every drag listener: part of `resetTauriMock`. */
export function resetDrags(): void {
  listeners.clear()
}

/** The `@tauri-apps/api/webview` module shape a test mocks in. */
export function webviewModule(): {
  getCurrentWebview: () => {
    onDragDropEvent: (callback: DragListener) => Promise<() => void>
  }
} {
  return {
    getCurrentWebview: () => ({
      onDragDropEvent: (callback) => {
        listeners.add(callback)
        return Promise.resolve(() => {
          listeners.delete(callback)
        })
      },
    }),
  }
}
