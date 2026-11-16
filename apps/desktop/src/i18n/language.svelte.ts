// The language the app's own words are written in (0.2.0, S1, S2 and S4).
//
// Rust owns the choice (it lives in the app config, so it survives a restart),
// and this module copies it: the language every push carries is adopted as it
// lands, so a change reaches the words in the same tick and needs no restart
// (S2) and no webview reload. It keeps no copy of its own - no `localStorage`
// entry - for the reason `theme/paint.ts` gives about the theme: a second copy
// could disagree with the config and would outlive a config that was replaced.
//
// While the state carries `null` the app has no preference yet, so the MACHINE's
// own language decides, with exactly the rule `gentle_core::i18n` applies in Rust
// (the primary subtag `es` means Spanish, anything else English) - reimplemented
// here because the web side cannot call into the crate. That detected language is
// then reported ONCE through the normal write path (D2), so a Spanish machine
// starts Spanish on every later run and the choice stops depending on the
// machine. Nothing is reported for an English reading: English is what absence
// already means, so storing it would freeze a choice the user never made.
//
// The boot step is this module's own, because the Settings rail reads the
// language the moment it is imported and the app mounts after the theme is
// painted (`main.ts`): subscribing on import is what makes both true, and a
// `startLanguage()` call again adopts the current state a second time without
// ever subscribing twice.

import { dispatchIntent } from '../bridge'
import {
  commands,
  events,
  type LanguageView,
  type ViewState_Serialize as ViewState,
} from '../contract/bindings'

/** The language a run starts in when the machine names none English enough. */
const FALLBACK: LanguageView = 'en'

/** The language this app holds right now, as the words of this build name it. */
let current = $state<LanguageView>(detectLanguage(machineLocale()))

/** Whether this module already reported the language it detected. */
let reported = false

/** Whether the pushes are already subscribed: the boot step is repeatable. */
let subscribed = false

/** Whether a push landed already, so a first read in flight is stale by then. */
let pushed = false

/**
 * The language the app's own words are written in, read reactively: a `$derived`
 * or a template that reads `language.current` redraws when the state changes it.
 */
export const language = {
  get current(): LanguageView {
    return current
  },
}

/**
 * The language a locale tag starts the app in, the Rust rule of
 * `gentle_core::i18n::Language::detect_from_locale`: the primary subtag (before
 * the first `-` or `_`), trimmed of surrounding spaces and lowercased, and only
 * exactly `es` is Spanish. Empty, one-letter junk and every other language are
 * English, and no input throws.
 */
export function detectLanguage(locale: string | null | undefined): LanguageView {
  const primary = (locale ?? '')
    .split(/[-_]/)[0]
    .trim()
    .toLowerCase()
  return primary === 'es' ? 'es' : FALLBACK
}

/**
 * Chooses the language of the app's own words: the words change in this same
 * tick, and Rust stores the choice and re-emits the slice. The same language
 * again changes nothing in Rust, so nothing is saved.
 */
export function setLanguage(next: LanguageView): void {
  current = next
  void dispatchIntent({ type: 'SetLanguage', data: { language: next } })
}

/**
 * Follows the language of every pushed state, and adopts the state this run
 * starts from. Called once on import; a second call reads the current state
 * again and still holds one subscription.
 */
export async function startLanguage(): Promise<void> {
  if (!subscribed) {
    subscribed = true
    try {
      await events.stateChanged.listen((event) => {
        pushed = true
        adopt(event.payload)
      })
    } catch {
      // Without the pushes the read below still adopts this run's language.
    }
  }
  try {
    const view = await commands.getViewState()
    // A push that landed while this read was in flight is NEWER than it.
    if (!pushed) adopt(view)
  } catch {
    // A bridge that cannot answer leaves the machine's own language standing.
  }
}

/** Takes the language of one state, or the machine's rule when it stores none. */
function adopt(view: Pick<ViewState, 'language'>): void {
  if (view.language !== null) {
    current = view.language
    return
  }
  const detected = detectLanguage(machineLocale())
  current = detected
  report(detected)
}

/**
 * Writes the detected language back through the normal config write, once per
 * module lifetime: a run with no preference adopts the machine's language and
 * stores it, and no later push turns that into a second write.
 */
function report(detected: LanguageView): void {
  if (reported || detected === FALLBACK) return
  reported = true
  void dispatchIntent({ type: 'SetLanguage', data: { language: detected } })
}

/** What the machine says its own language is, `''` where there is no such API. */
function machineLocale(): string {
  return typeof navigator === 'undefined' ? '' : navigator.language
}

void startLanguage()
