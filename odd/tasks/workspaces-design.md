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
- S7 (New workspace): "es para elegir una carpeta nueva para el workspace, agrego la carpeta y se me abre un nuevo chat en ese workspace, si el usuario no usa ese chat ni ese workspace, permanece sin chat hasta que el usuario cree uno pero ya forma parte del workspace".
- S8 (no free chats): "que se elimine eso de los chats libres, si o si debe escoger un workspace una carpeta existente o podemos darle la opcion de crear un workspace nuevo (crear una carpeta nueva), el usuario elije la ubicacion de este workspace".
- S9: asked whether the collapsed or expanded state of each folder is remembered between runs, the answer was: "si".

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

- D1 (S1, S6, S8): one model, the workspace. No "Chats" section, no independent-chat group, no free chats. To
  start a chat the user must have a workspace: an existing folder, or a new folder created in a location the
  user picks.
- D2 (S2): the top button is **New session**; it opens a new chat in the folder of the current workspace.
- D3 (S3): each workspace row has a new-chat icon (circle plus) that shows on hover of the whole workspace
  section or on keyboard focus; same effect as D2 for that workspace.
- D4 (S4): with no current workspace, **New session** is disabled. In the design it uses `aria-disabled` and a
  visible reason, so a keyboard or screen-reader user can reach the reason.
- D5 (S5): no workspace menu is drawn. It exists only if it ever has options.
- D6: open and closed folder icons show the expanded state; there is no chevron (`aria-expanded` is kept).
- D7 (S7): **New workspace** (header icon) adds a folder as a workspace and opens a new chat in it. If the user
  never uses that chat or workspace, the workspace stays in the list with no chat until the user creates one.
  A workspace therefore exists independently of having chats.
- D8 (S9): the collapsed or expanded state of each workspace is remembered between runs.

Proposed by the assistant (not yet confirmed by the user):

- P1: a workspace is identified by the canonical folder path (on Windows, case-insensitively), so the same folder
  added twice, or already present because of its sessions, is one workspace. Its name is the last path segment;
  when two workspaces share it, show enough of the parent path to tell them apart.
- P2: the list of workspaces is the union of (a) the folders the user added, kept by gentle-desk, and (b) the
  `cwd` values found in session headers, so the history made from the terminal shows up. Added folders are never
  hidden; derived ones that are temporary folders are hidden by default.
- P3: order workspaces by last activity (the newest session, or the time it was added if it has none); chats
  inside by last activity.
- P4: the sidebar search keeps searching all workspaces at once.
- P5: the current workspace is the workspace of the selected chat, or the workspace row the user selected.
- P6: the app keeps its own small config file (added workspaces, collapse state) apart from the session-index
  cache, which is only a cache and can be deleted. Suggested place: `%APPDATA%gentle-deskconfig.json`.
  The cache lives in `%LOCALAPPDATA%gentle-desksession-index.json` today.

## Open questions

- Q1: what UI does **New workspace** have? Option A: a two-item menu, "Choose an existing folder" and "Create a
  new folder". Option B: one native folder dialog (the Windows dialog already has a "Make new folder" button).
  Option B is less to design; option A states the two paths the user named.
- Q2: the empty sidebar when no workspace exists yet (first run). New session is disabled there, so the sidebar
  needs a visible way forward: an empty state that points to New workspace. It is not drawn yet.
- Q3: an added folder that no longer exists on disk: show it with a visible reason and disable its new-chat
  icon? gentle-desk already surfaces a dead working folder when a chat starts (`crates/app/tests/chat_stderr.rs`).
- Q4: the native folder dialog comes from the Tauri dialog plugin. Tauri is built with its default features off
  (`common-controls-v6` broke plain test binaries). It must be checked that the dialog works without it.
- Q5: with 50 000 chats, the window served by Rust must be a flattened tree (workspace header rows and chat
  rows, collapsed groups skipped). This changes the shape of `sessions_window` and of the revision rule.
- Q6: "create a new folder" writes to the user's disk: it must refuse an existing non-empty target and report the
  error as a notice, never overwrite.

## Work, in order (each a reviewable slice)

- W0 (Rust, `apps/desktop/src-tauri` or `crates/runtime`): the app config file (P6) with atomic writes: the
  added workspaces and the collapse state, with tests, including a corrupt file (kept aside, not overwritten).
- W1 (Rust, `crates/app` or `crates/runtime`): build the workspace list (P1-P3) from the config plus the
  session headers, name and order it, with tests.
- W2 (Rust, `crates/backend`): carry a working folder from `ChatKey`/the open command to the spawner.
- W3 (contract, `apps/desktop/src-tauri`): a flattened-tree window, the intents (New session, new chat in a
  workspace, add workspace with the folder dialog, collapse), and the disabled New session state; regenerate
  `bindings.ts`.
- W4 (web): the sidebar as a tree on top of the T4b data layer, the empty state (Q2), and the New workspace UI
  (Q1), checked in a real window against `01a` by the parent.
