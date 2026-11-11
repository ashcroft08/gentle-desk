// The updater's notice as plain data: what the app says when a newer release
// exists, while it downloads it, and when something goes wrong. Pure, so the
// rules are testable without a window, a network or a running app.

export type UpdateState =
  | { kind: 'idle' }
  | { kind: 'available'; version: string }
  | { kind: 'downloading'; version: string; done: number; total: number | null }
  | { kind: 'ready'; version: string }
  | { kind: 'failed'; version: string | null; message: string }
  | { kind: 'dismissed' }

/** Nothing newer, or no answer at all: the window says nothing. */
export function stillIdle(): UpdateState {
  return { kind: 'idle' }
}

/** A newer release the app can install. */
export function available(version: string): UpdateState {
  return { kind: 'available', version }
}

/** The install started, or moved: `total` is `null` when the server sent no size. */
export function downloading(state: UpdateState, done = 0, total?: number): UpdateState {
  const version = state.kind === 'downloading' ? state.version : 'version' in state ? state.version : null
  if (version === null) return state
  return { kind: 'downloading', version, done, total: total ?? null }
}

/** Downloaded and verified by the plugin's own signature check. */
export function ready(state: UpdateState): UpdateState {
  // A failed state may carry no version at all, and there is nothing to finish.
  if (!('version' in state) || state.version === null) return state
  return { kind: 'ready', version: state.version }
}

/** The install or the check failed; the version is kept when it is known. */
export function failed(state: UpdateState, message: string): UpdateState {
  const version = 'version' in state ? state.version : null
  return { kind: 'failed', version, message }
}

/** Put the notice away for this run; a download in flight or a finished one keeps it. */
export function dismissed(state: UpdateState): UpdateState {
  if (state.kind === 'downloading' || state.kind === 'ready') return state
  return { kind: 'dismissed' }
}
