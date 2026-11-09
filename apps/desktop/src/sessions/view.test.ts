import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushMicrotasks } from '../test-support/manual-timer'
import {
  handlerArgs,
  pushEvent,
  resetTauriMock,
  sessionsWindow,
  setHandler,
  viewState,
  workspaceRow,
} from '../test-support/tauri-mock'
import { createSessionsView } from './view'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

beforeEach(() => {
  resetTauriMock()
})

describe('sessions view', () => {
  it('reads the page of the viewport and keeps it with the pushed state', async () => {
    const header = workspaceRow('repo')
    setHandler('get_view_state', () => viewState({ sessions_revision: 1, total_sessions: 1 }))
    setHandler('sessions_window', () => sessionsWindow(1, [header]))

    const view = createSessionsView()
    await view.ready
    view.show({ first: 0, visible: 10 })

    await vi.waitFor(() => expect(handlerArgs('sessions_window').length).toBe(1))
    expect(view.get().rows).toEqual([header])
    expect(view.get().total).toBe(1)
    expect(view.get().first).toBe(0)
    expect(view.get().loading).toBe(false)
    expect(view.get().view?.sessions_revision).toBe(1)
    view.dispose()
  })

  it('reads the same viewport again only when a push moves the revision', async () => {
    setHandler('get_view_state', () => viewState({ sessions_revision: 1 }))
    setHandler('sessions_window', () => sessionsWindow(1, [workspaceRow('repo')]))

    const view = createSessionsView()
    await view.ready
    view.show({ first: 0, visible: 10 })
    await vi.waitFor(() => expect(handlerArgs('sessions_window').length).toBe(1))

    // A push that does not move the list leaves the held page alone.
    pushEvent('state-changed', viewState({ sessions_revision: 1 }))
    await flushMicrotasks()
    expect(handlerArgs('sessions_window').length).toBe(1)

    pushEvent('state-changed', viewState({ sessions_revision: 2 }))
    await vi.waitFor(() => expect(handlerArgs('sessions_window').length).toBe(2))
    expect(view.get().view?.sessions_revision).toBe(2)
    view.dispose()
  })
})
