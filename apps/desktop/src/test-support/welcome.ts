// View A's data (welcome-screen T1), as Rust pushes it: the workspace the user
// worked in last, the chats it holds and the other configured workspaces, with
// every field a test does not override set to the ordinary case (an available
// folder, no reason). Split from `view-state.ts` so neither file grows past its
// lint cap, and built from the generated types so a changed contract breaks the
// tests that draw it.
import type {
  WelcomeChat,
  WelcomeLink,
  WelcomeView_Serialize,
  WelcomeWorkspace_Serialize,
} from '../contract/bindings'

/** The workspace worked in last: available, three chats, no age yet. */
export function welcomeWorkspace(
  overrides: Partial<WelcomeWorkspace_Serialize> = {},
): WelcomeWorkspace_Serialize {
  return {
    path: 'C:\\dev\\gentle-desk',
    label: 'gentle-desk',
    available: true,
    reason: null,
    chat_count: 3,
    last_activity_ms: null,
    ...overrides,
  }
}

/** One recent chat: its RAW session path is the id `OpenChat` takes back. */
export function welcomeChat(
  chatId: string,
  overrides: Partial<Omit<WelcomeChat, 'chat_id'>> = {},
): WelcomeChat {
  return { chat_id: chatId, title: chatId, modified_ms: null, ...overrides }
}

/** One other workspace: a quiet link, available unless a test says otherwise. */
export function welcomeLink(label: string, overrides: Partial<WelcomeLink> = {}): WelcomeLink {
  return { path: `C:\\dev\\${label}`, label, available: true, ...overrides }
}

/** The whole slice: a last workspace with no recents and no other workspaces. */
export function welcomeView(
  overrides: Partial<WelcomeView_Serialize> = {},
): WelcomeView_Serialize {
  return { last: welcomeWorkspace(), recents: [], others: [], ...overrides }
}
