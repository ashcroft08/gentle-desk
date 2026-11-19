# composer-dock: the status row, the chips and file attachments

The maintainer saw two things in the other desktop app and asked for them on
gentle-desk's own components: the row that shows what the agent is doing while it
works, and the composer's chips (context, folder, branch, mode, attachments). This
document exists BEFORE any code so the data question is answered first: three of
those chips are cheap, one needs a new command, and one cannot be built at all.

## Specs (verbatim)

- S1: "quiero agregar mas cosas de diseño hay podemos? Podemos mostrar el contexto
  en ese diseño y mira como se muestra cuando piensa y el workin, tambien lo de la
  sesion si esta normal/YOLO/Super yolo".
- S2: "tambien quiero hacer esto de que muestre en que workspace estamos, solo el
  nombre de la carpeta no toda la direccion, tambien la rama en la que estamos si es
  que hay github si no no se muestra nada, quiero implementar este mismo diseño
  usando nuestros componentes y colores".
- S3: "hay como un clip que abre el explorador de archivos para insertar archivos,
  al final ya tenemos lo de arrastrar imagenes, quiero hacer que tambien se pueda
  con archivos, como al final lo que se manda es la direccion".

## What the data says (read-only exploration, 2026-11-18)

| Chip | Status | Note |
| --- | --- | --- |
| Context percent + bar | AVAILABLE | `ContextUsageView` already carries percent AND window |
| Working state + elapsed | NEEDS PLUMBING | the mark exists; no timestamp reaches the view |
| Folder (last segment) | NEEDS PLUMBING | `SessionItem::project()` exists per session; the foreground chat's is not in the view |
| Git branch | NOT AVAILABLE | nothing reads git today; needs a command |
| Permission mode | AVAILABLE, unwired | over `/gentle:desktop` (`get_permission_mode`); read is safe, WRITE is unverified and upstream refuses YOLO outside a TUI |
| Output tokens / cost | NOT AVAILABLE | pi's RPC sends neither; drawing them would be inventing a number |

## Rulings taken from that table

- R1: the mode is drawn as a READ-ONLY label, never a switch. Upstream refuses to
  grant YOLO outside a TUI on purpose, so a control here would be a lie (same
  criterion as the "two methods" ruling for profiles: the terminal keeps what only
  the terminal can do).
- R2: no token or cost chip. Not "later": not at all, until pi exposes it.
- R3: with no repo, the branch chip is NOT drawn - no gap, no dash, no "detached".
- R4: the folder shows the LAST SEGMENT only; the full path lives in the tooltip.
- R5: an attached file from outside the workspace says so, because the child may not
  be able to read that path.
- R6: the clip inserts the file's PATH. Images keep travelling as images; a file
  travels as a path the agent opens with its own tools.

## Tasks

- [ ] T1 (S1, S2) the status row: the working state with an elapsed time and the
  project name (last segment only), sharing ONE anatomy with the reasoning row.
- [ ] T2 (S1, S2) the composer chips: context percent with its bar (reusing what
  exists), the folder name, and the branch - the branch behind a new Tauri command
  that answers "no repo" rather than failing.
- [ ] T3 (S1) the permission mode, read-only, behind `supports()`.
- [ ] T4 (S3) attachments: a picker command plus the existing drag-and-drop, chips
  in the composer, the outside-workspace warning, and paths inserted into the
  message.

## Log

