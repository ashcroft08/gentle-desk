// The one notice this slice draws: the report of a chat whose process is gone,
// which carries the single action that fixes it (S3, T6).
//
// Extensions' own notices and the info/warning levels are T7's; this rule keeps
// them out of the way instead of drawing a line with no action that the design
// has no place for yet.

import type { NoticeView } from '../contract/bindings'

/** One notice that can be fixed from here, with the chat its action restarts. */
export type RestartNotice = {
  /** The notice's id, which the dismiss intent refers back to. */
  id: number
  /** The reason the child exited, as the app worded it. */
  text: string
  /** The chat the restart starts again, the id the restart intent takes back. */
  chatId: string
}

/**
 * The restart notices among `notices`, oldest first, exactly as Rust keeps them.
 * The action's own name is what selects them: a notice that offers nothing is
 * not drawn here.
 */
export function restartNoticesOf(notices: NoticeView[]): RestartNotice[] {
  return notices.flatMap((notice) => {
    const action = notice.action
    if (action == null || action.type !== 'RestartChat') return []
    return [{ id: notice.id, text: notice.text, chatId: action.data.chat_id }]
  })
}
