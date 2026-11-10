// A stand-in for the two Tauri modules the generated bindings call at runtime:
// `@tauri-apps/api/core` (invoke) and `@tauri-apps/api/event` (listen). A test
// mocks both at the API level, so the generated bindings stay in the path under
// test instead of being replaced by a hand-written fake.
//
// Usage in a test file (the factories run before the test file's imports):
//
//   vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
//   vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

import type {
  ChatActivityView,
  DialogView_Serialize,
  NoticeView_Serialize,
  SessionsWindow_Serialize,
  SessionTreeRow_Serialize,
  ViewState_Serialize,
  WorkspaceRowView_Serialize,
} from '../contract/bindings'

/** The arguments of one invoke call, as the bridge commands take them. */
export type InvokeArgs = Record<string, unknown>

/** Answers one command call. A promise it returns is awaited by the caller. */
export type InvokeHandler = (args: InvokeArgs) => unknown

export type MockEvent = { payload: unknown }

export type InvokeCall = { command: string; args: InvokeArgs }

export type Deferred<T> = {
  promise: Promise<T>
  resolve: (value: T) => void
  reject: (error: unknown) => void
}

const handlers = new Map<string, InvokeHandler>()
const listeners = new Map<string, Set<(event: MockEvent) => void>>()

/** Every command call this mock received, oldest first. */
export const invokeCalls: InvokeCall[] = []

/** A promise the test resolves when it decides the response is ready. */
export function deferred<T>(): Deferred<T> {
  let resolve!: (value: T) => void
  let reject!: (error: unknown) => void
  const promise = new Promise<T>((ok, fail) => {
    resolve = ok
    reject = fail
  })
  return { promise, resolve, reject }
}

/** Forgets handlers, listeners and recorded calls. Call it in `beforeEach`. */
export function resetTauriMock(): void {
  handlers.clear()
  listeners.clear()
  invokeCalls.length = 0
}

/** Makes `command` answer with `handler`. Without a handler a call rejects. */
export function setHandler(command: string, handler: InvokeHandler): void {
  handlers.set(command, handler)
}

/** The arguments of every recorded call of `command`. */
export function handlerArgs(command: string): InvokeArgs[] {
  return invokeCalls.filter((call) => call.command === command).map((call) => call.args)
}

/** How many listeners `name` currently has. */
export function listenerCount(name: string): number {
  return listeners.get(name)?.size ?? 0
}

/** Emits `name` with `payload` to every registered listener, in order. */
export function pushEvent(name: string, payload: unknown): void {
  for (const callback of [...(listeners.get(name) ?? [])]) callback({ payload })
}

function tauriInvoke(command: string, args?: InvokeArgs): Promise<unknown> {
  const call = { command, args: args ?? {} }
  invokeCalls.push(call)
  const handler = handlers.get(command)
  if (handler === undefined) {
    return Promise.reject(new Error(`tauri-mock: no handler for ${command}`))
  }
  try {
    return Promise.resolve(handler(call.args))
  } catch (error) {
    return Promise.reject(error)
  }
}

function tauriListen(name: string, callback: (event: MockEvent) => void): Promise<() => void> {
  let set = listeners.get(name)
  if (set === undefined) {
    set = new Set()
    listeners.set(name, set)
  }
  set.add(callback)
  return Promise.resolve(() => {
    set.delete(callback)
  })
}

/** The `@tauri-apps/api/core` module shape a test mocks in. */
export function coreModule(): { invoke: typeof tauriInvoke } {
  return { invoke: tauriInvoke }
}

/** The `@tauri-apps/api/event` module shape a test mocks in. */
export function eventModule(): {
  listen: typeof tauriListen
  once: typeof tauriListen
  emit: () => Promise<void>
  emitTo: () => Promise<void>
} {
  return {
    listen: tauriListen,
    once: tauriListen,
    emit: () => Promise.resolve(),
    emitTo: () => Promise.resolve(),
  }
}

/** A view state as Rust pushes it, with the fields a test cares about set. */
export function viewState(overrides: Partial<ViewState_Serialize> = {}): ViewState_Serialize {
  return {
    foreground_chat: null,
    foreground_title: null,
    list_status: { type: 'Complete' },
    sessions_revision: 1,
    total_sessions: 0,
    messages: [],
    notices: [],
    dialog: null,
    helpers: null,
    selected_helper: null,
    helper_thread: null,
    activity: [],
    fatal: null,
    config_notice: null,
    workspace_notice: null,
    ...overrides,
  }
}

/** One activity entry, as `ViewState.activity` carries it. */
export function activity(chatId: string, mark: ChatActivityView['activity']): ChatActivityView {
  return { chat_id: chatId, activity: mark }
}

/** The notice a chat's death leaves: an error that offers the one action. */
export function restartNotice(id: number, text: string, chatId: string): NoticeView_Serialize {
  return {
    id,
    text,
    level: 'Error',
    action: { type: 'RestartChat', data: { chat_id: chatId } },
  }
}

/** The confirm dialog a `confirm` extension request leaves. */
export function confirmDialog(
  id: string,
  title = 'Allow this command?',
  message = 'Run the command?',
): DialogView_Serialize {
  return { kind: 'Confirm', id, title, message }
}

/** The select dialog a `select` extension request leaves. */
export function selectDialog(
  id: string,
  options: string[] = ['a', 'b'],
  title = 'Pick a letter',
): DialogView_Serialize {
  return { kind: 'Select', id, title, options }
}

/** The input dialog an `input` extension request leaves. */
export function inputDialog(
  id: string,
  placeholder: string | null = 'name',
  title = 'Your name',
): DialogView_Serialize {
  return { kind: 'Input', id, title, placeholder }
}

/** The editor dialog an `editor` extension request leaves. */
export function editorDialog(
  id: string,
  prefill: string | null = null,
  title = 'Edit the note',
): DialogView_Serialize {
  return { kind: 'Editor', id, title, prefill }
}

/** A notice that only reports something, with nothing to fix (T7's to draw). */
export function plainNotice(
  id: number,
  text: string,
  level: NoticeView_Serialize['level'] = 'Warning',
): NoticeView_Serialize {
  return { id, text, level, action: null }
}

/** One chat row, as `sessions_window` answers it under its workspace header. */
export function chatRow(
  id: string,
  overrides: Partial<Omit<Extract<SessionTreeRow_Serialize, { kind: 'Chat' }>, 'kind'>> = {},
): SessionTreeRow_Serialize {
  return {
    kind: 'Chat',
    id,
    title: id,
    project: 'gentle-desk',
    message_count: 1,
    modified_ago: 'now',
    ...overrides,
  }
}

/** One workspace header row, as `sessions_window` answers it. */
export function workspaceRow(
  label: string,
  overrides: Partial<Omit<WorkspaceRowView_Serialize, 'label'>> = {},
): SessionTreeRow_Serialize {
  return {
    kind: 'Workspace',
    key: label.toLowerCase(),
    label,
    path: `C:\\dev\\${label}`,
    name: label,
    collapsed: false,
    chat_count: 0,
    available: true,
    reason: null,
    ...overrides,
  }
}

/** One window page, as `sessions_window` answers it. */
export function sessionsWindow(
  total: number,
  rows: SessionTreeRow_Serialize[],
): SessionsWindow_Serialize {
  return { total, rows }
}