- L1 (2026-11-18, the maintainer's words in full, one line each of S1-S3 above).
- L2 (2026-11-18, exploration): the table above is the answer to "can we"; it is why
  this document rules out tokens and a mode switch instead of drawing them.
- L3 (2026-11-18, design first): the artboard is
  `design/composer-dock/01-dock.html`. Nothing compiles until it is approved.

## Log (cont.)

- L4 (2026-11-19, T1 and T2b landed): the status line (spinner + elapsed counted in the
  web + the folder name), the context rail above the composer, the regrouped row
  (model and effort both with a glyph and a caret, the 1px hairline splitting "what it
  runs on" from "where it runs"), the folder chip (last segment, full path on hover) and
  the key hint below the border. Then T2b: the branch chip. `git.rs` shells out with
  CREATE_NO_WINDOW, asks `rev-parse --is-inside-work-tree` and `--abbrev-ref HEAD` (and
  `--short HEAD` only for a detached head so the chip shows a sha instead of the word
  HEAD), answers `None` for every failure, and caches per working directory so no push
  ever waits for a process. R3 holds: no repo means no chip, no gap, no dash.
  Receipts: `npm run check` **0 errors**, web 203 tests, `-p gentle-desk-app` 560 across
  69 suites. The branch slice has NO review receipt - see L5.
- L5 (2026-11-19, the review machinery is broken OUTSIDE this repository): the review
  integration answers `package-local-binary-missing` and a subagent cannot start at all
  ("Failed to load extension ... gentle-pi: Cannot find module"). `npm root -g` still
  points at `%APPDATA%\npm\node_modules`, `npm ls -g` lists `gentle-pi@` with NO version,
  and the directory is gone - a damaged global install, not a defect in this candidate.
  The provider's own recovery (`node scripts/install-gentle-ai.mjs` inside the package)
  cannot run because the package is what is missing. Fix, outside the repo and the
  parent's to authorize: `npm install -g gentle-pi`. Until then: every slice ships
  verified by tests plus `npm run check` and WITHOUT a receipt; subagents are unavailable,
  so the remaining slices are done inline or wait.
- L6 (2026-11-19, what is left and in which order): T3 the read-only session mode
  (`get_permission_mode` behind `supports()`, one read when the chat opens, a display
  mapping with the raw value as fallback, NO switch per R1), then T4 the attachments
  (picker + drag, path insertion, the outside-workspace warning). The mode slice was
  started and could not be delegated for the reason in L5.
- L7 (2026-11-19, the maintainer asks for the mode to be changeable from the UI; probed
  against a real `gentle-shell --mode rpc` with `GENTLE_SHELL_INTERACTIVE_HOST=1`,
  contract in gentle-pi `docs/desktop-rpc.md`): R1 is REVOKED. Findings:
  - Four modes, wire strings `normal` | `auto` | `yolo` | `super-yolo` (the status text
    reads `NORMAL` / `SUPER-YOLO`; the wire is lowercase with a hyphen).
  - `get_permission_mode` -> `{mode, default_mode}`.
  - `set_permission_mode {mode, persist?}` (only those two keys; `persist` defaults to
    false = session only) answered ok for `normal`, `yolo` and `super-yolo`, each with
    `{mode, default_mode, persisted:false, changed:true}`, and emitted
    `permission_mode_changed {mode, previous_mode, default_mode, persisted,
    source:"desktop"}` AFTER the response. An unknown mode answers
    `invalid_request` ("Unknown permission mode"), state unchanged. The docs also list
    `permission_denied` / `permission_unavailable`, so the UI must show a refusal.
  - So the earlier premise "upstream refuses YOLO outside a TUI" does not hold for an
    interactive-host child. Rulings that replace R1: the UI changes only SESSION mode
    (`persist` never sent); the shown mode changes only on the confirmed response or
    `permission_mode_changed`, never optimistically; a failure shows its code and
    leaves the mode alone; yolo / super-yolo need an explicit confirmation step.
- L8 (2026-11-19, maintainer approved sheet A of `design/composer-dock/02-mode-picker.html`
  and the two proposals: `auto` is in the selector, yolo / super-yolo need confirmation).
  Probe of the open risk: with a real turn running (`bash sleep 25`, agent_start ..
  agent_end), `set_permission_mode` yolo -> super-yolo (an ESCALATION while busy) and
  super-yolo -> normal both answered ok. `escalation_while_busy` belongs to the
  desktop-feed session-action path, not to this RPC method. The control may therefore
  live in the status row, which only exists while working. T3 plan: T3a pure client
  (`crates/gentle-core/src/rpc/permission.rs`, mirror of `config_store.rs`), T3b runtime
  command + shell + `ViewState` field (mirror of `chat_profile`), T3c the web control.
- L9 (2026-11-19, T3 landed): T3a `1cf3988`, T3b `d04cae2`, T3c web (see git log). Review of the
  T3a+T3b range (lineage review-45a11616a8574766) found R3-001 CRITICAL: an answer from a
  chat that closed and reopened was installed on the new one (`is_open` is true again). The
  one bounded correction (`9ebc01c`) checked the epoch BEFORE the fold and its test only
  asserted the counter, so the targeted validator rejected it (escalated, terminal). The
  real fix landed after (`2841b1a`): the epoch is compared under the same lock the fold takes
  and `forget` bumps it under that lock, with a behaviour test through close and reopen.
  Not fixed on purpose (informational, same review): R3-002 the in-flight slot is not freed
  by `forget_session_mode` itself; R4 a refused FIRST read is invisible; R4 the shared
  desktop-client mutex can stall other same-chat requests up to 5 s; R4 an `ok` write answer
  this contract cannot parse shows `no_answer` while the session may have changed (worth a
  verification read); `session_epochs` is never pruned. `permission_mode_changed` external
  events do not reach the UI. The closing review of the T3 range was escalated, so it has
  no receipt.
