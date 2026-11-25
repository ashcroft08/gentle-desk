// How many open chats are in the middle of a turn (close-chats S3).
//
// The Updates pane's "Close the chats" button closes every chat that is NOT running
// a turn and leaves the others open (the app's own close refuses a chat mid-turn:
// `close_chat` in `crates/app`). The pane says how many stay and why, so the button
// never looks broken.
//
// The count comes from the activity marks the push already carries, never from a
// second read: a chat is in the middle of a turn when its mark is `Working` (a model
// call in flight) or `NeedsAttention` (the turn is waiting for the user's answer: it
// is still that turn, and the close refuses it the same way). A chat that merely
// finished (`NewReply`), waits in a queue (`Queued`), is opening or has ended is not
// counted: the button closes those.
import type { ChatActivityMark } from '../contract/bindings'

/** One entry of `ViewState.activity`: only the mark is read here. */
interface ActivityEntry {
  activity: ChatActivityMark
}

/** The marks that mean "the turn of this chat is still running". */
const MID_TURN: readonly ChatActivityMark[] = ['Working', 'NeedsAttention']

/** The number of chats the button will leave open because they are mid-turn. */
export function workingChats(activity: readonly ActivityEntry[] | null | undefined): number {
  if (!activity) return 0
  return activity.filter((entry) => MID_TURN.includes(entry.activity)).length
}
