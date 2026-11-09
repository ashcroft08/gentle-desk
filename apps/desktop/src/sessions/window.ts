// The windowed read of the session list (S3): the sidebar draws a viewport, and
// this module asks Rust only for the rows around it. The whole list is never
// requested, never held and never serialized here.
//
// One responsibility: turn (query, viewport) into one `sessions_window` call,
// keep the pages it answered, and make sure the newest answer wins. The rules:
//
// - the page is `first - OVERSCAN` for `visible + 2 * OVERSCAN` rows, capped at
//   MAX_WINDOW because Rust caps the answer the same way;
// - pages are cached by page start per query, and the whole cache is dropped
//   when the query changes or when `sessions_revision` changes (the list Rust
//   holds moved, so the rows are stale);
// - a response that resolves after a newer request is ignored, so a slow page
//   never overwrites a fast one;
// - a rejected invoke becomes an error state, never a thrown exception;
// - a query change is debounced (QUERY_DEBOUNCE_MS) through an injectable
//   timer, so a test never sleeps.

import { commands, type SessionRowView, type SessionsWindow } from '../contract/bindings'

/** Rows fetched beyond the viewport on each side. */
export const OVERSCAN = 10

/** The most rows one window asks for; the same cap Rust applies to an answer. */
export const MAX_WINDOW = 200

/** How long a query change waits before it reaches Rust. */
export const QUERY_DEBOUNCE_MS = 150

/** The part of the list a viewport shows: its first row and how many fit. */
export type Viewport = { first: number; visible: number }

/** The rows one request holds: `[offset, offset + length)`. */
export type Page = { offset: number; length: number }

/** What the sidebar can draw right now. */
export type WindowState = {
  query: string
  rows: SessionRowView[]
  total: number
  loading: boolean
  error: string | null
}

export type WindowDeps = {
  /** The invoke call; defaults to the generated `sessions_window` binding. */
  fetch?: (offset: number, length: number, query: string) => Promise<SessionsWindow>
  /** The debounce timer; defaults to `setTimeout`. */
  schedule?: (fn: () => void, ms: number) => unknown
  /** The debounce timer's cancel; defaults to `clearTimeout`. */
  cancel?: (handle: unknown) => void
}

export type WindowFetcher = {
  get(): WindowState
  subscribe(listener: (state: WindowState) => void): () => void
  /** Reads the page the `viewport` needs, debouncing a query change. */
  request(query: string, viewport: Viewport): void
  /** Drops every cached page when Rust's list revision moved. */
  setRevision(revision: number): void
  /** The cached row at `index`, when a page holding it is still cached. */
  rowAt(index: number): SessionRowView | undefined
  dispose(): void
}

/** The rows a viewport asks Rust for, overscanned and capped. */
export function pageFor(viewport: Viewport): Page {
  const first = Math.max(0, Math.trunc(viewport.first) || 0)
  const visible = Math.max(0, Math.trunc(viewport.visible) || 0)
  return {
    offset: Math.max(0, first - OVERSCAN),
    length: Math.min(visible + 2 * OVERSCAN, MAX_WINDOW),
  }
}

const invoke = (offset: number, length: number, query: string) =>
  commands.sessionsWindow(offset, length, query)

export function createWindowFetcher(deps: WindowDeps = {}): WindowFetcher {
  const fetchPage = deps.fetch ?? invoke
  const schedule = deps.schedule ?? ((fn: () => void, ms: number) => setTimeout(fn, ms))
  const cancel = deps.cancel ?? ((handle: unknown) => clearTimeout(handle as number))

  let state: WindowState = { query: '', rows: [], total: 0, loading: false, error: null }
  let pages = new Map<number, SessionRowView[]>()
  let revision: number | null = null
  let pending: { query: string; viewport: Viewport } | null = null
  let timer: unknown = null
  let latest = 0
  const listeners = new Set<(state: WindowState) => void>()

  function emit(next: WindowState): void {
    state = next
    for (const listener of [...listeners]) listener(state)
  }

  function fail(id: number, error: unknown): void {
    if (id !== latest) return
    emit({
      ...state,
      loading: false,
      rows: [],
      error: error instanceof Error ? error.message : String(error),
    })
  }

  function load(query: string, viewport: Viewport): void {
    const { offset, length } = pageFor(viewport)
    const cached = pages.get(offset)
    if (cached !== undefined) {
      emit({ query, rows: cached, total: state.total, loading: false, error: null })
      return
    }
    const id = latest + 1
    latest = id
    emit({ ...state, query, loading: true, error: null })
    let call: Promise<SessionsWindow>
    try {
      call = fetchPage(offset, length, query)
    } catch (error) {
      fail(id, error)
      return
    }
    call.then(
      (page) => {
        // A newer request or a new query owns the state now: this page is old.
        if (id !== latest || query !== state.query) return
        pages.set(offset, page.rows)
        emit({ query, rows: page.rows, total: page.total, loading: false, error: null })
      },
      (error: unknown) => fail(id, error),
    )
  }

  function request(query: string, viewport: Viewport): void {
    if (query !== state.query) {
      pages = new Map()
      pending = { query, viewport }
      emit({ query, rows: [], total: 0, loading: true, error: null })
      if (timer !== null) cancel(timer)
      timer = schedule(() => {
        timer = null
        const next = pending
        pending = null
        if (next !== null) load(next.query, next.viewport)
      }, QUERY_DEBOUNCE_MS)
      return
    }
    load(query, viewport)
  }

  function setRevision(next: number): void {
    if (revision === next) return
    revision = next
    pages = new Map()
  }

  function rowAt(index: number): SessionRowView | undefined {
    for (const [start, rows] of pages) {
      if (index >= start && index < start + rows.length) return rows[index - start]
    }
    return undefined
  }

  function subscribe(listener: (state: WindowState) => void): () => void {
    listeners.add(listener)
    return () => {
      listeners.delete(listener)
    }
  }

  function dispose(): void {
    if (timer !== null) cancel(timer)
    timer = null
    pending = null
    latest += 1
    pages = new Map()
    listeners.clear()
  }

  return { get: () => state, subscribe, request, setRevision, rowAt, dispose }
}
