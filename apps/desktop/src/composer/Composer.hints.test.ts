// The keys hint, one test per state of the chat: while a turn runs, or while its
// prompt waits for a free slot, the composer shows Stop and Enter sends nothing
// at all, so the line under the box must not promise a send it will not do; an
// idle chat still promises it.
//
// Split out of `Composer.test.ts` for the same reason its images were: that file
// is at the repo's 200-line cap, and the rule below is the hint's own business.
// The two columns of the dictionary are pinned in `copy.test.ts`, and that the
// drawn line follows the language of the moment in `copy.flip.test.ts`; this file
// pins only WHICH of the two lines each state draws.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, render, screen } from '@testing-library/svelte'
import type { ChatActivityMark } from '../contract/bindings'
import { resetTauriMock, setHandler, viewState } from '../test-support/tauri-mock'
import Composer from './Composer.svelte'
import { composerCopy } from './copy'

vi.mock('@tauri-apps/api/core', () => import('../test-support/tauri-mock').then((m) => m.coreModule()))
vi.mock('@tauri-apps/api/event', () => import('../test-support/tauri-mock').then((m) => m.eventModule()))
vi.mock('@tauri-apps/api/webview', () => import('../test-support/webview').then((m) => m.webviewModule()))

type Props = { chat: string | null; mark?: ChatActivityMark | null }

/** Draws the composer for one chat in one state. */
function draw(props: Props) {
  return render(Composer, { props })
}

// The two lines read through the dictionary, in English: what a run that has not
// been told otherwise draws.
const { keysHint: KEYS_IDLE, keysHintWorking: KEYS_WORKING } = composerCopy('en')

afterEach(cleanup)

beforeEach(() => {
  resetTauriMock()
  setHandler('dispatch_intent', () => viewState())
})

describe('Composer keys hint', () => {
  it('says Enter is off while the chat works, and drops the send promise', () => {
    draw({ chat: 'chat-1', mark: 'Working' })

    expect(screen.getByText(KEYS_WORKING)).toBeTruthy()
    expect(screen.queryByText(KEYS_IDLE)).toBeNull()
  })

  it('says Enter is off while the chat waits for a slot, and drops the send promise', () => {
    draw({ chat: 'chat-1', mark: 'Queued' })

    expect(screen.getByText(KEYS_WORKING)).toBeTruthy()
    expect(screen.queryByText(KEYS_IDLE)).toBeNull()
  })

  // PRESERVE: an idle composer is the one it always was, hint included.
  it('still promises the send on an idle chat, and never says Enter is off', () => {
    draw({ chat: 'chat-1' })

    expect(screen.getByText(KEYS_IDLE)).toBeTruthy()
    expect(screen.queryByText(KEYS_WORKING)).toBeNull()
  })
})
