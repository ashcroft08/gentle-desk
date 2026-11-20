// Which chat draws view B, and how far the composer travels to leave it
// (welcome-screen T3, S1, S3, R1). Two rules, both answered from what the page
// already holds, so nothing here asks Rust for a new field:
//
// - a NEW chat is the one whose id is `ChatId::fresh`'s form: `crates/app/src/
//   chats/id.rs` builds `new-<counter>` and `ChatId::session_path` answers `None`
//   for exactly that form, so the id text the page holds is enough to tell a chat
//   with no session file yet from a stored one. A stored chat stays a stored chat
//   while its rows are still on their way, which is what keeps view B from
//   flashing over a history that is about to paint;
// - view B stands while the thread has NO rows at all, messages and items alike
//   (the provisional file rows count: they are rows). The first send adds the
//   reader's own message, and that is what ends it.
//
// The move itself is the FLIP the pane applies: `flipOffset` is how far the dock
// must be pushed back so it starts where it was, and `DOCK_MOVE_MS` is how long
// it then takes to reach where it belongs (design/welcome-screen/02-hero-composer
// sheet B2: about 240 ms, ease-out).
import type { ChatItemView_Serialize as ChatItem, MessageView } from '../contract/bindings'

/** `ChatId::fresh`'s text: the `new-` prefix and an all-digit counter. */
const FRESH = /^new-\d+$/

/** How long the dock takes to travel from the centre of the pane to the bottom. */
export const DOCK_MOVE_MS = 240

/** Whether `chat` is a chat Rust started with no session file (a new chat). */
export function isFreshChat(chat: string | null): boolean {
  return chat !== null && FRESH.test(chat)
}

/** Whether the pane draws view B: a new chat that holds no row at all yet. */
export function isNewSession(
  chat: string | null,
  messages: readonly MessageView[],
  items: readonly ChatItem[],
): boolean {
  return isFreshChat(chat) && messages.length === 0 && items.length === 0
}

/**
 * The inverse transform the move starts from: the dock's top before the class
 * change minus its top after it, or `null` when there was no place to measure
 * (a first paint) or no place to measure now.
 */
export function flipOffset(from: number | null, to: number | null): number | null {
  if (from === null || to === null) return null
  return from - to
}
