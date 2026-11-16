// The thread's words, in one place (the sidebar's `copy.ts` convention): the
// two role lines a screen reader hears before a message, the Copy label, the
// one live status line and the model-error title. `crates/ui/src/view/thread.rs`
// draws a cross mark before "Model error"; no glyph is drawn here, because the
// thread draws no icon characters at all.
//
// 0.2.0 (S3, D4): those words are written in BOTH languages here - one dictionary
// per language and a builder that answers the whole object, so no string-keyed
// lookup is introduced and the thread redraws them in the same tick it is told the
// new language (S2). The English column is what the thread has drawn since the
// iced view, unchanged; the Spanish one is neutral - no voseo, no regionalisms -
// and keeps its accents.
//
// What does NOT move: a message's text, the markdown a reply is drawn from and
// the provider's own error text, which arrive exactly as written (D7).
import type { LanguageView } from '../contract/bindings'

/** The thread's own words for one language. */
export interface ThreadCopy {
  /** The accessible name of the scroll region. */
  threadLabel: string
  /** The visually hidden author line of a user message's article. */
  youSaid: string
  /** The same for the assistant's. */
  assistantSaid: string
  /** The one message action this slice draws. */
  copy: string
  /** The only live region of the thread, present while a message streams. */
  replying: string
  /** The title of the danger box a failed turn draws. */
  modelErrorTitle: string
  /** The activity line while the chat's process comes up, and while it works. */
  opening: string
  working: string
}

/** English: the words the thread already drew. */
const ENGLISH: ThreadCopy = {
  threadLabel: 'Chat thread',
  youSaid: 'You said',
  assistantSaid: 'Assistant said',
  copy: 'Copy',
  replying: 'Assistant is replying',
  modelErrorTitle: 'Model error',
  opening: 'Opening chat',
  working: 'Working',
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH: typeof ENGLISH = {
  threadLabel: 'Hilo del chat',
  youSaid: 'Tú dijiste',
  assistantSaid: 'El asistente dijo',
  copy: 'Copiar',
  replying: 'El asistente está respondiendo',
  modelErrorTitle: 'Error del modelo',
  opening: 'Abriendo el chat',
  working: 'Trabajando',
}

/** The thread's own words in `language`, the whole dictionary at once (D4). */
export function threadCopy(language: LanguageView): typeof ENGLISH {
  return language === 'es' ? SPANISH : ENGLISH
}
