# workspace-removal: removing a workspace, and deleting its chats

Remove a workspace from gentle-desk with a clean, strict flow: a workspace that still
holds chats cannot be removed until they are deleted, the deletion is trash-first, and
the dialog follows the maintainer's destructive-dialog anatomy. Source decisions:
`odd/tasks/tauri-settings-roadmap.md` section `## 0.2.x: removing a workspace, and
deleting its chats` plus the 2026-11-16 rulings below. Chat data lives in pi's session
store (`<agent-dir>/sessions/`, indexed by workspace path — gentle-desk maps via the
session index, not a folder-name encoding).

## Specs

- S1 (GATE, maintainer 2026-11-16): "no deberiamos permitirle al usuario quitar un
  workspace si aun tiene chats, debe quedar limpio para poder eliminar o quitarlo para
  que tenga logica". So: a workspace with 0 chats removes freely (forgets the
  bookmark, touches nothing on disk); a workspace with N ≥ 1 chats offers ONE
  destructive path — delete its N chats, then remove. There is no opt-in
  checkbox; the recorded 2026-11-14 opt-in shape (`[ ] Also delete those N chats
  (M MB)`, primary button becomes `Remove and delete 2 chats` only when checked) is
  SUPERSEDED by this gate. In the N ≥ 1 dialog the destructive button is
  `Delete N chats and remove`, last, red, scope-labelled — WITHOUT the parenthetical
  (maintainer, canvas comment on the artboards): the size lives in the consequence
  line and, where shown, the disclosure, per S2.
