// The open chat's chrome, taken from the same pushed `ViewState` the sidebar and
// the thread read (S2). Rust owns the state; this module only picks the three
// slices the main area draws around the thread:
//
// - the header's title, which is Rust's `foreground_title` (the stored
//   session's own title, "New chat" for a chat with no session file yet);
// - the mark the foreground chat's activity calls for, which is what decides
//   Send, Stop and the disabled composer's reason (S6: the rule itself lives in
//   Rust, `crates/runtime`, and arrives here as plain data);
// - the notices, unchanged and in the order Rust keeps them, every level drawn;
// - the foreground chat's helpers: the counters line and the cards the panel
//   draws, the selection the reducer keeps for it, and the thread that
//   selection opens (T7b-2);
// - the first pending dialog of that chat, one at a time, which the dialogs
//   slice draws as a modal;
// - the run's one fatal failure, which belongs to no chat and outlives all of
//   them: a launcher that cannot be resolved means no chat can start at all.
//
// No chat open draws no chat chrome at all: a push that carries a title, a
// notice or a dialog for a chat that is already gone must not leave a header
// behind. The fatal bar is the exception, because it is not about a chat.

import type {
  ChatActivityMark,
  DialogView,
  HelperThreadView as HelperThreadView,
  HelpersView_Serialize as HelpersView,
  NoticeView,
  ShellView_Serialize as ShellView,
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
  /** Every notice Rust pushes, oldest first, with its level and its one action. */
  notices: NoticeView[]
  /** The foreground chat's helpers panel, or `null` while no chat is open. */
  helpers: HelpersView | null
  /** The helper card the reducer selected, or `null` for the list itself. */
  selectedHelper: string | null
  /** The selected helper's thread, or `null` while no thread is open. */
  helperThread: HelperThreadView | null
  /** The first dialog that chat's extension is waiting on, or `null`. */
  dialog: DialogView | null
  /** The run's fatal launcher failure, or `null`; not about any one chat. */
  fatal: string | null
  /** What the app found about pi and what was chosen; not about any one chat. */
  shell: ShellView
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

/** Before the first push: no pi found, nothing chosen, nothing asked. */
const NO_SHELL: ShellView = { pi_path: null, choice: null, ask: false }

/** The slice of `view` the main area's chrome draws. */
export function chatOf(view: ViewState | null): ChatViewState {
  const chat = view?.foreground_chat ?? null
  // The fatal bar is the run's, so it is read whatever chat is open.
  const fatal = view?.fatal ?? null
  // So is what the app found about the shell: it is not about a chat either.
  const shell = view?.shell ?? NO_SHELL
  if (chat === null) {
    return {
      chat: null,
      title: null,
      mark: null,
      notices: NO_NOTICES,
      helpers: null,
      selectedHelper: null,
      helperThread: null,
      dialog: null,
      fatal,
      shell,
    }
  }
  return {
    chat,
    title: view?.foreground_title ?? null,
    mark: markOf(view, chat),
    notices: view?.notices ?? NO_NOTICES,
    helpers: view?.helpers ?? null,
    selectedHelper: view?.selected_helper ?? null,
    helperThread: view?.helper_thread ?? null,
    dialog: view?.dialog ?? null,
    fatal,
    shell,
  }
}

/**
 * Whether two pushes say the same about the shell. By value, because every push
 * brings a new object: comparing identity would redraw the app on every text
 * delta, and ignoring it would never draw the answer to the welcome question.
 */
function sameShell(a: ShellView, b: ShellView): boolean {
  return a.pi_path === b.pi_path && a.choice === b.choice && a.ask === b.ask
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
  let state: ChatViewState = {
    chat: null,
    title: null,
    mark: null,
    notices: [],
    helpers: null,
    selectedHelper: null,
    helperThread: null,
    dialog: null,
    fatal: null,
    shell: NO_SHELL,
  }
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
      next.notices === state.notices &&
      next.helpers === state.helpers &&
      next.selectedHelper === state.selectedHelper &&
      next.helperThread === state.helperThread &&
      next.dialog === state.dialog &&
      next.fatal === state.fatal &&
      sameShell(next.shell, state.shell)
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
