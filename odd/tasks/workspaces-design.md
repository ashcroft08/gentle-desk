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
- The process layer can start a child with a working folder (`crates/backend/src/process.rs`, option `cwd`), and
  since W2 the chat host carries it: `ChatHost::open_in(key, folder)` refuses a folder that cannot host a child
  (`crates/backend/src/chat_host/folder.rs`) and starts the child there through `Spawner::spawn(key, folder)`, which
  sets `cwd` in `ProcessSpawner::options_for` (`crates/backend/src/chat_host/spawner.rs`).
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
- P4 (DROPPED in L15): the sidebar search keeps searching all workspaces at once. The user removed the
  search (`tauri-migration-m19.md` L15), so the tree has no query and the window command carries none.
- P5: the current workspace is the workspace of the selected chat, or the workspace row the user selected.
- P6: the app keeps its own small config file (added workspaces, collapse state) apart from the session-index
  cache, which is only a cache and can be deleted. Place, as W0 built it: `<roaming app data>/gentle-desk/config.json`
  (`crates/runtime/src/app_config/paths.rs`).
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

## W1 decisions

What the workspace builder decided, beyond the specs above:

- The builder is a pure module of `crates/runtime` (`crates/runtime/src/workspaces.rs`), not of `crates/app`:
  `gentle-runtime` depends on `gentle-app`, so a builder in `gentle-app` that reads a `SessionItem` would close a
  cycle. It takes the config and the session items and returns the list: no file, no environment, no clock.
- The list is the union of the folders the config holds and the `cwd` values of the session headers, matched by
  `folder_key`, so one folder is one workspace whatever spelling each side reports. The row shows the config's
  spelling when both sides have the folder. A session with an empty or blank `cwd` belongs to no workspace: none
  is invented for it, and `WorkspaceList::unplaced` counts it.
- Temporary folders are not hidden: no rule tells a temporary folder from a lasting one yet, so hiding any of them
  would be a guess.
- A workspace with no chat has nothing to be ordered by, and the config holds no "added at" time: workspaces with
  no chat follow the ones with activity, in the order the config lists them. Ties among workspaces that have
  activity are broken by `key`.

## W3a decisions

What the tree window of the bridge decided, beyond the specs above:

- The window serves a ROW UNION, not a session list: `SessionTreeRow` is a serde enum tagged by
  `kind`, with a `Workspace` variant (key, label, path, name, collapsed, chat_count, available,
  reason) and a `Chat` variant carrying the fields of `SessionRowView`. The web side reads one flat
  array and draws headers and chats at the same fixed row height.
- `SessionsWindow.total` counts the flattened rows: workspace headers included, chats of a collapsed
  workspace excluded, so it is exactly the number of rows the sidebar can scroll through.
- A collapsed workspace still serves its header - with `collapsed` and its full `chat_count` - and
  none of its chats, so folding a group never changes what the group says about itself.
- The search is gone from the contract (L15, P4 dropped): `sessions_window` takes `offset` and
  `length` only. `crates/app` keeps `SetSessionQuery` for the iced app until `crates/ui` is deleted.
- Whether a workspace folder still exists (`available`, `reason`) touches the file system, so it is
  asked per served page and only for the header rows that page holds: at most MAX_WINDOW (200)
  probes, never one per workspace of the whole list.
- The flattened tree is cached by the holder's revision: a window call reuses the tree until the
  session list or the config moves, and the rebuild reads a snapshot of the state and of the config
  outside every lock.
- `sessions_revision` also moves when the config snapshot is replaced
  (`AppStateHolder::replace_config`), because the tree is built from the config too.
- The app config is read once at startup, outside every lock, and the notice of a damaged file reaches
  the web side as the new optional `ViewState.config_notice` field. A run with no config path holds the
  empty config in memory, and this slice never writes a config file.

## W3b decisions

What the bridge's workspace intents decided, beyond the specs above:

- Two intents were added to the contract and nothing else: `NewChatInWorkspace { path }` and
  `SetWorkspaceCollapsed { path, collapsed }`. The web side decides which workspace is the current
  one (P5 stays a view-local selection) and names it in the intent.
- `NewChatInWorkspace` is the intent of BOTH the header's New session button (D2) and a workspace's
  circle-plus icon (D3), because both mean the same thing: a new chat in one named folder. It folds
  to the existing `OpenNewChat` path, so the chat is a chat of the per-chat model; the legacy
  single-child `NewChat` is untouched by this slice.
- The folder never enters `crates/app` (S2, unchanged reducer and state types). It waits in a
  pending-folder table keyed by the `ChatId` the reducer allocated: the intent path sets it before
  the `OpenProcess` command is run, the executor takes it and calls `ChatHost::open_in(key, folder)`
  (`ChatRuntime::execute_in`). An entry is removed when it is taken, so a folder can never reach a
  second open, and an open nobody set a folder for inherits the process folder as before.
