# sessions-rename-delete: rename and delete a chat (backlog item 5)

Design for backlog item 5. Nothing here is implemented yet; the three decisions below are open and
waiting for the user, and no code is written until they are answered. All three were answered 2026-11-11 (see Log L3), and T1's live probe still gates any build.

## Specs

- S1: A chat's title is `name.or(first_message)`, where `name` is the LATEST `session_info` entry inside the pi session `.jsonl` at `<home>/sessions/--<encoded-cwd>--/<timestamp>_<id>.jsonl` (sessions-m4 S4, `crates/gentle-core/src/session.rs`), and gentle-desk never writes to the pi home today.
- S2: A rename must show the same title in the terminal and in the desk, so the name belongs where pi itself reads it.
- S3: Deleting one `.jsonl` heals the list by itself: the index drops missing files and the watcher classifies the removal.
- S4: The drawn board governs the UX: rename is inline in the row title, delete opens the confirm modal that names the real file path (`design/gentle-desk-v2/00-states-board.html`; row menu Rename/Delete in `01a-tree-chat.html`).
- S5: Any new `UiIntent` or `ViewState` field regenerates `apps/desktop/src/contract/bindings.ts` (specta), and the drift guard test fails until it is regenerated.

## Decisions (answered 2026-11-11)

D1 (S2) How a name is stored on rename:

- (a) Append a `session_info` line with the new name to the `.jsonl`: pi-native, and the terminal shows it too; the risk is that this is gentle-desk's first write to the user's pi data, and whether pi tolerates an external append while a chat process holds the file open is UNVERIFIED, so a live probe comes before any build. **RECOMMENDED**
- (b) Keep the name only in the gentle-desk config: pi never sees it, the desk shows it, the terminal does not match.
- (c) Rename the file itself: it moves the chat id, the live child and the selection, so the largest blast radius; not recommended.

CHOSEN (2026-11-11): (a), the session_info append, with T1's live probe before any build.

D2 (S3) The confirmation step before deleting a file:

- (a) Delete after one confirm, with no live-chat guard.
- (b) Refuse with a notice while that chat has a live working child; otherwise delete the file after a confirmation that names the real path. **RECOMMENDED**
- (c) Move the file to a trash folder instead of deleting it.

CHOSEN (2026-11-11): (b), refuse with a notice while the chat has a live working child, else confirm-and-delete naming the real path.

D3 (S5) Deleting the last chat of a DERIVED workspace (a folder known only from session `cwd`):

- (a) Let the workspace row disappear and show a notice; the explicit "remove workspace" stays a separate existing action. **RECOMMENDED**
- (b) Add the folder to the config so the row stays with no chats.
- (c) Refuse deleting the last chat of a derived workspace.

CHOSEN (2026-11-11): (a), the derived workspace row disappears with a notice.

## Tasks

- [x] T1 (S2, D1) Live probe: append a `session_info` line to a chat's `.jsonl` while a child holds it open and check that pi keeps the chat usable and shows the new name -- route: design -- DONE 2026-11-11, PASS 5/5 (Log L4)
- [x] T2 (S2, D1, S4) Rename slice: intent, inline row title, the write chosen by D1, `bindings.ts` plus drift guard -- route: design -- DONE 2026-11-11, PR pending (Log L5)
- [x] T3 (S3, D2, S4) Delete slice: row menu entry, confirm modal naming the real path, the live-child rule of D2, file removal -- route: design -- DONE 2026-11-11, PR pending (Log L6)
- [x] T4 (S5, D3) Workspace side-effect and the notices it carries -- route: design -- DONE 2026-11-11, PR pending (Log L6)

## Log

- L1: "Rename / delete sessions (needs two decisions: how pi stores a name, and a confirmation step before deleting a file)."
- L2: 2026-11-11, the user approved the design-first plan: "dale asi".
- L3: 2026-11-11, the user chose D1(a), D2(b) and D3(a) (all three the RECOMMENDED options).
- L4: 2026-11-11, T1 live probe PASSED 5/5 (`target/ui-check/probe-rename.ps1`, isolated home in target/probe-home, since deleted): a minimal synthetic session loaded by the real gentle-shell child with `--session` was appended externally with a `session_info` line while the child held it open; no file-sharing violation, the child stayed alive and answered `get_state` over RPC after the append, and the last `session_info.name` reads back exactly. Limits: the session carried no messages and no model turn ran, so "usable" here means the RPC child is alive and answering; pi also accepted a session file the app wrote, which is worth knowing.
- L5: 2026-11-11, T2 implemented on feat/session-rename: the backend append (one write, no create, separator guard, JSON-escaped name), the `rename_session` tauri command refusing while the chat's activity is `Working` (mirroring D2(b)), the drawn per-row menu with inline rename (Enter commits, Escape and blur cancel; blur cancels instead of the board's commit so a stray click cannot rename), Delete drawn disabled, `ViewState.sessions_notice` added and bindings regenerated with the drift guard green. Residual window documented: the working check and the append are not atomic, a far smaller hazard than the turn-length one the rule removes.
- L6: 2026-11-11, T3+T4 implemented on feat/session-delete plus the user's design fix for the rename row: the backend removal deletes exactly the one named path behind a SessionRemovals seam (Missing with its own line); the `delete_session` command is a no-op without the dialog's `confirmed`, refuses Working with a notice and never kills a child; the confirm modal names the real path and sends confirmed once; the vanished derived workspace is noticed only when it was the current one. Design fix (user screenshot): the actions slot reserves 44px with the timestamp fading out on hover/focus/menu-open (it no longer collides with "now"), and the menu opens in a portaled layer right-aligned inside the sidebar, unreadable-clipping fixed; pinned by ChatRowMenu.test.ts with a 2-failure differential against the pre-fix CSS. The delete of an IDLE child's file may answer access denied on Windows, surfaced as the sessions notice (the documented D2(b) window).
