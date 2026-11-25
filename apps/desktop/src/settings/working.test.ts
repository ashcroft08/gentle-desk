// Which open chats the Updates gate's button will leave open (close-chats S3): the
// ones in the middle of a turn. The count reads the activity marks the push already
// carries; these tests pin WHICH marks count, because the app's own close refuses a
// chat whose turn is running and the pane has to tell the truth about how many stay.
import { describe, expect, it } from 'vitest'
import type { ChatActivityMark } from '../contract/bindings'
import { workingChats } from './working'

/** An activity list: one entry per mark, in the order given. */
function marks(...names: ChatActivityMark[]): { chat_id: string; activity: ChatActivityMark }[] {
  return names.map((activity, index) => ({ chat_id: `chat-${index}`, activity }))
}

describe('workingChats', () => {
  it('is zero when there is nothing to read', () => {
    expect(workingChats(null)).toBe(0)
    expect(workingChats(undefined)).toBe(0)
    expect(workingChats([])).toBe(0)
  })

  it('counts a chat whose model call is in flight', () => {
    expect(workingChats(marks('Working'))).toBe(1)
  })

  it('counts a chat whose turn waits for the user, which is still that turn', () => {
    // A permission question during a turn marks the chat NeedsAttention, and the
    // app's close refuses it exactly like a working one.
    expect(workingChats(marks('NeedsAttention'))).toBe(1)
  })

  it('does not count a chat the button will close', () => {
    // A finished reply, a queued prompt, an opening chat and an ended one are all
    // closable: the button takes them, so the pane must not say they stay.
    expect(workingChats(marks('NewReply', 'Queued', 'Opening', 'Ended'))).toBe(0)
  })

  it('counts each mid-turn chat once among the others', () => {
    expect(workingChats(marks('Working', 'NewReply', 'NeedsAttention', 'Queued', 'Working'))).toBe(3)
  })
})
