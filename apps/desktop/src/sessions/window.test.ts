import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { SessionsWindow_Serialize } from '../contract/bindings'
import {
  chatRow,
  deferred,
  handlerArgs,
  resetTauriMock,
  sessionsWindow,
  setHandler,
  workspaceRow,
} from '../test-support/tauri-mock'
import { flushMicrotasks } from '../test-support/manual-timer'
import { createWindowFetcher, MAX_WINDOW, OVERSCAN, pageFor } from './window'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

beforeEach(() => {
  resetTauriMock()
})

describe('pageFor', () => {
  it('asks for the viewport plus the overscan on both sides', () => {
    expect(pageFor({ first: 0, visible: 10 })).toEqual({ offset: 0, length: 10 + 2 * OVERSCAN })
    expect(pageFor({ first: 40, visible: 10 })).toEqual({ offset: 40 - OVERSCAN, length: 30 })
  })

  it('never asks for a negative offset', () => {
    expect(pageFor({ first: 3, visible: 10 }).offset).toBe(0)
  })

  it('caps the length the way Rust caps the answer', () => {
    expect(pageFor({ first: 0, visible: 500 }).length).toBe(MAX_WINDOW)
  })
})

describe('window fetcher', () => {
  it('keeps the rows and the total of the page it asked for', async () => {
    setHandler('sessions_window', () =>
      sessionsWindow(40, [workspaceRow('repo', { chat_count: 1 }), chatRow('a')]),
    )
    const fetcher = createWindowFetcher()
    fetcher.request({ first: 0, visible: 10 })
    await flushMicrotasks()
    expect(handlerArgs('sessions_window')).toEqual([{ offset: 0, length: 30 }])
    expect(fetcher.get().total).toBe(40)
    expect(fetcher.get().rows.map((row) => row.kind)).toEqual(['Workspace', 'Chat'])
    expect(fetcher.get().loading).toBe(false)
    expect(fetcher.get().error).toBeNull()
    expect(fetcher.rowAt(1)?.kind).toBe('Chat')
  })

  it('answers a tree with no rows without inventing rows', async () => {
    setHandler('sessions_window', () => sessionsWindow(0, []))
    const fetcher = createWindowFetcher()
    fetcher.request({ first: 0, visible: 10 })
    await flushMicrotasks()
    expect(fetcher.get().total).toBe(0)
    expect(fetcher.get().rows).toEqual([])
  })

  it('asks for the last page of a long tree', async () => {
    setHandler('sessions_window', () => sessionsWindow(100, [chatRow('last')]))
    const fetcher = createWindowFetcher()
    fetcher.request({ first: 99, visible: 10 })
    await flushMicrotasks()
    expect(handlerArgs('sessions_window')[0]).toEqual({ offset: 89, length: 30 })
    expect(fetcher.rowAt(89)?.kind).toBe('Chat')
  })

  it('does not refetch a page it already holds', async () => {
    setHandler('sessions_window', () => sessionsWindow(40, [chatRow('a')]))
    const fetcher = createWindowFetcher()
    fetcher.request({ first: 0, visible: 10 })
    await flushMicrotasks()
    fetcher.request({ first: 0, visible: 10 })
    await flushMicrotasks()
    expect(handlerArgs('sessions_window')).toHaveLength(1)
  })

  it('drops the cache when the revision changes', async () => {
    setHandler('sessions_window', () => sessionsWindow(40, [chatRow('a')]))
    const fetcher = createWindowFetcher()
    fetcher.request({ first: 0, visible: 10 })
    await flushMicrotasks()
    expect(fetcher.rowAt(0)?.kind).toBe('Chat')
    fetcher.setRevision(2)
    expect(fetcher.rowAt(0)).toBeUndefined()
    fetcher.request({ first: 0, visible: 10 })
    await flushMicrotasks()
    expect(handlerArgs('sessions_window')).toHaveLength(2)
  })

  it('ignores a response that resolves after a newer request', async () => {
    const older = deferred<SessionsWindow_Serialize>()
    const newer = deferred<SessionsWindow_Serialize>()
    const answers = [older, newer]
    setHandler('sessions_window', () => answers.shift()?.promise)
    const fetcher = createWindowFetcher()
    fetcher.request({ first: 0, visible: 10 })
    fetcher.request({ first: 50, visible: 10 })
    newer.resolve(sessionsWindow(100, [chatRow('new')]))
    await flushMicrotasks()
    older.resolve(sessionsWindow(100, [chatRow('old')]))
    await flushMicrotasks()
    expect(fetcher.get().rows.map((row) => row.kind)).toEqual(['Chat'])
    expect(fetcher.rowAt(0)).toBeUndefined()
  })

  it('reports a rejected invoke as an error state instead of throwing', async () => {
    setHandler('sessions_window', () => Promise.reject(new Error('list unavailable')))
    const fetcher = createWindowFetcher()
    fetcher.request({ first: 0, visible: 10 })
    await flushMicrotasks()
    expect(fetcher.get().error).toBe('list unavailable')
    expect(fetcher.get().loading).toBe(false)
    expect(fetcher.get().rows).toEqual([])
  })
})
