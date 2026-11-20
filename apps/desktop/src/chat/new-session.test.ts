// View B's rules, without a window (welcome-screen T3, S1, S3, R1): WHICH chat
// draws the centred composer, and how far the composer has to be pushed back for
// the move to look like one move. The pane that obeys them is ChatPane.svelte.
//
// "A new chat" is Rust's own distinction: `ChatId::fresh` builds `new-<counter>`
// and `ChatId::session_path` answers `None` for exactly that form
// (`crates/app/src/chats/id.rs`), so the page reads the id it already holds. A
// stored chat - still loading its rows, or already showing them - is never fresh,
// which is what keeps view B from flashing over a history that is on its way.
import { describe, expect, it } from 'vitest'
import type { ChatItemView_Serialize as ChatItem } from '../contract/bindings'
import { DOCK_MOVE_MS, flipOffset, isFreshChat, isNewSession } from './new-session'

/** One text row of the transcript, as `items` carries it. */
const ROW: ChatItem = {
  kind: 'Message',
  data: {
    id: 'msg-1',
    role: 'User',
    text: 'hola',
    streaming: false,
    error: null,
    images: [],
  },
}

describe('a fresh chat', () => {
  it('is the id Rust builds for a chat with no session file', () => {
    // `ChatId::fresh()`: the prefix and the counter, and nothing else.
    expect(isFreshChat('new-1')).toBe(true)
    expect(isFreshChat('new-42')).toBe(true)
  })

  it('is neither a stored session nor a string that merely looks like one', () => {
    // A stored chat's id is its session path, prefixed (or the bare path a
    // sidebar row still sends): neither is a chat with no file yet.
    expect(isFreshChat('session:C:\\sessions\\alpha.jsonl')).toBe(false)
    expect(isFreshChat('C:\\sessions\\alpha.jsonl')).toBe(false)
    // Rust's own `is_fresh` reads the prefix AND an all-digit counter, so a path
    // that happens to start with the same letters is not a new chat.
    expect(isFreshChat('new-session.md')).toBe(false)
    expect(isFreshChat('new-')).toBe(false)
    expect(isFreshChat('new-1-alpha')).toBe(false)
    expect(isFreshChat(null)).toBe(false)
  })
})

describe('which chat draws view B', () => {
  it('is a fresh chat with no rows at all, messages and items alike', () => {
    expect(isNewSession('new-1', [], [])).toBe(true)
  })

  it('stops being one the moment the first row exists', () => {
    // The first send adds the reader's own message, in both slices.
    expect(isNewSession('new-1', [ROW.data], [])).toBe(false)
    expect(isNewSession('new-1', [], [ROW])).toBe(false)
  })

  it('is never a stored chat, however few rows it has arrived with yet', () => {
    // A stored chat that is still loading its rows has none: it is not view B,
    // and the history it is about to paint never flashes a welcome.
    expect(isNewSession('session:C:\\sessions\\alpha.jsonl', [], [])).toBe(false)
    expect(isNewSession(null, [], [])).toBe(false)
  })
})

describe('the move from the centre to the bottom', () => {
  it('takes the artboard’s 240 ms', () => {
    expect(DOCK_MOVE_MS).toBe(240)
  })

  it('is the offset that puts the dock back where it was', () => {
    // Measured above its usual place (a bigger `top` afterwards): the inverse
    // transform is negative, so the dock starts at the old spot and goes to 0.
    expect(flipOffset(200, 400)).toBe(-200)
    expect(flipOffset(400, 200)).toBe(200)
  })

  it('is nothing at all when there is no place to come from, or no distance', () => {
    expect(flipOffset(null, 400)).toBeNull()
    expect(flipOffset(200, null)).toBeNull()
    expect(flipOffset(300, 300)).toBe(0)
  })
})
