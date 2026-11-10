// The open chat's chrome, taken from the same pushed `ViewState` the sidebar and
// the thread read (S2). Rust owns the state; this module only picks the three
// slices the main area draws around the thread:
//
// - the header's title, which is Rust's `foreground_title` (the stored
//   session's own title, "New chat" for a chat with no session file yet);
// - the mark the foreground chat's activity calls for, which is what decides
//   Send, Stop and the disabled composer's reason (S6: the rule itself lives in
//   Rust, `crates/runtime`, and arrives here as plain data);
// - the notices, unchanged and in the order Rust keeps them, so only the ones
//   that carry an action are drawn as a restart notice.
//
// No chat open draws no chrome at all: a push that carries a title or a notice
// for a chat that is already gone must not leave a header behind.

import type {
  ChatActivityMark,
  NoticeView,
  ViewState_Serialize as ViewState,
} from '../contract/bindings'
import { createSessionsStore, type SessionsStore } from '../sessions/store'

/** What the main area can draw around the thread right now. */
export type ChatViewState = {
  /** The foreground chat, or `null` while no chat is open. */
  chat: string | null
  /** The header's title, or `null` while no chat is open. */
  title: string | null
  /** That chat's mark, or `null` when it is idle (or no chat is open). */
  mark: ChatActivityMark | null
  /** Every notice Rust pushes, oldest first; most carry no action (T7 draws those). */
  notices: NoticeView[]
}

export type ChatView = {
  get(): ChatViewState
  subscribe(listener: (state: ChatViewState) => void): () => void
  /** Settles once the first read and the subscription have been attempted. */
  ready: Promise<void>
  dispose(): void
}

export type ChatViewDeps = { store?: SessionsStore }

/** The one empty notice list, so "no notices" never looks like a new push. */
const NO_NOTICES: NoticeView[] = []

/** The slice of `view` the main area's chrome draws. */
export function chatOf(view: ViewState | null): ChatViewState {
  const chat = view?.foreground_chat ?? null
  if (chat === null) {
    return { chat: null, title: null, mark: null, notices: NO_NOTICES }
  }
  return {
    chat,
    title: view?.foreground_title ?? null,
    mark: markOf(view, chat),
    notices: view?.notices ?? NO_NOTICES,
  }
}

/**
 * The mark of one chat: only the chats that are not idle are listed, so a chat
 * the list does not name is idle. An activity name the contract does not carry
 * draws nothing rather than a wrong mark.
 */
function markOf(view: ViewState | null, chat: string): ChatActivityMark | null {
  return view?.activity.find((entry) => entry.chat_id === chat)?.activity ?? null
}

/**
 * Starts the view. The first answer arrives through `subscribe`, so a component
 * that binds to it draws a value from the first render, never `undefined`.
 */
export function createChatView(deps: ChatViewDeps = {}): ChatView {
  const store = deps.store ?? createSessionsStore()
  let state: ChatViewState = { chat: null, title: null, mark: null, notices: [] }
  const listeners = new Set<(state: ChatViewState) => void>()

  function emit(next: ChatViewState): void {
    state = next
    for (const listener of [...listeners]) listener(state)
  }

  const stop = store.subscribe((view) => {
    const next = chatOf(view)
    // A push that moves none of these slices changes nothing here, and the
    // arrays are the ones Rust sent, so identity is the cheapest answer.
    if (
      next.chat === state.chat &&
      next.title === state.title &&
      next.mark === state.mark &&
      next.notices === state.notices
    ) {
      return
    }
    emit(next)
  })

  function subscribe(listener: (state: ChatViewState) => void): () => void {
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
