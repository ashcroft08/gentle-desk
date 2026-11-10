// The notice strip under the header: every pushed notice, in the order Rust
// keeps them (oldest first), each one its own card. This is the slice T6 left
// half-drawn: a notice with nothing to fix now has a place too, instead of being
// filtered out for want of one.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import { plainNotice, resetTauriMock, restartNotice, setHandler, viewState } from '../test-support/tauri-mock'
import type { NoticeView_Serialize } from '../contract/bindings'
import Notices from './Notices.svelte'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))

const DIED = 'Chat died: the child that runs this chat exited.'

function draw(notices: NoticeView_Serialize[]): void {
  render(Notices, { props: { notices } })
}

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
})

describe('the notice strip', () => {
  it('draws every notice, in the order Rust sent them', () => {
    draw([
      plainNotice(1, 'Could not read 3 chat files.', 'Warning'),
      restartNotice(2, DIED, 'c:chat-1'),
    ])

    const texts = [...document.querySelectorAll('.notice .m')].map((line) => line.textContent)
    expect(texts).toEqual([
      'Could not read 3 chat files.',
      'Chat died: the child that runs this chat exited.',
    ])
    // The first is a status, the second an alert: the order survives the roles.
    expect(screen.getByRole('status').textContent).toContain('Warning')
    expect(screen.getByRole('alert').textContent).toContain('Error')
  })

  it('draws nothing when there is nothing to say', () => {
    draw([])

    expect(screen.queryByRole('status')).toBeNull()
    expect(screen.queryByRole('alert')).toBeNull()
  })
})
