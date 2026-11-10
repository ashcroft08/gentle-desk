// The thread's read model: the foreground chat and its messages, taken from the
// same pushed `ViewState` the sidebar reads (S2). Rust owns the state; this
// module only decides what the thread draws:
//
// - no foreground chat means the placeholder, so a push that carries messages
//   for a chat that is already gone draws nothing;
// - the messages are the last ones Rust pushed for that chat, oldest first,
//   untouched: the order, the streaming flag and the model error are Rust's.

import type { MessageView, ViewState_Serialize as ViewState } from '../contract/bindings'
import { createSessionsStore, type SessionsStore } from '../sessions/store'

/** What the thread can draw right now. */
export type ThreadViewState = {
  /** The foreground chat, or `null` while no chat is open. */
  chat: string | null
  /** That chat's messages, oldest first. Empty while no chat is open. */
  messages: MessageView[]
}

export type ThreadView = {
  get(): ThreadViewState
  subscribe(listener: (state: ThreadViewState) => void): () => void
  /** Settles once the first read and the subscription have been attempted. */
  ready: Promise<void>
  dispose(): void
}

export type ThreadViewDeps = { store?: SessionsStore }

/** The one empty message list, so "no chat" never looks like a new answer. */
const NONE: MessageView[] = []

/** The slice of `view` the thread draws. */
export function threadOf(view: ViewState | null): ThreadViewState {
  const chat = view?.foreground_chat ?? null
  if (chat === null) return { chat: null, messages: NONE }
  return { chat, messages: view?.messages ?? NONE }
}

/**
 * Starts the view. The first answer arrives through `subscribe`, so a component
 * that binds to it draws a value from the first render, never `undefined`.
 */
export function createThreadView(deps: ThreadViewDeps = {}): ThreadView {
  const store = deps.store ?? createSessionsStore()
  let state: ThreadViewState = { chat: null, messages: [] }
  const listeners = new Set<(state: ThreadViewState) => void>()

  function emit(next: ThreadViewState): void {
    state = next
    for (const listener of [...listeners]) listener(state)
  }

  const stop = store.subscribe((view) => {
    const next = threadOf(view)
    // A push that leaves the chat and its messages alone changes nothing here:
    // the array is the one Rust sent, so identity is the cheapest answer.
    if (next.chat === state.chat && next.messages === state.messages) return
    emit(next)
  })

  function subscribe(listener: (state: ThreadViewState) => void): () => void {
    listeners.add(listener)
    listener(state)
    return () => {
      listeners.delete(listener)
    }
  }

  function dispose(): void {
    stop()
    store.dispose()
    listeners.clear()
  }

  return { get: () => state, subscribe, ready: store.ready, dispose }
}
