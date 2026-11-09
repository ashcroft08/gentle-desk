// The session list slice the web side renders (S2, S3): one first read through
// `get_view_state`, then every push of `state-changed`, and nothing else. Rust
// owns the state, so this store never derives a list of its own: it holds the
// newest `ViewState` it was given, which carries `list_status`
// (`Loading{done,total}` or `Complete`), `total_sessions`, `sessions_revision`
// and the per-chat `activity` rows the sidebar marks.
//
// The only rule here is ordering: a push that lands before the first read
// resolves must not be undone by that read, and a push older than the state
// already held is dropped. Revisions wrap, so "newer" is decided in modular
// arithmetic, not with `<`.

import { commands, events, type ViewState_Serialize as ViewState } from '../contract/bindings'

/**
 * Is `candidate` a newer revision than `current`, over the wrapping counter
 * Rust bumps (`sessions_revision` is a `u32`)? A gap of half the range or more
 * counts as older, so the answer follows the nearer end of the wrap.
 */
export function newerRevision(current: number, candidate: number): boolean {
  const step = (candidate - current) >>> 0
  return step !== 0 && step < 0x8000_0000
}

/** Something to re-render: the state changed, or the store was disposed. */
export type ViewListener = (state: ViewState | null) => void

export type SessionsStore = {
  /** The newest state the store holds, or `null` before the first answer. */
  get(): ViewState | null
  /** Registers a listener and returns the function that removes it again. */
  subscribe(listener: ViewListener): () => void
  /** Settles once the first read and the subscription have been attempted. */
  ready: Promise<void>
  /** Removes the event listener and every local listener, for good. */
  dispose(): void
}

/**
 * Starts the store: it subscribes first, so no push is missed, and then does
 * the one first read (`get_view_state`) that a view starting late needs to see
 * a list that settled before it opened.
 */
export function createSessionsStore(): SessionsStore {
  let current: ViewState | null = null
  let unlisten: (() => void) | null = null
  let disposed = false
  const listeners = new Set<ViewListener>()

  function publish(state: ViewState): void {
    current = state
    for (const listener of [...listeners]) listener(current)
  }

  /** A push is applied unless it is genuinely older than what is already held. */
  function acceptPush(state: ViewState): void {
    if (disposed) return
    if (
      current !== null &&
      state.sessions_revision !== current.sessions_revision &&
      !newerRevision(current.sessions_revision, state.sessions_revision)
    ) {
      return
    }
    publish(state)
  }

  /** The first read only wins over a state that is strictly older than it. */
  function acceptRead(state: ViewState): void {
    if (disposed) return
    if (current !== null && !newerRevision(current.sessions_revision, state.sessions_revision)) {
      return
    }
    publish(state)
  }

  const ready = (async () => {
    try {
      const stop = await events.stateChanged.listen((event) => acceptPush(event.payload))
      if (disposed) {
        stop()
        return
      }
      unlisten = stop
      acceptRead(await commands.getViewState())
    } catch {
      // A failed first read is not fatal and is not worth a UI state of its
      // own: the pushes below still fill the store with the same slice.
    }
  })()

  function subscribe(listener: ViewListener): () => void {
    listeners.add(listener)
    return () => {
      listeners.delete(listener)
    }
  }

  function dispose(): void {
    disposed = true
    listeners.clear()
    unlisten?.()
    unlisten = null
  }

  return { get: () => current, subscribe, ready, dispose }
}
