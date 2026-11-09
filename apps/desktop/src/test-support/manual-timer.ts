// A scheduler a test fires by hand, so a debounce is observed without sleeping.

export type ManualTimer = {
  /** Records a callback; the returned handle is its id. */
  schedule: (fn: () => void, ms: number) => unknown
  /** Forgets a pending callback. */
  cancel: (handle: unknown) => void
  /** Runs every pending callback once, oldest first. */
  run: () => void
  /** How many callbacks are still pending. */
  pending: () => number
}

export function manualTimer(): ManualTimer {
  const jobs = new Map<number, () => void>()
  let next = 1
  return {
    schedule: (fn) => {
      const handle = next
      next += 1
      jobs.set(handle, fn)
      return handle
    },
    cancel: (handle) => {
      jobs.delete(handle as number)
    },
    run: () => {
      const runnable = [...jobs.values()]
      jobs.clear()
      for (const fn of runnable) fn()
    },
    pending: () => jobs.size,
  }
}

/** Lets queued promise reactions settle, without waiting on the clock. */
export async function flushMicrotasks(times = 6): Promise<void> {
  for (let i = 0; i < times; i += 1) await Promise.resolve()
}
