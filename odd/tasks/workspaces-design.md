# workspaces-design: the sidebar as workspaces with their chats (backlog 13)

Decision record for the chosen design (direction A, `design/gentle-desk-v2/01a-tree-chat.html`
and `01b`). Facts are marked as verified by reading code or files; proposals are marked as such.
Nothing here is built yet.

## Specs

The user's own words, kept as typed:

- S1: "vamos a eliminar por completo la seccion de chats, vamos a dejar solo lo de workspace".
- S2: "el boton de new chat cambia a new session que me abre un nuevo chat en la carpeta donde me encuentra".
- S3: "en seccion de la carpeta va a existen un icono de nuevo chat".
- S4: New session with no current workspace: "se desabilita".
- S5: The workspace menu: "son las opciones de ese workspace luego vemos que opciones puede tener si no puede tener ninguna no va" the "..." icon.
- S6: "si al final los chats se guardar en un a carpeta, es mejor trabajar solo con workspace".

## Facts (verified)

- A chat is a pi session file: `<agent home>\sessions\--<cwd with separators turned into dashes>--\<stamp>_<id>.jsonl`.
  Every chat therefore lives in a folder; there is no folderless chat in pi. This is what makes S6 true.
- The folder name of a session directory is lossy (separators and colons become dashes), so two different
  folders can produce the same name. gentle-shell-desktop and gentle-desk both read the folder from the `cwd`
  field of the session header, never from the directory name. gentle-desk: the project of a chat is the last
  segment of that `cwd` (`crates/app/src/state.rs`, `SessionItem::project`).
- gentle-shell-desktop has no workspace concept: a flat list grouped by day (UTC boundaries), and "new chat"
  never passes a folder, so the child inherits the process folder silently. gentle-desk must not copy that.
- The real sessions folder of this machine holds 12 project folders and 18 chats. Some folders are empty
  (one is the user's home folder), and several are temporary folders (their origin is not known; the dates are from before this work).
- The process layer can start a child with a working folder (`crates/backend/src/process.rs`, option `cwd`), but
  the chat host's spawner always passes none (`crates/backend/src/chat_host/spawner.rs`, `cwd: None`). The folder
  has to be plumbed through the chat host; the capability exists.
- Today the list is flat: `sessions_window(offset, length, query)` returns rows of one list, and a row carries
  `id, title, project, message_count, modified_ago` (`apps/desktop/src-tauri/src/contract.rs`).

## Decisions

Decided by the user:

- D1 (S1, S6): one model, the workspace. No separate "Chats" section and no independent-chat group.
- D2 (S2): the top button is **New session**; it opens a new chat in the folder of the current workspace.
- D3 (S3): each workspace row has a new-chat icon (circle plus) that shows on hover of the whole workspace
  section or on keyboard focus; same effect as D2 for that workspace.
- D4 (S4): with no current workspace, **New session** is disabled. In the design it uses `aria-disabled` and a
  visible reason, so a keyboard or screen-reader user can reach the reason.
- D5 (S5): no workspace menu is drawn. It exists only if it ever has options.
- D6: open and closed folder icons show the expanded state; there is no chevron (`aria-expanded` is kept).

Proposed by the assistant (not yet confirmed by the user):

- P1: the workspace is the `cwd` of the session header. Its name is the last path segment; when two workspaces
  share it, show enough of the parent path to tell them apart.
- P2: order workspaces by last activity (the newest session), chats inside by last activity.
- P3: hide a workspace that has no chats, and temporary folders, by default.
- P4: the sidebar search keeps searching all workspaces at once (the terminal shows one location at a time; the
  app can find a chat without knowing where it was).
- P5: the current workspace is the workspace of the selected chat, or the workspace row the user selected.

## Open questions

- Q1: what does the **New workspace** icon of the header do? A workspace with no chats has no session file, so it
  would need to be remembered by the app (a small config file of gentle-desk, not gentle-pi's files), and it needs
  a folder picker. If it is not wanted yet, remove the icon from the design.
- Q2: where does a user who does not code start a chat, if there is no independent group? One answer is a
  workspace the app creates for them in a fixed folder; that folder has to be chosen and checked against gentle-shell.
- Q3: collapsed or expanded groups: is the state remembered between runs?
- Q4: with 50 000 chats, the window served by Rust must be a flattened tree (workspace header rows and chat rows,
  collapsed groups skipped). This changes the shape of `sessions_window` and of the revision rule.

## Work, in order (each a reviewable slice)

- W1 (Rust, `crates/app` or `crates/runtime`): group sessions by header `cwd`, name and order them (P1-P3), with tests.
- W2 (Rust, `crates/backend`): carry a working folder from `ChatKey`/the open command to the spawner.
- W3 (contract, `apps/desktop/src-tauri`): a flattened-tree window and the intents for New session and new chat
  in a workspace; regenerate `bindings.ts`.
- W4 (web): the sidebar as a tree on top of the T4b data layer, with the disabled New session state.
- Not part of this work: the real window check against `01a` happens in W4, by the parent.
