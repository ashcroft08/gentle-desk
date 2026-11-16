// The rules a workspace removal draws on (S1, S2, S3): the count and the size the
// dialog names, the path it shows with its middle elided, the bold scope of its
// one consequence line, the chat titles its disclosure lists, and the two
// commands it sends.
//
// The numbers are DISPLAY rules only: what a removal would delete comes from Rust
// (`workspace_removal_offer`), never from a count guessed here. The view carries
// `chats` and `bytes`, and everything below turns those two into the words the
// dialog and its buttons are drawn with.
import { commands } from '../contract/bindings'
import type { WorkspaceRemovalView } from '../contract/bindings'
import { isWorkspaceRow, type TreeRow } from '../sessions/rows'

/** The key the sidebar registers its removal host under (S8). */
export const REMOVAL_HOST = 'workspace-removal'

/**
 * What a workspace header row's `...` menu asks the sidebar for: the one place
 * that owns the dialog, its words and its command (`Sidebar.svelte`). A row
 * reaches it through the context rather than a prop, so the tree stays a drawing
 * of rows and a header row carries no removal state of its own.
 */
export type RemovalHost = { askRemove: (path: string) => void }

/**
 * One consequence line, split where its scope starts and ends (S3): the dialog
 * draws `before`, then `scope` in bold, then `after`, so the scope is the only
 * emphasised run and no sentence is put together from a translation at runtime.
 */
export type ConsequenceLine = { before: string; scope: string; after: string }

/** One consequence line as one sentence, for anywhere that needs it whole. */
export function consequenceText(line: ConsequenceLine): string {
  return `${line.before}${line.scope}${line.after}`
}

/** `1 chat` / `2 chats`: the count's own word, which both languages spell alike. */
export function chatCount(chats: number): string {
  return chats === 1 ? '1 chat' : `${chats} chats`
}

/** A byte count the way the dialog shows it: megabytes with one decimal (`1.7 MB`). */
export function megabytes(bytes: number | null): string | null {
  return bytes === null ? null : `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

/** The scope of a removal: `2 chats (1.7 MB)`, or `2 chats` when no size arrived. */
export function chatScope(chats: number, bytes: number | null): string {
  const size = megabytes(bytes)
  return size === null ? chatCount(chats) : `${chatCount(chats)} (${size})`
}

/**
 * A path with its middle elided (S3): the root and the folder itself identify the
 * workspace, so `C:\dev\gentle-desk` reads `C:\…\gentle-desk` and a deeper path
 * keeps both of its ends the same way. A path of one or two parts is all ends
 * already, so it is drawn whole.
 */
export function elideMiddle(path: string): string {
  const whole = path.trim()
  const parts = whole.split(/[\\/]+/).filter((part) => part.length > 0)
  if (parts.length < 3) return whole
  const rooted = /^[\\/]/.test(whole)
  const sep = whole.includes('\\') ? '\\' : '/'
  return `${rooted ? sep : parts[0] + sep}…${sep}${parts[parts.length - 1]}`
}

/**
 * The titles of the chats a removal would delete, from the rows the sidebar
 * already holds: the offer carries their count and their total size, and their
 * names are what the disclosure lists (S2). A page that does not hold them all -
 * a folded workspace, a page still being read - answers fewer titles, and the
 * dialog draws its disclosure only when every one of them is there.
 */
export function chatTitles(rows: TreeRow[], path: string): string[] {
  const titles: string[] = []
  let inside = false
  for (const row of rows) {
    if (isWorkspaceRow(row)) inside = row.path === path
    else if (inside) titles.push(row.title)
  }
  return titles
}

/**
 * The removal offer of one workspace (S1, S2): the count and the bytes the dialog
 * names. `null` when the shell refused the read - the dialog draws no number
 * rather than a guess, and says the read failed instead (R3-002).
 */
export async function removalOffer(path: string): Promise<WorkspaceRemovalView | null> {
  try {
    const answer = await commands.workspaceRemovalOffer(path)
    return answer.status === 'ok' ? answer.data : null
  } catch {
    return null
  }
}

/**
 * Removes one workspace (S1, S5): its chats go with it in the one command, and
 * the `confirmed` sent here is always the dialog's own answer, never a default.
 *
 * A rejected answer is NOT swallowed (R3-003): a Rust `Err` arrives as an answer
 * of status `error` and a failed call as a rejection, and both are handed back so
 * the dialog can stay open and say what happened instead of closing over an
 * operation that never ran. A refusal that Rust answers as an `Ok` view - a chat
 * of that workspace still working - is still the push's to draw.
 */
export async function removeWorkspace(path: string): Promise<void> {
  const answer = await commands.removeWorkspace(path, true)
  if (answer.status === 'error') throw new Error(answer.error)
}
