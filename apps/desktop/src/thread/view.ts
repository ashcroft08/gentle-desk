// The thread's read model: the foreground chat and its transcript, taken from the
// same pushed `ViewState` the sidebar reads (S2). Rust owns the state; this
// module only decides what the thread draws:
//
// - no foreground chat means the placeholder, so a push that carries rows for a
//   chat that is already gone draws nothing;
// - the rows are the last ones Rust pushed for that chat, oldest first, untouched:
//   the order, the kind, the streaming flag, the payloads and the two loss marks
//   are Rust's (chat-transcript T2b). A text row of the list is the same row the
//   `messages` slice carries, so the thread draws it from whichever arrived;
// - `provisional` says those rows were read from the chat's session FILE while its
//   child was starting (chat-open-latency T1). It is Rust's flag: only the shell
//   knows whether the rows are the child's answer or the file's, and the thread
//   only draws what the flag says it may.

import type {
  ChatItemView_Serialize as ChatItem,
  MessageView,
  ViewState_Serialize as ViewState,
} from '../contract/bindings'
import { createSessionsStore, type SessionsStore } from '../sessions/store'

/** What the thread can draw right now. */
export type ThreadViewState = {
  /** The foreground chat, or `null` while no chat is open. */
  chat: string | null
  /** That chat's messages, oldest first. Empty while no chat is open. */
  messages: MessageView[]
  /** That chat's whole transcript, oldest row first: text, thinking and tools. */
  items: ChatItem[]
  /** Whether those rows came from the session file, not from the child (T1). */
  provisional: boolean
}

export type ThreadView = {
  get(): ThreadViewState
  subscribe(listener: (state: ThreadViewState) => void): () => void
  /** Settles once the first read and the subscription have been attempted. */
  ready: Promise<void>
  dispose(): void
}

export type ThreadViewDeps = { store?: SessionsStore }

/** The one empty list, so "no chat" never looks like a new answer. */
const NONE: MessageView[] = []

/** The one empty transcript, for the same reason. */
const NO_ROWS: ChatItem[] = []

/** The slice of `view` the thread draws. */
export function threadOf(view: ViewState | null): ThreadViewState {
  const chat = view?.foreground_chat ?? null
  if (chat === null) return { chat: null, messages: NONE, items: NO_ROWS, provisional: false }
  return {
    chat,
    messages: view?.messages ?? NONE,
    items: view?.items ?? NO_ROWS,
    provisional: view?.transcript_provisional === true,
  }
}

/**
 * Starts the view. The first answer arrives through `subscribe`, so a component
 * that binds to it draws a value from the first render, never `undefined`.
 */
export function createThreadView(deps: ThreadViewDeps = {}): ThreadView {
  const store = deps.store ?? createSessionsStore()
  let state: ThreadViewState = { chat: null, messages: [], items: [], provisional: false }
  const listeners = new Set<(state: ThreadViewState) => void>()

  function emit(next: ThreadViewState): void {
    state = next
    for (const listener of [...listeners]) listener(state)
  }

  const stop = store.subscribe((view) => {
    const next = threadOf(view)
    // A push that leaves the chat and both lists alone changes nothing here: the
    // arrays are the ones Rust sent, so identity is the cheapest answer.
    if (
      next.chat === state.chat &&
      next.messages === state.messages &&
      next.items === state.items &&
      next.provisional === state.provisional
    )
      return
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
