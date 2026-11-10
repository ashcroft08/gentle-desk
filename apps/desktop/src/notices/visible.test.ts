// Which of Rust's notices the strip draws: info notices (registry refreshed,
// token counts, model config applied) are routine and only took room above the
// conversation, so they are not drawn at all; warnings and errors stay until the
// user dismisses them.

import { describe, expect, it } from 'vitest'
import { plainNotice } from '../test-support/tauri-mock'
import { visibleNotices } from './visible'

describe('the notices that are drawn', () => {
  it('drops every info notice and keeps warnings and errors in their order', () => {
    const all = [
      plainNotice(1, 'Skill registry refreshed (19 skills)', 'Info'),
      plainNotice(2, 'Could not read 3 chat files.', 'Warning'),
      plainNotice(3, 'TPS 35.4 tok/s.', 'Info'),
      plainNotice(4, 'Chat died.', 'Error'),
    ]

    expect(visibleNotices(all).map((notice) => notice.id)).toEqual([2, 4])
  })

  it('draws nothing when every notice is info', () => {
    expect(visibleNotices([plainNotice(1, 'Skill registry refreshed', 'Info')])).toEqual([])
  })
})
