// The notices the strip draws. Info notices (registry refreshed, token counts,
// model config applied) are routine and only took room above the conversation,
// so they are not drawn; Rust still keeps them. Warnings and errors stay until
// the user dismisses them.

import type { NoticeView } from '../contract/bindings'

/** The notices worth the user's room, in Rust's order. */
export function visibleNotices(notices: NoticeView[]): NoticeView[] {
  return notices.filter((notice) => notice.level !== 'Info')
}
