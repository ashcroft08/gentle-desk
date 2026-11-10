// The one action a notice may offer, as the contract names it. The rule is a
// table on the action's own type, so the card draws whatever Rust sends instead
// of deciding for itself that a button means "restart": an action this build has
// no rule for draws no button, and the notice keeps its text and its dismiss.

import type { NoticeActionView, UiIntent } from '../contract/bindings'
import type { IconName } from '../icons/shapes'
import { RESTART } from './copy'

/** One drawn action: what the button says, the icon it draws and what it sends. */
export type NoticeAction = { label: string; icon: IconName; intent: UiIntent }

/** The button for `action`, or `null` when this build cannot draw it. */
export function actionOf(action: NoticeActionView): NoticeAction | null {
  switch (action.type) {
    case 'RestartChat':
      return {
        label: RESTART,
        icon: 'restart',
        intent: { type: 'RestartChat', data: { chat_id: action.data.chat_id } },
      }
    default:
      return null
  }
}
