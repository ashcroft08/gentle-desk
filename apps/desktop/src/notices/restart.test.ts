// The restart rule, one test per case: only the notices that carry the restart
// action are drawn, each with the chat its action names, and a notice that only
// reports something stays out of this slice.

import { describe, expect, it } from 'vitest'
import { plainNotice, restartNotice } from '../test-support/tauri-mock'
import { restartNoticesOf } from './restart'

describe('restartNoticesOf', () => {
  it('keeps the death notice with the chat its action restarts', () => {
    const notice = restartNotice(7, 'Chat died.', 'c:chat-1')

    expect(restartNoticesOf([notice])).toEqual([
      { id: 7, text: 'Chat died.', chatId: 'c:chat-1' },
    ])
  })

  it('leaves a notice with nothing to fix to the notices slice', () => {
    expect(restartNoticesOf([plainNotice(1, 'Could not read 3 chat files.')])).toEqual([])
  })

  it('keeps the app order among several restart notices', () => {
    const first = restartNotice(1, 'One died.', 'c:one')
    const second = restartNotice(2, 'Two died.', 'c:two')

    expect(restartNoticesOf([first, second]).map((row) => row.id)).toEqual([1, 2])
  })

  it('draws nothing when there is nothing', () => {
    expect(restartNoticesOf([])).toEqual([])
  })
})
