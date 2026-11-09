// The sidebar's read model: the pushed state and the page of tree rows the
// viewport needs, kept together so one view draws one consistent tree.
//
// Two rules live here, and nowhere in a component:
//
// - the page is asked for by the viewport (`sessions_window` through
//   `window.ts`), and the first row of the page is remembered so the tree can
//   measure the spacer above the rows it draws;
// - a push that moves `sessions_revision` means the tree Rust holds changed, so
//   the cached pages are dropped and the SAME viewport is read again. A push
//   that leaves the revision alone changes no row, and reads nothing.

import type { ViewState_Serialize as ViewState } from '../contract/bindings'
import type { TreeRow } from './rows'
import { createSessionsStore, type SessionsStore } from './store'
import { createWindowFetcher, pageFor, type Viewport, type WindowFetcher } from './window'

/** What the sidebar can draw right now. */
export type SessionsViewState = {
  /** The newest pushed state, or `null` before the first one arrives. */
  view: ViewState | null
  /** The rows of the served page, in the order Rust flattened them. */
  rows: TreeRow[]
  /** Index of the first row of the page: the top spacer measures from it. */
  first: number
  /** How many rows the whole tree has, headers included. */
  total: number
  /** Whether a page is being read. Never a reason to call the tree empty. */
  loading: boolean
  error: string | null
}

export type SessionsView = {
  get(): SessionsViewState
  subscribe(listener: (state: SessionsViewState) => void): () => void
  /** Asks for the page the viewport needs; remembered for the revision re-read. */
  show(viewport: Viewport): void
  /** Settles once the first read and the subscription have been attempted. */
  ready: Promise<void>
  dispose(): void
}

export type SessionsViewDeps = {
  store?: SessionsStore
  fetcher?: WindowFetcher
}

/**
 * Starts the view. `loading` begins true, so a sidebar mounted before the first
 * answer draws its reading line instead of claiming the list is empty.
 */
export function createSessionsView(deps: SessionsViewDeps = {}): SessionsView {
  const store = deps.store ?? createSessionsStore()
  const fetcher = deps.fetcher ?? createWindowFetcher()

  let state: SessionsViewState = {
    view: null,
    rows: [],
    first: 0,
    total: 0,
    loading: true,
    error: null,
  }
  let viewport: Viewport | null = null
  let revision: number | null = null
  /** The page offset of the newest request, applied once its rows arrive. */
  let requestedFirst = 0
  const listeners = new Set<(state: SessionsViewState) => void>()

  function emit(next: SessionsViewState): void {
    state = next
    for (const listener of [...listeners]) listener(state)
  }

  const stopPages = fetcher.subscribe((page) => {
    // `first` only follows the rows: while a page is in flight the rows still
    // on screen keep their own offset, so nothing jumps between the two.
    emit({
      ...state,
      rows: page.rows,
      total: page.total,
      loading: page.loading,
      error: page.error,
      first: page.loading ? state.first : requestedFirst,
    })
  })

  function read(viewportToRead: Viewport): void {
    requestedFirst = pageFor(viewportToRead).offset
    fetcher.request(viewportToRead)
  }

  const stopState = store.subscribe((view) => {
    if (view === null) {
      emit({ ...state, view: null })
      return
    }
    const moved = revision !== null && view.sessions_revision !== revision
    revision = view.sessions_revision
    emit({ ...state, view })
    if (moved && viewport !== null) {
      // The tree Rust holds moved: the held pages are stale.
      fetcher.setRevision(view.sessions_revision)
      read(viewport)
    }
  })

  function show(next: Viewport): void {
    viewport = { first: next.first, visible: next.visible }
    read(viewport)
  }

  function subscribe(listener: (state: SessionsViewState) => void): () => void {
    listeners.add(listener)
    // The store contract: a subscriber is answered with the current state at
    // once, so `$view` (and any other reader) starts from a value, not undefined.
    listener(state)
    return () => {
      listeners.delete(listener)
    }
  }

  function dispose(): void {
    stopState()
    stopPages()
    store.dispose()
    fetcher.dispose()
    listeners.clear()
  }

  return { get: () => state, subscribe, show, ready: store.ready, dispose }
}
