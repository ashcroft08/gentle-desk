// The Settings pane's Try it (S5), and the API that pane imports: the tone first,
// then one toast, and the toast's answer back - 'sent' or 'denied' - so the
// maintainer hears the tone and learns whether Windows allows the notification
// without waiting for a real event.
//
// The tone plays whatever the toast then answers, on purpose: hearing it is half
// of what the button is for, and a machine that refuses notifications still has
// working audio.
import type { LanguageView } from '../contract/bindings'
import { notifyCopy } from './copy'
import { sendToast, type ToastResult } from './toast'
import { playTone } from './tone'

/** Plays the tone and sends the one test toast, in `language`. */
export async function tryNotification(language: LanguageView): Promise<ToastResult> {
  const copy = notifyCopy(language)
  await playTone()
  return sendToast(copy.testTitle, copy.testBody)
}
