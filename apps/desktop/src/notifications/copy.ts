// The words of the app's notifications (S2, S3, S5), in both languages the app
// speaks: one dictionary per language and a builder that answers the whole
// object, so a toast written in the same tick the language changed is already in
// the new one (the same rule the rest of the copy follows).
//
// A chat's own name is NOT here: it is Rust's `foreground_title`, so a reply
// toast says the chat's name when the view can give it and this generic line
// when it cannot. The failure toast's body is the notice's own text, which is
// also Rust's. So these are the words nobody else owns: the generic titles and
// the one line each kind adds, plus the Try it pair (S5).
import type { LanguageView } from '../contract/bindings'

/** What the notifications say, for one language. */
export interface NotifyCopy {
  /** The Try it toast (S5): it is a test, and it says so. */
  testTitle: string
  testBody: string
  /** A reply finished (S3a) and the view cannot name the chat. */
  replyTitle: string
  replyBody: string
  /** A chat needs the user (S3b). */
  needsYouTitle: string
  needsYouBody: string
  /** A new Warning/Error notice (S3c): its body is the notice's own text. */
  failureTitle: string
  failureBody: string
}

/** English: the words the rest of the app already uses for the same things. */
const ENGLISH: NotifyCopy = {
  testTitle: 'gentle-desk',
  testBody: 'This is the tone and the notification, so you know what to expect.',
  replyTitle: 'Reply ready',
  replyBody: 'The chat finished its reply.',
  needsYouTitle: 'Needs you',
  needsYouBody: 'A chat is waiting for your answer.',
  failureTitle: 'Something failed',
  failureBody: 'Open gentle-desk to see what happened.',
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH: typeof ENGLISH = {
  testTitle: 'gentle-desk',
  testBody: 'Este es el tono y el aviso, para que sepas qué esperar.',
  replyTitle: 'Respuesta lista',
  replyBody: 'El chat terminó su respuesta.',
  needsYouTitle: 'Te necesita',
  needsYouBody: 'Un chat espera tu respuesta.',
  failureTitle: 'Algo falló',
  failureBody: 'Abre gentle-desk para ver qué pasó.',
}

/** The notifications' own words in `language`, the whole dictionary at once. */
export function notifyCopy(language: LanguageView): NotifyCopy {
  return language === 'es' ? SPANISH : ENGLISH
}
