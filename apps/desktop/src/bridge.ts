// The thin command layer of the web side: components never call `invoke`
// themselves, they name an intent here and the bridge decides what to send.
//
// A refusal is not thrown back at the view: Rust answers with the current slice
// and the reason reaches the user as `ViewState.workspace_notice` (a folder
// that is gone, a name Windows would not take) or `ViewState.config_notice`
// (what became of the config file) in the next push, which the sidebar draws.

import { commands, type UiIntent } from './contract/bindings'

/** Sends one user intent; the answer is drawn through the next pushed state. */
export async function dispatchIntent(intent: UiIntent): Promise<void> {
  try {
    await commands.dispatchIntent(intent)
  } catch {
    // The push carries the reason; a failed dispatch leaves the view as it was.
  }
}

/**
 * "Choose an existing folder": the native folder picker, then a new chat in the
 * folder the user picked. A refused or cancelled pick changes nothing.
 */
export async function addWorkspace(): Promise<void> {
  try {
    await commands.addWorkspace()
  } catch {
    // Same as above: `workspace_notice` says why nothing was added.
  }
}

/**
 * Copies the raw markdown of one message: the thread names the message by the id
 * Rust gave it, and the clipboard write is the shell's own side effect (T5a).
 * A failed copy is not pushed anywhere: Rust logs it, and a view that claimed a
 * copy the user cannot verify would be worse than silence.
 */
export async function copyMessage(messageId: string): Promise<void> {
  await dispatchIntent({ type: 'CopyMessage', data: { message_id: messageId } })
}

/**
 * Opens a link of a drawn reply in the system browser. The web side only
 * forwards (and refuses an empty or non-web href before calling); Rust decides
 * what it opens, and a refusal is silent here because the shell already told
 * the user nothing happened.
 */
export async function openLink(url: string): Promise<void> {
  try {
    await commands.openLink(url)
  } catch {
    // Rust refuses with a reason that has no drawn home in this slice.
  }
}

/**
 * "Create a new folder": Rust validates the name and then asks for the parent
 * folder, so the name can sit behind a picker for minutes; the caller shows a
 * busy state meanwhile. A refusal is not thrown back at the view either: the
 * dialog closes and the next push carries `workspace_notice`.
 */
export async function createWorkspace(name: string): Promise<void> {
  try {
    await commands.createWorkspace(name)
  } catch {
    // Same as above: `workspace_notice` says why no folder was created.
  }
}