- S2 (counts and sizes, roadmap): "Size is therefore shown in megabytes next to the
  count, not only a count" — measured baseline 12 chats / 36 MB, a workspace of 2
  chats / 1.7 MB. The count and byte size of a workspace's chats come from the
  session index (pi's store), surfaced wherever the removal offer is drawn.
- S3 (dialog anatomy, roadmap verbatim): "the title is the question (`Remove this
  workspace?` / `Delete this chat?`); one consequence line in the user's words, with
  the scope in bold (`Delete 37 chats`), never the words 'chat file'; NO inner card
  and no boxed path; the destructive button is last, red, and labelled with its
  scope; actions right-aligned. Where a path is shown, it is plain monospaced text
  with the middle elided (`C:\…\gentle-desk`), or behind a `Show file paths`
  disclosure. The workspace path IS the identity and belongs in the first sentence; a
  chat's internal `<timestamp>_<id>.jsonl` name is noise and moves to a row-menu
  `Copy file path`." The existing delete-chat dialog is upgraded to this same
  anatomy.
- S4 (trash-first, maintainer 2026-11-16: "papelera, como pi"): deleting chat files
  moves them to the OS trash first (Rust `trash` crate, native Windows recycle-bin
  API — pi's own terminal delete tries `trash` then falls back to `rm`); unlink is
  the fallback when the trash move fails.
- S5 (teardown first, roadmap): "removing a workspace that has an open chat must
  first close that chat's child process and drop its cached session (process-per-chat,
  sessions-cache, refresh-watcher) ... That teardown is the real work of the safe
  half and needs its own spec before any code." Order: close the workspace's open
  (idle) children → then delete the chat files → then forget the bookmark. A
  WORKING chat of that workspace refuses the whole removal with the existing
  working-chat wording, nothing deleted.
- S6 (design first, maintainer + roadmap): "artboard en gentle-design antes de tocar
  Svelte" / "Visual work goes through the gentle-design canvas before
  SettingsDialog/dialogs change, not by hand in Svelte."
- S7 (bilingual, 0.2.0 standing rule): new copy is born bilingual — web dictionaries
  en/es with parity-pinned flip tests; new Rust-drawn notices enter
  `crates/gentle-core/src/i18n/messages.rs` in both columns.
- S8 (row menu, deviation recorded): the workspace header row gains a `...` menu
  whose destructive item offers removal (D5 previously kept the row menu-free; the
  circle-plus stays). English rule: what the D5 note pinned was "no menu", this
  feature is the maintainer's decision to add one.

## Open (decided here, recorded)

- Orphan chats (folder deleted outside the app) are OUT of scope; the maintainer
  deferred the `Unused chats` Settings view ("eso de huerfanos le veo mal"). The gate
  makes in-app orphans impossible.
- Version number for the release carrying this (0.2.1 patch vs 0.3.0 minor) is the
  maintainer's call at close; the roadmap calls this section 0.2.x.

## Tasks

- [x] T1 (S6) gentle-design: artboard(s) of the removal dialog — DONE. Three
      directions written (`01-question-plain`, `02-evidence-disclosure`,
      `03-one-quiet-line`), each with the 0-chat and N-chat states; design-system
      sync re-run first. Canvas comment senflglr applied (button label without the
      parenthetical). The maintainer chose **02-evidence-disclosure (B)** for
      implementation — the handoff's chosen artboard; 01 and 03 are alternatives.
- [x] T2 (S2, S4, S5, S7) Rust core — DONE, commit 8713e43. Writer report: S4
      complete (session_delete is trash-first with unlink fallback, `TrashMoves`
      seam + `OsTrash`, `trash 5.3.0` dep), S2 complete (SessionEntry.bytes from
      the cached size; the offer is its own command `workspace_removal_offer`
      returning `WorkspaceRemovalView { key, path, label, chats, bytes: f64 }` —
      deviations recorded in L8), S5 complete (`workspace_remove::remove`: refuse
      while a workspace chat is WORKING → close open idle children via
      CloseProcess → bulk delete trash-first → forget the bookmark; stop on first
      refused delete keeps the bookmark), S7 complete (`remove_workspace_working_refusal`,
      `workspace_removal_failed` in both columns, asserted en+es). TDD: RED
      observed (E0432/E0425/E0599/E0609, specta BigInt ban moved bytes→f64);
      GREEN: workspace 106 suites 0 failed, clippy -D warnings, fmt, bindings
      drift guard byte-identical with and without regen.
- [x] T3 (S1, S3, S7, S8) web — DONE, commit 3721764. Writer report: S8 (the
      header row's `...` menu, its one destructive item, circle-plus intact, D5
      note rewritten), S1 (RemoveWorkspaceDialog per artboard B: 0-chat = plain
      Remove, N-chat = one destructive `Delete N chats and remove` sending
      `remove_workspace(path, true)`; no checkbox), S3 (shared anatomy in both
      dialogs: question title, bold-scope consequence, plain elided mono path —
      no boxed `.dlg code`, no inner card, right-aligned actions; `Copy file
      path` joins the chat row menu), S7 (all new copy bilingual with parity +
      flip tests). TDD: RED 8/8, then GREEN 10/10; vitest 664/664 in 108 files,
      eslint 0, svelte-check 0. Deviations in L9.
- [x] T4 (close) — DONE, split: (a) full battery green on the integrated tree
      (cargo 1268/0, clippy -D warnings, fmt, bindings drift guard byte-identical
      with regen, vitest 664/664 then 668/668 after the fix batch, svelte-check 0,
      eslint 0); (b) the maintainer's real-window check on the maintainer's
      machine 2026-11-16 (npm run tauri dev): "ya probé y funciona bien" — the
      removal dialog in both states, the gate, the teardown, the trash and the
      row-menu copy path exercised; (c) the maintainer's decision on the review
      findings: the three UX findings fixed pre-release (L11), the rest backlog;
      (d) version 0.3.0 in the three carriers + the root Cargo.lock (the
      maintainer's ruling stands: capability jump, not a patch).

## Log

- L1 (user, 2026-11-16, session start): "Siguiente feature después del release:
  remoción de workspaces con borrado opt-in de chats (roadmap, sección ## 0.2.x:
  removing a workspace, and deleting its chats, commit 97c958a): diálogo sin card
  interna, check [ ] Also delete those N chats (M MB), teardown de hijos primero, y
  artboard en gentle-design antes de tocar Svelte."
- L2 (user, 2026-11-16): on the trash decision, asked "cuando se borran en la
  terminal con ctr+d y enter, a donde se van?" — evidence: pi's session-selector
  tries `trash` (spawnSync) with an `rm`/unlink fallback. Decision: "papelera, como
  pi" (S4). Orphan view still open at this point.
- L3 (user, 2026-11-16): "para que haya logica, no deberiamos permitirle al usuario
  quitar un workspace si aun tiene chats, debe quedar limpio para poder eliminar o
  quitarlo para que tenga logica. porque eso de huerfanos le veo mal" — parent
  surfaced the trade-off (gate makes destroying history mandatory for removal and
  loses the temporary-removal case; opt-in keeps deletion optional and re-adding the
  path restores the chats). Maintainer chose the GATE with full knowledge; the
  opt-in shape of L1 is superseded (S1), the orphan view is deferred (S2 note).
- L4 (parent, 2026-11-16): maintainer also ruled trash-first (pi's own terminal
  delete tries `trash` then falls back to `rm`; gentle-desk does the same with the
  Rust `trash` crate, S4) and deferred the orphan-chats Settings view.
- L5 (parent, 2026-11-16): design-system sync re-run before designing —
  `design/system.md` + `design/system.html` rewritten from
  `apps/desktop/src/{theme,app}.css` (maintainer's OK). Removal-dialog artboards
  written: `design/workspace-removal-dialog/` — 01-question-plain (A),
  02-evidence-disclosure (B), 03-one-quiet-line (C); each shows the 0-chat and the
  N-chat states. Canvas opened for the maintainer's review; the chosen artboard
  feeds T3.
- L6 (maintainer, 2026-11-16, canvas comment senflglr): "quita lo del paréntesis, en
  esta vista y en las otras" — the destructive button reads `Delete 2 chats and
  remove` in all three artboards; the parenthetical size stays in the consequence
  line and the disclosure (S2). S1 rewritten accordingly.
- L7 (maintainer, 2026-11-16, design handoff): implement
  `design/workspace-removal-dialog/02-evidence-disclosure.html` — artboard B is the
  chosen one; T3 maps its parts to the real stack (bits-ui dialog + the house
  tokens of design/system.md), never copying the artboard HTML. T2 (Rust core)
  runs first so the dialog draws on real bindings.
- L8 (parent, 2026-11-16): T2 delivered by the writer with two approved
  deviations: (a) the offer is its own command `workspace_removal_offer` reading
  the index at offer time, NOT a `WorkspaceRowView.chat_bytes` field — closer to
  S2's "the size comes from the session index" and it avoided touching
  SessionItem literals outside the surfaces; the authorized sessions.rs/state.rs/
  tree.rs/window.rs paths were not needed. (b) `bytes` is `f64` on the wire
  (Specta forbids exporting 64-bit integers; the contract's measured-value
  channel). Slice volume ~465 source + ~860 test lines, above the ~400 guide
  because flow and exposure were authorized together; the boundary is recorded
  here and the web slice follows as T3. Path widening mid-task was approved by
  the parent (contract/bindings, commands/lib registration, workspace_remove.rs).
- L9 (parent, 2026-11-16): T3's two approved deviations: (a) the disclosure lists
  chat TITLES with the aggregate size in the summary — the offer carries no
  per-chat bytes, so no row sizes are drawn (a folded or half-read page draws the
  consequence line alone, pinned by a test); per-row sizes are a future
  refinement of the contract if the maintainer asks. (b) The removal command
  wrappers live in `sidebar/removal.ts` because `bridge.ts` was outside the
  surfaces; moving them later is call-site-neutral. Open point carried to T4's
  real-window review: the delete-chat dialog still shows its elided path beside
  the new `Copy file path` row-menu item — dropping the path is a one-line
  removal if the maintainer prefers S3's letter.
- L10 (parent, 2026-11-16): the native ordinary review ran on the slice
  (lineage review-74c37722100c058e, candidate = ce23434..HEAD, 33 paths, 2706
  lines, tier medium, correction budget 200): lenses risk/resilience/reliability
  (3 host-relay runs, ~$0.058: glm-5.3-flash ×2, deepseek-v4.1-flash) + provider
  refuter (glm-5.3-flash, ~$0.0025; the first refuter output was malformed JSON
  and refused at admission — the provider reoffered the slot and the second run
  closed). Verdict: APPROVED, authority burned (acknowledgement v1), delivery
  ordinary-repository-policy. 12 advisory findings, ALL non-blocking (none
  opened a correction; the CRITICAL R3-001 "unconfirmed data loss" was REFUTED):
  risk R1-01 (working-gate TOCTOU), R1-02 (chats created between offer and
  confirm get deleted), R1-03 (partial-refusal notice understates what was
  destroyed); reliability R3-002/R4-dialog (no actions when the offer read
  fails), R3-003/R4-err-path (Err closes the dialog silently), R3-004 (stale
  deleted row until refresh heals), R3-005 (the real-trash test is
  environment-dependent); resilience R4-serial-close-latency (2s grace per
  child, no progress on the dialog). Recorded as later work, never a reopen of
  this review; the maintainer picks which ones ride a pre-release fix batch.
- L11 (parent, 2026-11-16, pre-release fix batch, maintainer-approved): the
  writer fixed exactly three of L10's advisory findings, RED observed for all
  three, commit fe24b33. R3-002/R4-dialog: a failed/refused offer read now
  renders a bilingual danger line + the normal action row (Cancel, no invented
  count, nothing sent). R3-003/R4-err-path: `removeWorkspace` propagates the
  Rust `Err` and call failures; the dialog stays open with a second bilingual
  danger line, re-enables the button, never re-sends alone. R1-01/TOCTOU: the
  delete loop re-checks `WorkingChats::is_working` before EACH deletion and
  stops the whole flow with the cataloged wording, bookmark kept. Validation:
  vitest 668/668, svelte-check 0, eslint clean, workspace_remove 11/11, clippy
  -D warnings, fmt clean. The remaining findings (R1-02, R1-03, R3-004, R3-005,
  R4-serial-close-latency) stay backlog.
- L12 (parent, 2026-11-16): the fix batch's own native review — lineage
  review-f95cbc6c7bc257ee, candidate = cd4b96b..HEAD (10 paths, 304 lines, tier
  medium), lenses risk/resilience/reliability, 3 host-relay runs ~$0.033 —
  APPROVED on the last admitted capture (no refuter required), authority
  burned, delivery ordinary-repository-policy. 5 advisory findings, all
  non-blocking: R1-caller-audit (audit removeWorkspace callers — only the
  dialog calls it), R1-partial-delete-refusal (the mid-flow refusal may follow
  already-deleted chats; verify the wording or re-check before the first file
  at the real-window check), R1-residual-toctou (the narrowed but unclosed
  race, backlog), R4-001 (stale sidebar rows for already-deleted chats until
  the next rescan, accepted fix-forward), R4-002 (the dialog's catch shows the
  generic line; whether the backend push carries the Err reason is for the
  real-window check).
- L13 (maintainer, 2026-11-16): "ya probé y funciona bien" — the real-window check
  approved (npm run tauri dev, job log clean except Chromium teardown noise). The
  maintainer also accepted the parent's ruling chain: fix batch of the three UX
  findings, backlog for the rest, version 0.3.0.
