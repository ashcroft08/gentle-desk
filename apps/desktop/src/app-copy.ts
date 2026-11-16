// The app shell's own words (0.2.0, S3 and D4): the skip link that lands the
// keyboard in the chat, the scrim that closes a floating sidebar, and the main
// area's placeholder while no chat is open.
//
// They belong to no other area's copy module - the shell is the frame around the
// sidebar, the chat and every panel - so the shell keeps its own small dictionary
// rather than borrowing the sidebar's or the chat's words (D5: one module per
// area, never one file per string). One dictionary per language and a builder that
// answers the whole object, so a shell drawn from it changes language in the same
// tick (S2). The English column is what the shell has drawn since the Tauri
// migration, unchanged; the Spanish one is neutral - no voseo, no regionalisms -
// and keeps its accents.
//
// What does NOT move: the chat's title beside these words, which Rust sends (D7).
import type { LanguageView } from './contract/bindings'

/** The app shell's own words for one language. */
export interface AppCopy {
  /** The skip link, first in the tab order: it lands the focus in the chat. */
  skipToChat: string
  /** The scrim under a floating sidebar: one click closes it. */
  closeSidebar: string
  /** The main area's heading while no chat is open. */
  noChatOpen: string
  /** What to do about it: pick a chat, or start one. */
  noChatHint: string
}

/** English: the words the shell already drew. */
const ENGLISH: AppCopy = {
  skipToChat: 'Skip to chat',
  closeSidebar: 'Close sidebar',
  noChatOpen: 'No chat open',
  noChatHint: 'Choose a chat in the workspace tree, or start a new session.',
}

/** Neutral Spanish: no voseo, no regionalisms, nothing left in English. */
const SPANISH: typeof ENGLISH = {
  skipToChat: 'Ir al chat',
  closeSidebar: 'Cerrar la barra lateral',
  noChatOpen: 'Ningún chat abierto',
  noChatHint: 'Elige un chat en el árbol de espacios de trabajo o inicia una sesión nueva.',
}

/** The app shell's own words in `language`, the whole dictionary at once (D4). */
export function appCopy(language: LanguageView): typeof ENGLISH {
  return language === 'es' ? SPANISH : ENGLISH
}
