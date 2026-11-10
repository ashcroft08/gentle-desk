// The one place a dialog turns into the answer Rust folds: the three shapes the
// child's protocol takes (confirmed, value, cancelled), and the intent that
// names them back to the dialog they answer.
//
// Nothing of the typed text reaches here on its own: a dialog calls these only
// when the user submits, so the text stays the view's own state (S2).

import type { DialogAnswer, DialogView, UiIntent } from '../contract/bindings'

/** A select option, an input's text or an editor's text. */
export function valueAnswer(value: string): DialogAnswer {
  return { type: 'Value', data: { value } }
}

/** A confirm dialog's Yes (`true`) or No (`false`). */
export function confirmedAnswer(confirmed: boolean): DialogAnswer {
  return { type: 'Confirmed', data: { confirmed } }
}

/** Escape, Cancel, or a dialog the user dismissed: the extension decides. */
export const CANCELLED: DialogAnswer = { type: 'Cancelled' }

/** The intent that answers `dialog`, named by the id Rust sent with it. */
export function answerIntent(dialog: DialogView, answer: DialogAnswer): UiIntent {
  return { type: 'AnswerDialog', data: { id: dialog.id, answer } }
}
