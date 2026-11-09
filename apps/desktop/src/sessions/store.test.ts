import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { ViewState_Serialize } from '../contract/bindings'
import {
  activity,
  deferred,
  handlerArgs,
  listenerCount,
  pushEvent,
  resetTauriMock,
  setHandler,
  viewState,
} from '../test-support/tauri-mock'
import { createSessionsStore, newerRevision } from './store'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

async function listening(): Promise<void> {
  await vi.waitFor(() => expect(listenerCount('state-changed')).toBe(1))
}

beforeEach(() => {
  resetTauriMock()
})

describe('newerRevision', () => {
  it('orders revisions across the wrap of the counter', () => {
    expect(newerRevision(1, 2)).toBe(true)
    expect(newerRevision(2, 1)).toBe(false)
    expect(newerRevision(1, 1)).toBe(false)
    expect(newerRevision(0xffffffff, 0)).toBe(true)
  })
})

describe('sessions store', () => {
  it('does one first read and exposes the pushed list fields', async () => {
    setHandler('get_view_state', () =>
      viewState({
        list_status: { type: 'Loading', data: { done: 4, total: 9 } },
        sessions_revision: 3,
        total_sessions: 9,
        activity: [activity('a', 'Working')],
      }),
    )
    const store = createSessionsStore()
    await store.ready
    expect(handlerArgs('get_view_state')).toHaveLength(1)
    const state = store.get()
    expect(state?.list_status).toEqual({ type: 'Loading', data: { done: 4, total: 9 } })
    expect(state?.sessions_revision).toBe(3)
    expect(state?.total_sessions).toBe(9)
    expect(state?.activity).toEqual([activity('a', 'Working')])
  })

  it('keeps the newer state when a push arrives before the first read resolves', async () => {
    const first = deferred<ViewState_Serialize>()
    setHandler('get_view_state', () => first.promise)
    const store = createSessionsStore()
    await listening()
    pushEvent('state-changed', viewState({ sessions_revision: 2, total_sessions: 7 }))
    first.resolve(viewState({ sessions_revision: 1, total_sessions: 3 }))
    await store.ready
    expect(store.get()?.sessions_revision).toBe(2)
    expect(store.get()?.total_sessions).toBe(7)
  })

  it('applies a push that keeps the revision because its activity changed', async () => {
    setHandler('get_view_state', () => viewState({ sessions_revision: 4 }))
    const store = createSessionsStore()
    await store.ready
    pushEvent('state-changed', viewState({ sessions_revision: 4, activity: [activity('a', 'Ended')] }))
    expect(store.get()?.activity).toEqual([activity('a', 'Ended')])
  })

  it('ignores a push older than the state it holds', async () => {
    setHandler('get_view_state', () => viewState({ sessions_revision: 5, total_sessions: 11 }))
    const store = createSessionsStore()
    await store.ready
    pushEvent('state-changed', viewState({ sessions_revision: 4, total_sessions: 1 }))
    expect(store.get()?.sessions_revision).toBe(5)
    expect(store.get()?.total_sessions).toBe(11)
  })

  it('unsubscribes on dispose and stops applying pushes', async () => {
    setHandler('get_view_state', () => viewState({ sessions_revision: 6 }))
    const store = createSessionsStore()
    await store.ready
    const seen: number[] = []
    const stop = store.subscribe((state) => {
      if (state !== null) seen.push(state.sessions_revision)
    })
    store.dispose()
    expect(listenerCount('state-changed')).toBe(0)
    pushEvent('state-changed', viewState({ sessions_revision: 9 }))
    stop()
    expect(store.get()?.sessions_revision).toBe(6)
    expect(seen).toEqual([])
  })

  it('keeps pushes when the first read fails', async () => {
    setHandler('get_view_state', () => Promise.reject(new Error('bridge offline')))
    const store = createSessionsStore()
    await store.ready
    expect(store.get()).toBeNull()
    pushEvent('state-changed', viewState({ sessions_revision: 2 }))
    expect(store.get()?.sessions_revision).toBe(2)
  })
})
