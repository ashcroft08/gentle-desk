import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { SessionsWindow } from '../contract/bindings'
import {
  deferred,
  handlerArgs,
  resetTauriMock,
  sessionRow,
  sessionsWindow,
  setHandler,
} from '../test-support/tauri-mock'
import { flushMicrotasks, manualTimer } from '../test-support/manual-timer'
import { createWindowFetcher, MAX_WINDOW, OVERSCAN, pageFor, QUERY_DEBOUNCE_MS } from './window'

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

  it('debounces a query change by 150 ms', () => {
    expect(QUERY_DEBOUNCE_MS).toBe(150)
  })
})

describe('window fetcher', () => {
  it('keeps the rows and the total of the page it asked for', async () => {
    setHandler('sessions_window', () => sessionsWindow(40, [sessionRow('a'), sessionRow('b')]))
    const fetcher = createWindowFetcher()
    fetcher.request('', { first: 0, visible: 10 })
    await flushMicrotasks()
    expect(handlerArgs('sessions_window')).toEqual([{ offset: 0, length: 30, query: '' }])
    expect(fetcher.get().total).toBe(40)
    expect(fetcher.get().rows.map((row) => row.id)).toEqual(['a', 'b'])
    expect(fetcher.get().loading).toBe(false)
    expect(fetcher.get().error).toBeNull()
    expect(fetcher.rowAt(1)?.id).toBe('b')
  })

  it('answers an empty list without inventing rows', async () => {
    setHandler('sessions_window', () => sessionsWindow(0, []))
    const fetcher = createWindowFetcher()
    fetcher.request('', { first: 0, visible: 10 })
    await flushMicrotasks()
    expect(fetcher.get().total).toBe(0)
    expect(fetcher.get().rows).toEqual([])
  })

  it('asks for the last page of a long list', async () => {
    setHandler('sessions_window', () => sessionsWindow(100, [sessionRow('last')]))
    const fetcher = createWindowFetcher()
    fetcher.request('', { first: 99, visible: 10 })
    await flushMicrotasks()
    expect(handlerArgs('sessions_window')[0]).toEqual({ offset: 89, length: 30, query: '' })
    expect(fetcher.rowAt(89)?.id).toBe('last')
  })

  it('does not refetch a page it already holds', async () => {
    setHandler('sessions_window', () => sessionsWindow(40, [sessionRow('a')]))
    const fetcher = createWindowFetcher()
    fetcher.request('', { first: 0, visible: 10 })
    await flushMicrotasks()
    fetcher.request('', { first: 0, visible: 10 })
    await flushMicrotasks()
    expect(handlerArgs('sessions_window')).toHaveLength(1)
  })

  it('drops the cache when the revision changes', async () => {
    setHandler('sessions_window', () => sessionsWindow(40, [sessionRow('a')]))
    const fetcher = createWindowFetcher()
    fetcher.request('', { first: 0, visible: 10 })
    await flushMicrotasks()
    expect(fetcher.rowAt(0)?.id).toBe('a')
    fetcher.setRevision(2)
    expect(fetcher.rowAt(0)).toBeUndefined()
    fetcher.request('', { first: 0, visible: 10 })
    await flushMicrotasks()
    expect(handlerArgs('sessions_window')).toHaveLength(2)
  })

  it('ignores a response that resolves after a newer request', async () => {
    const older = deferred<SessionsWindow>()
    const newer = deferred<SessionsWindow>()
    const answers = [older, newer]
    setHandler('sessions_window', () => answers.shift()?.promise)
    const fetcher = createWindowFetcher()
    fetcher.request('', { first: 0, visible: 10 })
    fetcher.request('', { first: 50, visible: 10 })
    newer.resolve(sessionsWindow(100, [sessionRow('new')]))
    await flushMicrotasks()
    older.resolve(sessionsWindow(100, [sessionRow('old')]))
    await flushMicrotasks()
    expect(fetcher.get().rows.map((row) => row.id)).toEqual(['new'])
    expect(fetcher.rowAt(0)).toBeUndefined()
  })

  it('runs a query change once, after the debounce', async () => {
    const timer = manualTimer()
    setHandler('sessions_window', () => sessionsWindow(2, [sessionRow('a')]))
    const fetcher = createWindowFetcher({ schedule: timer.schedule, cancel: timer.cancel })
    fetcher.request('t', { first: 0, visible: 10 })
    fetcher.request('ta', { first: 0, visible: 10 })
    expect(timer.pending()).toBe(1)
    expect(handlerArgs('sessions_window')).toHaveLength(0)
    timer.run()
    await flushMicrotasks()
    expect(handlerArgs('sessions_window')).toEqual([{ offset: 0, length: 30, query: 'ta' }])
    expect(fetcher.get().query).toBe('ta')
  })

  it('reports a rejected invoke as an error state instead of throwing', async () => {
    setHandler('sessions_window', () => Promise.reject(new Error('list unavailable')))
    const fetcher = createWindowFetcher()
    fetcher.request('', { first: 0, visible: 10 })
    await flushMicrotasks()
    expect(fetcher.get().error).toBe('list unavailable')
    expect(fetcher.get().loading).toBe(false)
    expect(fetcher.get().rows).toEqual([])
  })
})
