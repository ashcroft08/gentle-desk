// What one pushed ViewState says that the previous one did not (S3, S6): which
// chat finished a reply, which one needs the user, and whether a new failure
// notice arrived. Pure and stateless - two states in, the alerts to announce
// out - so the tone and the OS toast (T3) follow this module and decide nothing
// themselves.
//
// The rules come from what Rust already puts in `ViewState`:
//
// - a reply finished when a chat's activity was `Working` and is no longer: the
//   activity list omits idle chats, and the foreground chat never shows
//   `NewReply` (`crates/app/src/chats/activity.rs`). `NeedsAttention` and
//   `Ended` are NOT replies - those turns stopped to ask or died, which the
//   other two kinds already tell - and a chat first seen already finished or
//   idle tells nothing: no turn finished while watching.
// - the user is needed when the foreground dialog goes from none to one, or a
//   chat's activity is newly `NeedsAttention`.
// - a failure is a notice whose `id` is new in this push and whose level is
//   Warning or Error: ids are monotonic (`crates/app/src/state.rs`), Info
//   notices are routine and silent (`notices/visible.ts`).
//
// At most one alert per kind per push: a burst is one sound, not a queue, so
// the first of its kind is kept and the rest of the push is dropped (S6).

import type {
  ChatActivityMark,
  ViewState_Serialize as ViewState,
} from '../contract/bindings'

/** The three things worth announcing (S3); nothing else ever alerts. */
export type AlertKind = 'reply' | 'needs-you' | 'failure'

/**
 * One thing to announce. `chatId` is the chat it is about (absent for a notice,
 * which belongs to no chat) and `text` is the notice's own words (absent for
 * the two chat kinds, whose words the announcer owns).
 */
export type Alert = {
  kind: AlertKind
  chatId?: string
  text?: string
}

/** A chat's mark in this state, or `null` when the list does not name it. */
function markOf(view: ViewState, chatId: string): ChatActivityMark | null {
  return view.activity.find((entry) => entry.chat_id === chatId)?.activity ?? null
}

/** The first chat whose turn was running and ended in a reply, or `null`. */
function finishedReply(prev: ViewState, next: ViewState): Alert | null {
  for (const entry of prev.activity) {
    if (entry.activity !== 'Working') continue
    const after = markOf(next, entry.chat_id)
    if (after === null || after === 'NewReply') {
      return { kind: 'reply', chatId: entry.chat_id }
    }
  }
  return null
}

/** The one ask of this push: a dialog that appeared, else a chat newly waiting. */
function needsYou(prev: ViewState, next: ViewState): Alert | null {
  if (prev.dialog === null && next.dialog !== null) {
    const chatId = next.foreground_chat
    return chatId === null ? { kind: 'needs-you' } : { kind: 'needs-you', chatId }
  }
  const waiting = new Set(
    prev.activity.filter((entry) => entry.activity === 'NeedsAttention').map((entry) => entry.chat_id),
  )
  for (const entry of next.activity) {
    if (entry.activity === 'NeedsAttention' && !waiting.has(entry.chat_id)) {
      return { kind: 'needs-you', chatId: entry.chat_id }
    }
  }
  return null
}

/** The first notice of this push that is new and not routine, or `null`. */
function newFailure(prev: ViewState, next: ViewState): Alert | null {
  const seen = new Set(prev.notices.map((notice) => notice.id))
  for (const notice of next.notices) {
    if (seen.has(notice.id) || notice.level === 'Info') continue
    return { kind: 'failure', text: notice.text }
  }
  return null
}

/**
 * What `next` announces that `prev` did not, one alert per kind, in the order
 * reply, needs-you, failure. `prev === null` is the first view after launch:
 * everything it already holds is old news, so it says nothing (S6).
 */
export function detect(prev: ViewState | null, next: ViewState): Alert[] {
  if (prev === null) return []
  const alerts: Alert[] = []
  const reply = finishedReply(prev, next)
  if (reply !== null) alerts.push(reply)
  const ask = needsYou(prev, next)
  if (ask !== null) alerts.push(ask)
  const failure = newFailure(prev, next)
  if (failure !== null) alerts.push(failure)
  return alerts
}