- A folder that is not there, or that is a file, never starts a child: the host refuses before
  anything is reserved, and the refusal travels the path a failed open already takes
  (`Outcome::OpenFailed`, `Input::ChatFailed`), which ends the opening phase and leaves one error
  notice naming the reason. No new error surface was invented for it.
- `SetWorkspaceCollapsed` is not a reducer intent: it edits the app config snapshot
  (`AppConfig::set_collapsed`) and installs it, which moves `sessions_revision`, so the tree is
  rebuilt and the folded workspace serves its header only. The collapsed state is the config's
  because it must survive the run (D8, S9).
- A workspace that exists only because a session lives there has no state in the config to remember
  (the limit W1 recorded), so it is added with `AppConfig::add_workspace` the first time it is
  folded, and folded right after. Adding a workspace through the UI (D7, the New workspace dialog)
  is W3c, not this slice.
- A change to the config is ONE step (`AppStateHolder::update_config`): the read, the edit and the
  install of the new snapshot all happen under the config lock, so two folds of two different
  workspaces at the same time cannot lose each other, and the revision moves inside the same step.
  Whether a folder is a workspace at all is decided BEFORE that step, because the answer reads the
  tree and the session list: no other lock is ever taken inside the config lock.
- One thread owns the config file for the whole run and always writes the latest snapshot. The lock
  is never held for disk I/O: the numbered snapshot leaves for the writer's own thread, which
  drains its queue to the newest one and drops any snapshot numbered below the one already written,
  so a caller descheduled between its edit and its save cannot undo a newer state. With no config
  path there is no thread and nothing is written; the change still lives in memory for that run.
- A save that fails does not panic and does not undo the change: the run keeps the value it holds and
  the reason reaches the user as `ViewState.config_notice` (the field W3a introduced for a damaged
  file at startup now carries a failed save too). A later successful save does not clear that line
  in this slice.
- An unknown or empty path changes NOTHING: no workspace is added, no revision moves, no file is
  written and no chat is opened.

## Q4 answer: the native folder dialog (evidence, not a decision)

Q4 asked whether the Tauri 2 dialog plugin works with tauri built `default-features = false`
(the `common-controls-v6` problem of `tauri-contract-proof.md`). It was wired, built, and measured
against tauri 2.12.5 / tauri-plugin-dialog 2.8.4, then REVERTED: the wiring cannot be guarded by a
test binary, and which picker W3 uses is a decision to make before W3 (options at the end).

What was measured:

- `cargo tree -e features -p gentle-desk-app -i tauri` before and after adding the plugin: the
  tauri feature set is IDENTICAL (`specta`, `tauri-runtime-wry`, `webkit2gtk`, `webview2-com`, `wry`).
  The plugin does not re-enable tauri's default features: its own `tauri` dependency is
  `default-features = false`.
- New Cargo.lock packages: exactly four, `rfd 0.17.2`, `tauri-plugin 2.7.2`, `tauri-plugin-dialog
  2.8.4`, `tauri-plugin-fs 2.7.1`. Nothing pulling ashpd, gtk or glib: the plugin's `default`
  feature is `gtk3` (a Linux-only rfd backend), so it is turned off on a Windows-only app.
- `cargo build -p gentle-desk-app`: success.
- `cargo test -p gentle-desk-app` with the plugin registered in `run()`: all 14 integration test
  binaries and both unit binaries start and pass, no 0xc0000139. Nothing in the suite reaches
  `run()`, so nothing in the suite links the plugin's code.
- No capability permission is needed for Rust-side use: `DialogExt::dialog` reads the plugin's
  managed state, while capabilities gate only the plugin's IPC commands
  (`dialog:allow-open/save/message`). `capabilities/default.json` was left as `core:default`.

The blocker, exact:

- The plugin's `init()` registers its IPC handler, which includes `commands::message`, which pulls
  rfd's `MessageDialog` and with it `TaskDialogIndirect` from comctl32 v6. rfd's own
  `common-controls-v6` feature is enabled UNCONDITIONALLY by the plugin's `rfd` dependency, so it
  cannot be switched off from this crate.
- A test binary that only names `tauri_plugin_dialog::init::<tauri::Wry>()` (and one that builds a
  `tauri::test::mock_builder()` app with the plugin) loads with
  `STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139)`: `objdump -p` on the test exe shows an import of
  `TaskDialogIndirect` from `comctl32.dll`, and test exes carry no application manifest. Enabling
  tauri's empty `test` feature (`features = ["test"]`, adds no crate) does not change this.
- The GUI exe is not affected: `target/debug/gentle-desk-app.exe` also imports
  `TaskDialogIndirect`, but it embeds the `Microsoft.Windows.Common-Controls` version `6.0.0.0`
  assembly manifest, so the import resolves there.
