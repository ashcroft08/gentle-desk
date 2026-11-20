// The alerting path (S1, S2, S3, S6): the tone and the OS toast, driven by the
// pushes and silenced by the window's own focus. `detect.ts` decides WHICH alerts
// a push carries and `focus.svelte.ts` reports whether the maintainer is already
// looking; this file is the only place the two meet the outputs, so every rule
// stays in the module that owns it.
//
// The wiring follows `state-changed` itself, the way `theme/paint.ts` does,
// rather than borrowing one of the stores: an alert is about the app, not about
// one slice of the window, and every push carries the whole state a comparison
// needs. Its rules, each one a spec:
//
// - nothing on the first state after launch: that is `detect(prev = null)`, and
//   the same rule is what keeps reopening a stored chat silent (S6);
// - nothing while the window is in front (S1) and nothing while the setting is
//   off (S4) - and a push that arrives then is still consumed, so turning the
//   setting on later never replays an old transition;
// - one tone and one toast per alert, tone first (S2, S3);
// - one kind never sounds twice inside five seconds, so a burst is a single
//   greeting and not a queue (S6). The budget is per kind: a failure right after
//   a reply is still worth hearing.
//
// Nothing here throws: the tone and the toast both resolve, and a toast the OS
// refused is simply not sent.

import {
  events,
  type LanguageView,
  type ViewState_Serialize as ViewState,
} from '../contract/bindings'
import { notifyCopy, type NotifyCopy } from './copy'
import { detect, type Alert, type AlertKind } from './detect'
import { windowFocused } from './focus.svelte'
import { playTone } from './tone'
import { sendToast } from './toast'

/** How long one kind stays quiet after it sounded (S6). */
export const RATE_LIMIT_MS = 5_000

/** The language this run speaks, or a getter so a switch is followed at once. */
export type LanguageSource = LanguageView | (() => LanguageView)

export type NotifyOptions = {
  /**
   * The newest state the caller already holds, when it holds one: the push is
   * then only the tick and this is what gets announced. Left out, the state the
   * push carried is the state announced, which is what the shell does - it keeps
   * no copy of its own.
   */
  view?: () => ViewState | null
  /** The language the toasts are written in (S2). */
  language: LanguageSource
}

/** The chat's own title, when the view is the one naming it; otherwise nothing. */
function chatTitle(alert: Alert, view: ViewState): string | null {
  if (alert.chatId === undefined || alert.chatId !== view.foreground_chat) return null
  return view.foreground_title
}

/** The one toast each alert shows, in `copy`, with the chat's or the notice's own words. */
function words(alert: Alert, view: ViewState, copy: NotifyCopy): { title: string; body: string } {
  switch (alert.kind) {
    case 'reply':
      return { title: chatTitle(alert, view) ?? copy.replyTitle, body: copy.replyBody }
    case 'needs-you':
      return { title: chatTitle(alert, view) ?? copy.needsYouTitle, body: copy.needsYouBody }
    case 'failure':
      return { title: copy.failureTitle, body: alert.text ?? copy.failureBody }
  }
}

/** Reads the language a toast is written in, whether it was a value or a getter. */
function speak(source: LanguageSource): LanguageView {
  return typeof source === 'function' ? source() : source
}

/**
 * Starts announcing what the pushes say, and returns the function that stops it
 * (which also releases the window listeners the focus store added). Called once
 * per run, from the shell.
 */
export function startNotifications(options: NotifyOptions): () => void {
  const focus = windowFocused()
  // When each kind last sounded; a kind missing from it has not sounded yet.
  const sounded = new Map<AlertKind, number>()
  let previous: ViewState | null = null
  let stopped = false
  let unlisten: (() => void) | null = null

  /** Plays the tone and shows the toast of one alert, tone first. */
  async function raise(alert: Alert, view: ViewState): Promise<void> {
    const copy = notifyCopy(speak(options.language))
    const { title, body } = words(alert, view, copy)
    try {
      await playTone()
      await sendToast(title, body)
    } catch {
      // Both resolve by contract; a path that still throws is only silence.
    }
  }

  /** What one pushed state announces: the alerts, minus the ones not due yet. */
  function announce(next: ViewState): void {
    const alerts = detect(previous, next)
    previous = next
    // The two gates come first, so a push nothing may announce never spends a
    // kind's budget: the alert is not held back, it is not due at all.
    if (next.notifications !== 'when-unfocused' || focus.focused) return
    const now = Date.now()
    for (const alert of alerts) {
      const at = sounded.get(alert.kind)
      if (at !== undefined && now - at < RATE_LIMIT_MS) continue
      sounded.set(alert.kind, now)
      void raise(alert, next)
    }
  }

  void (async () => {
    try {
      const stop = await events.stateChanged.listen((event) => {
        announce(options.view?.() ?? event.payload)
      })
      if (stopped) {
        stop()
        return
      }
      unlisten = stop
    } catch {
      // A build with no pushes has nothing to announce.
    }
  })()

  return () => {
    stopped = true
    focus.dispose()
    unlisten?.()
    unlisten = null
  }
}
