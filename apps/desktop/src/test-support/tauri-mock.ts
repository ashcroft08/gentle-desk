// A stand-in for the two Tauri modules the generated bindings call at runtime:
// `@tauri-apps/api/core` (invoke) and `@tauri-apps/api/event` (listen). A test
// mocks both at the API level, so the generated bindings stay in the path under
// test instead of being replaced by a hand-written fake.
//
// Usage in a test file (the factories run before the test file's imports):
//
//   vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
//   vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

import { resetDrags } from './webview'

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

/** Forgets handlers, listeners, drag listeners and recorded calls. */
export function resetTauriMock(): void {
  handlers.clear()
  listeners.clear()
  resetDrags()
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
  const set = listeners.get(name) ?? new Set<(event: MockEvent) => void>()
  listeners.set(name, set)
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

export {
  viewState,
  activity,
  restartNotice,
  confirmDialog,
  selectDialog,
  inputDialog,
  editorDialog,
  plainNotice,
  chatRow,
  workspaceRow,
  sessionsWindow,
} from './view-state'