- A test binary that reaches only the Rust-side picker path (`app.dialog().file()` inside a lib
  function, no `init()`) links and RUNS: it imports comctl32 without `TaskDialogIndirect`.

Not verified here: that the picker actually opens and returns a folder in a real window. That is the
parent's real-window check; this slice proved the dependency wiring and its link-time cost only.

Decision for W3 (choices, none implemented):

- A: drop the Tauri plugin and call `rfd` 0.17 directly with `default-features = false`, whose file
  dialog is the Win32 `IFileDialog`; only its `common-controls-v6`-gated message dialog imports
  `TaskDialogIndirect`, so no test binary gains the import. Source reading only; not yet linked
  and measured.
- B: keep the plugin and give test binaries the comctl32 v6 manifest (build-script
  `cargo:rustc-link-arg-tests=/MANIFESTDEPENDENCY:...`), documented with the reason.
- C: keep the plugin and open the picker from the web side (`@tauri-apps/plugin-dialog` plus a
  `dialog:allow-open` capability): the plugin's code is then only linked through `run()`, which no
  test binary reaches, at the cost of an npm package and an IPC surface.

## W3c decisions

What the two "New workspace" paths decided, beyond the specs above:

- Q4 is answered by the user as OPTION A: the Tauri dialog plugin is not used and the app depends on
  `rfd` 0.17 with `default-features = false`. Measured on this branch: `cargo tree -p
  gentle-desk-app -e features -i tauri` shows the same tauri features as before the dependency
  (`specta`, `tauri-runtime-wry`, `webkit2gtk`, `webview2-com`, `wry`; no `common-controls-v6`), the
  new lockfile package is `rfd` alone, and a test binary that links the real picker starts and passes
  instead of failing with `0xc0000139`. `objdump -p` of that binary shows comctl32 imports
  (`DefSubclassProc`, `RemoveWindowSubclass`, `SetWindowSubclass`) and no `TaskDialogIndirect`; the
  app exe imports comctl32 the same way, with no `TaskDialogIndirect` either, and still embeds the
  common-controls v6 assembly manifest.
- The picker is a seam, not a call: `picker::FolderPicker` (`pick_folder(title) -> Option<PathBuf>`),
  held by the bridge as `Arc<dyn FolderPicker>` and taken by the flows as `&dyn`. The real
  implementation is rfd's `IFileDialog`; every test injects a fake. A dialog can stay open for
  minutes, so it never runs on the window's thread and never under a lock: both commands run their
  whole flow on a blocking thread (`tauri::async_runtime::spawn_blocking`) and are `async`.
- The dialog is MODAL to the main window: `NativeFolderPicker` keeps the window the setup step hands
  it (`app.get_webview_window("main")`) and passes it to rfd's `set_parent`, which sets the dialog's
  owner from the window handle. It was little code, not a guess: rfd 0.17 and tauri 2 both take the
  handle from `raw-window-handle` 0.6.
- The two paths are COMMANDS (`addWorkspace`, `createWorkspace`), not `UiIntent` variants, because
  they do not fold in one step: both wait for a human and both touch the disk, then answer the same
  `ViewState`. The web side awaits them; neither starts a new chat through the reducer directly.
  `createWorkspace` takes the name; the parent comes from the picker.
- "Create a new folder" validates the name FIRST, so a name Windows would not accept never opens a
  dialog. The name is trimmed; it must not be empty, must not end with a space (checked on the typed
  text, because Windows would drop it silently), must not contain a path separator or `..`, must not
  contain any of `< > : " | ? *`, must not end with a dot, and must not be a device name (`CON`,
  `NUL`, `COM1`..., also with an extension).
- Q6: the target is created with `create_dir`, never `create_dir_all`: the parent comes from the
  picker and must exist, so a vanished parent is a refusal and not a chain of new folders. An
  existing EMPTY folder is accepted as the workspace and nothing is written into it; an existing
  folder that holds anything, or a file of that name, is refused and nothing is written.
- Both paths end in the same place (`workspace::adopt`): the folder is added to the config with
  `AppStateHolder::update_config` (idempotent for a folder already there, stored in the canonical
  spelling the config writes) and a new chat opens there through the same `NewChatInWorkspace` fold
  W3b built, with the folder waiting in the pending table. The workspace stays whatever becomes of
  that chat (D7).
- A refusal has no reducer path: `crates/app` owns the chat and event notices and this slice does not
  change it. A cancelled dialog changes nothing; a refusal sets the new shell-owned
  `ViewState.workspace_notice` with one line of plain text, moves no revision and writes nothing, and
  the next workspace action that works clears it. It is NOT `config_notice`, which says what became
  of the config file.
