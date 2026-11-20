// The welcome's words (welcome-screen T2, S2, S3): the title, the rose's
// accessible name, the line a workspace with no chat says, the two sentences
// that name a workspace, and the two lines of the folder that cannot host a chat.
// The screen is view A of `design/welcome-screen/01-resume.html` (sheets A and
// A2), and these are its words.
//
// Written in BOTH languages, one dictionary per language and a builder that
// answers the whole object, as the neighbouring areas do: the sentences that name
// a workspace stay functions and no string-keyed lookup is introduced. The
// English column is the approved artboard's; the Spanish one is neutral - no
// voseo, no regionalisms - and keeps its accents.
//
// What does NOT move: the workspace's label and a chat's title, which Rust sends
// as the sidebar draws them, and `last.reason`, which Rust already worded for the
// sidebar and the screen prints VERBATIM (R3: never translated, never reworded).
import type { LanguageView } from '../contract/bindings'

/** The welcome's own words for one language. */
export interface HomeCopy {
  /** The view's title, the page's h2 under the header's h1. */
  title: string
  /** The rose's accessible name: the picture is decoration with a name. */
  roseLabel: string
  /** The line beside the folder when it holds no chat yet, instead of an age. */
  noChats: string
  /** "· last activity 21m ago": the line beside the folder, given its age. */
  lastActivity: (age: string) => string
  /** The one primary action, naming the workspace it starts a chat in. */
  newSession: (label: string) => string
  /** The recents section's heading, naming the same workspace. */
  recentsTitle: (label: string) => string
  /** The quiet links' line: the other configured workspaces, then the links. */
  othersTitle: string
  /** What to do instead, under a folder that is gone (sheet A2). */
  pickAnother: string
}

/** English: the words sheets A and A2 draw. */
const ENGLISH: HomeCopy = {
  title: 'Pick up where you left off',
  roseLabel: 'The gentle-shell rose',
  noChats: 'no chats yet',
  lastActivity: (age) => `· last activity ${age} ago`,
  newSession: (label) => `New session in ${label}`,
  recentsTitle: (label) => `Recent chats in ${label}`,
  othersTitle: 'Other workspaces:',
  pickAnother: 'Pick another workspace in the sidebar.',
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH: typeof ENGLISH = {
  title: 'Sigue donde lo dejaste',
  roseLabel: 'La rosa de gentle shell',
  noChats: 'sin chats todavía',
  lastActivity: (age) => `· última actividad hace ${age}`,
  newSession: (label) => `Nueva sesión en ${label}`,
  recentsTitle: (label) => `Chats recientes de ${label}`,
  othersTitle: 'Otros espacios:',
  pickAnother: 'Elige otro espacio en la barra lateral.',
}

/** The welcome's own words in `language`, the whole dictionary at once. */
export function homeCopy(language: LanguageView): typeof ENGLISH {
  return language === 'es' ? SPANISH : ENGLISH
}
