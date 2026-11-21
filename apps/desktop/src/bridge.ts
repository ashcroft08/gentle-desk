// The thin command layer of the web side: components never call `invoke`
// themselves, they name an intent here and the bridge decides what to send.
//
// A refusal is not thrown back at the view: Rust answers with the current slice
// and the reason reaches the user as `ViewState.workspace_notice` (a folder
// that is gone, a name Windows would not take) or `ViewState.config_notice`
// (what became of the config file) in the next push, which the sidebar draws.

import { commands, type AttachmentView, type SettingsSnapshot_Serialize as SettingsSnapshot, type TerminalActionView, type UiIntent, type WhatsNewRead } from './contract/bindings'

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
 * Renames one chat (T2): the row sends the session path it was given and the name
 * the user typed in its title, and Rust appends it to the session file. A
 * refused rename - a chat that is working, a blank name, a file the disk would
 * not take - is not thrown back at the view: the next push carries the reason in
 * `ViewState.sessions_notice`.
 */
export async function renameSession(sessionId: string, name: string): Promise<void> {
  try {
    await commands.renameSession(sessionId, name)
  } catch {
    // The push carries the reason; a failed call leaves the title as it was.
  }
}

/**
 * Deletes one chat (T3): the row's confirm dialog sends the session path it named,
 * and this is the only caller, so `confirmed` is always the user's answer in that
 * dialog and never a default. Rust removes that one file and asks for the pass
 * that heals the list. A refusal - a chat that is working, a file that is already
 * gone - is not thrown back at the view: the next push carries the reason in
 * `ViewState.sessions_notice`, and a workspace the delete empties says so in
 * `ViewState.workspace_notice`.
 */
export async function deleteSession(sessionId: string): Promise<void> {
  try {
    await commands.deleteSession(sessionId, true)
  } catch {
    // The push carries the reason; a failed call leaves the file where it is.
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

/**
 * The file dialog the composer's clip opens (design/composer-dock T4, S3): the
 * paths the human picked, and the empty list for a cancel so nothing changes.
 * `title` is the composer's own word for the dialog, in the language it is drawn
 * in, so one dictionary names both the button and what it opens. A dialog that
 * could not open is the same as a cancel: nothing is attached and nothing is
 * said on top of the line the composer already has.
 */
export async function pickFiles(title: string): Promise<string[]> {
  try {
    const answer = await commands.pickFiles(title)
    // The command answers `ViewState`-style errors in the envelope, so a refusal
    // is the same as a cancel here: nothing is attached.
    return answer.status === 'ok' ? answer.data : []
  } catch {
    return []
  }
}

/**
 * What each path the clip or a drop named is (design/composer-dock T4, R5, R6):
 * an image is handed on to the image command, every other file becomes a chip,
 * and a refused path carries its reason. `folder` is the foreground chat's own
 * working directory, which is what makes a file "outside" it. A call that fails
 * attaches nothing: it is the shell's own command, so a refusal has nothing to
 * say that the composer could draw.
 */
export async function describeAttachments(
  paths: string[],
  folder: string | null,
): Promise<AttachmentView[]> {
  try {
    return await commands.describeAttachments(paths, folder)
  } catch {
    return []
  }
}

/**
 * Attaches the images a drop named: the paths come from Tauri's own drop event,
 * because a browser `File` has no path to give. Nothing but PATHS travel up: Rust
 * reads the files, checks them, and answers with a thumbnail's name and URL in
 * the next push, or with `images_notice` when it refused one.
 */
export async function attachImages(paths: string[]): Promise<void> {
  try {
    await commands.attachImages(paths)
  } catch {
    // The next push carries `images_notice`, the same reason the call carried.
  }
}

/**
 * The dismiss of the what's-new modal (S5): it stores the version this run runs
 * as and clears the notes it pushed. It takes no argument on purpose - the page
 * says it put the modal away, and the shell is the one that knows its version -
 * and a failure is silent, because the notes simply come back and the modal is
 * drawn again, which is the right answer to a dismiss that did not land.
 */
export async function storeLastRunVersion(): Promise<void> {
  try {
    await commands.storeLastRunVersion()
  } catch {
    // The next push still carries the notes; the modal comes back.
  }
}

/**
 * The Settings dialog's one read (settings-snapshot): every page's data in one
 * plain object - the theme, the language, the home, the running version, and the
 * three reads that cost something (the sign-ins, the published versions and the
 * subscription usage). The dialog asks for it ONCE per open and every section
 * draws from the answer, so walking the rail reads nothing.
 *
 * `null` is a load that did not answer at all: the dialog keeps drawing what the
 * last push carried, and a section whose own read failed says so INSIDE the
 * object (each section carries its own status), never here.
 */
export async function settingsSnapshot(): Promise<SettingsSnapshot | null> {
  try {
    const answer = await commands.settingsSnapshot()
    // The command answers its refusals in the envelope, so a load that did not
    // run is the same here as one that said nothing: `null`.
    return answer.status === 'ok' ? answer.data : null
  } catch {
    return null
  }
}

/**
 * "What is new in this version" (settings-redesign T4, S9, D6): the one read on
 * demand of the RUNNING version's notes, so the About pane can open the modal on
 * every run and not only right after an update. It takes no argument - the shell
 * knows which version this run runs as and fetches the notes off the window's
 * thread.
 *
 * The answer says what the pane draws and nothing more: `Shown` means the notes
 * reached `ViewState.whats_new` and the modal is up by itself, so the pane draws
 * NOTHING of its own; `Unavailable(line)` carries the one quiet line it shows
 * instead, already worded in the app's language, and leaves the modal closed.
 * A refusal - the shell closing under the click - is the same as no answer:
 * `null`, and the pane stays as quiet as it was.
 */
export async function showWhatsNew(): Promise<WhatsNewRead | null> {
  try {
    const answer = await commands.showWhatsNew()
    // The command answers its refusals in the envelope, so a read that did not
    // run is the same here as one that said nothing: `null`.
    return answer.status === 'ok' ? answer.data : null
  } catch {
    return null
  }
}

/**
 * Opens a visible terminal window on one of the jobs the desk does not do
 * itself (0.1.12, S1, S2, T2, T3): the shell's profiles screen, its login or
 * logout, or its own update. Rust resolves the launcher and runs it with the
 * home flags the chats' children get, so the window reads the home the desk is
 * showing. A refusal - no launcher on this machine, no console to open - comes
 * back as the one line the pane draws under its button, so a click that did
 * nothing is never silent.
 */
export async function openTerminal(action: TerminalActionView): Promise<string | null> {
  try {
    const answer = await commands.openTerminal(action)
    return answer.status === 'ok' ? null : answer.error
  } catch (error) {
    return error instanceof Error ? error.message : String(error)
  }
}

/**
 * Takes one attached image away before its send, named by the id the composer
 * drew. Nothing is sent, and a chat that goes away first changes nothing.
 */
export async function removeImage(id: string): Promise<void> {
  try {
    await commands.removeImage(id)
  } catch {
    // The next push carries what the chat really holds.
  }
}
