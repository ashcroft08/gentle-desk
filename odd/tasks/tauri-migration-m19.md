# tauri-migration-m19: strangler migration of the iced UI to Tauri 2 + Svelte

Implements backlog item 17: the frontend stack is decided (Tauri 2 on WebView2, evidence in
`odd/tasks/tauri-spike.md` on branch `spike/tauri`, commit `2d38d00`), and `apps/desktop`
(Svelte + TypeScript + Vite) grows beside `crates/ui` until feature parity, when `crates/ui`
is deleted. `crates/gentle-core`, `crates/backend` and `crates/app` (16 584 lines) survive
untouched.

## Specs

- S1: Strangler only: nothing of the working iced app is removed before parity. The migration is a list of feature-parity checks taken from the real features of the iced app (the checklist lives in the Design notes), and `crates/ui` is deleted, with the README's "Code organization" section and CI updated, in the last task only.
- S2: `crates/app` stays the single source of truth: its reducer and state types are unchanged by the migration, the web side is a pure view that sends intents as Tauri commands and renders state pushed as events. Pushed state is slices, never the whole `AppState`; side effects (clipboard, children, refresh passes) stay Rust-side.
- S3: The 50 000-chat list is served from Rust by a windowed query (`offset`/`length` + `query`), never serialized whole; the first pass keeps today's streaming partial lists (the loading state of sessions-loading-m17 is reproduced, via the same tested pure rules in Rust).
- S4: A UI-free runtime crate (`crates/runtime`, name to confirm in T1) receives the modules of `crates/ui/src` that carry no iced import (verified per file in the inventory below): `chat_runtime`, `chat_keys`, and the pure halves of the refresh/session machinery, so the Tauri app uses them as crates, without reimplementation.
- S5: The TypeScript types of the Rust→web contract are GENERATED from the Rust definitions (candidate generators evaluated in T2 with a tiny proof: `ts-rs` vs `tauri-specta` vs `specta`; not pre-decided), so the two sides cannot drift.
- S6: Frontend layout: `apps/desktop` holds `src-tauri` (Rust, workspace member) and `src` (Svelte); one folder per feature (`sidebar`, `thread`, `composer`, `notices`, `dialogs`, `helpers`, `chats`, `helpers` panel's thread inside `helpers`), one component per file, stores/state and all `invoke`-style bridge calls in their own files, a `max-lines` lint (~200) enforced in CI, Vitest + Svelte testing-library with tests next to the code. Pure display decisions already tested in Rust (`badge`, `status` texts, helper `format`) move their rule to `crates/runtime` and arrive as plain data over the contract.
- S7: CI gains node and the Tauri build on the Windows workflow; npm runs locally (no global installs). Expected added build time on warm caches ~24–27 s for the release rust crate plus ~57.5 s for `npx tauri build` incl. NSIS+MSI bundling (spike row 8); a genuinely cold total was not captured, so CI caching of both `~/.cargo` and node deps is required, and WiX/NSIS download once.
- S8: One branch and one PR per task, ~400 authored lines per slice; tests first with RED observed; `cargo fmt --check` and `cargo clippy -D warnings` (and for the web side the equivalent lint) before commit; every UI task ends with the parent's real-window check recipe below.

## Tasks

Sequential unless noted; route gentle-ai-worker, parent verifies; commit blank.

- [ ] T1 (S4) Extract the UI-agnostic runtime crate from `crates/ui` with NO behaviour change — `cargo move`-style: create `crates/runtime` with the modules the inventory marks MOVE (below), re-point `crates/ui` at it, the iced app keeps working on the extracted code, all tests move unchanged. The iced-bound wrappers (`worker::stream`, `refresh_flow::pump`, `WatchSource`, `subscription`) stay behind in `crates/ui`; their non-iced cores move. Crate name confirmed here. Route: gentle-ai-worker; commit:
- [ ] T2 (S5, S6, S7) `apps/desktop` skeleton: Tauri 2 + Svelte + TypeScript + Vite, `src-tauri` wired into the workspace, one-component-per-file with the `max-lines` lint enforced, Vitest + Svelte testing-library with tests next to the code, a contract-type generation proof (one tiny Rust struct → its TS type through the chosen generator, with the generator comparison recorded), and the accessible-primitives check: Bits UI / Melt UI candidates tried against a dialog/list/menu sample with UIA probes from the spike's `a11y-check.ps1`; if neither passes, say so before T3 rather than assuming. CI: node + the Tauri build job on the Windows workflow (S7). Route: gentle-ai-worker; commit:
- [ ] T3 (S2) The state bridge: `src-tauri` owns `AppState` (folded behind a Mutex as today), intent commands for every `Intent` the sidebar/composer/notices/dialogs emit, and state events that type over the chosen generator's output; Rust tests for the reducer round-trip and one integration test with `tools/fake-gentle-shell` via `GENTLE_SHELL_BIN` (never the real shell). Route: gentle-ai-worker; commit:
- [ ] T4 (S2, S3, S6) Sessions windowed query + sidebar: offset/length/query served from Rust, streaming partial lists during the first pass, Svelte sidebar with a virtualized list, search, loading state ("reading…", never a false "no chats"), per-row badges from the contract's badge data. Route: gentle-ai-worker; commit:
- [ ] T5 (S2) Thread: streamed assistant messages with incremental markdown rendering (render only the new tail — never the spike's whole-document re-parse, spike row 5 measured 5.3–5.4 CPU s/10 s for that idea), model-error box, Copy action, link handling. Route: gentle-ai-worker; commit:
- [ ] T6 (S2) Composer + chats: Send/Stop, the disabled states (working/queued/opening/ended), per-chat badges (working `…`, opening `◌`, finished-unread, needs-attention) and the restart notice with its action. Route: gentle-ai-worker; commit:
- [ ] T7 (S2) Extension dialogs (confirm/select/input/editor), notices (info/warning/error + dismiss + one action), the helpers panel and its helper thread. Route: gentle-ai-worker; commit:
- [ ] T8 (S2) Close flow (flush the throttled cache, stop every live child, no orphan process, without freezing the window), cache + watcher refresh wiring (debounced batches, watcher-lost fallback to polling, safety-net timer). Route: gentle-ai-worker; commit:
- [ ] T9 (S1-S7, and S8 with it — the process rules of every task are exercised in this last PR one final time) Parity check in the real window with the stand-in (`GENTLE_SHELL_BIN`), accessibility probes and the 50 000-chat performance check against the spike numbers (first paint ~183/172 ms vs iced 296/289; idle WS ~390 MB; first CPU/10 s ~0.02); then delete `crates/ui`, update the README ("Code organization", layout) and CI in the same PR. Route: gentle-ai-worker; commit:

Every UI task (T3–T9) ends with a real-window check **by the parent** (the parent's PowerShell driver recipe: PrintWindow on the window handle only, real cursor parked inside the window for posted clicks, SendKeys only when the window is verified foreground; restore the cursor; delete scratch).

## Design notes

The decider's proposals, kept here with their verification status against the actual code.

1. **Rust owns the state — sound, and already true.** `crates/app` and `crates/backend` have zero iced imports (grepped per file; both crates were built UI-free from the start). The window-side state of the iced app that is genuinely view-local (`input`, `dialog_text`, `sidebar_offset`, and the `ThreadCache` render cache) stays on the web side as TS store state, so nothing about the composer text or dialog-typed text needs to round-trip. What is **pushed**: per-chat decoded events (`Input`s folded through `crates/app`, re-emitted as the small fields each view needs), session list slices in windowed form (T4), progress `{done,total}` + partial-list slices during the first pass only, notices and dialogs when they appear/disappear, working/queued/opening/ended activity marks per chat as they change, helper panel summaries per update, and per-chat stderr reason lines (bounded ring of 3). What is **never pushed**: the whole `AppState`, the whole prepared list (today it ships whole per partial — 29 MB of cache at 50 k chats is the reason for S3), and markdown render caches.
2. **Windowed 50 k query — sound, and required.** The pure search/ordering already lives in `crates/app` (`prepare_sessions`/`SessionQuery`); slicing it behind a bridge command is a new narrow API over unchanged logic. One subtlety: per-row badge data must ride with the served slice (a snapshot keyed by session path of the chats that are not idle, as `view/sidebar` does today at per-drawn-row cost), not as a per-frame whole-map push for 50 k chats.
3. **UI-agnostic extraction into `crates/runtime` — sound; verified iced-free per file.** `grep iced` over every file of `crates/ui/src` (imports, not comments) confirms the spike's claim: `chat_runtime/` (all six impl modules plus tests, 862 lines total) and `chat_keys/` (185) have **zero iced imports** and move mechanically. The same check shows the refresh machinery is iced-free too **except its three transport wrappers**: `session_refresh::worker::stream` wraps the pass in `iced::stream::channel` (the `work` function under it is pure and moves), `refresh_flow::pump` consumes it via `Task::stream`, and `watch_source`/`subscription` wrap receivers in iced streams. Those three wrappers are exactly what the Tauri bridge replaces (a thread pumping a channel into `emit`), so the orchestration **is** movable — `Flow`'s decision logic (one pass at a time, settle, pump, retry-watcher, adaptive delay) moves with its two iced tasks cut out. "If it is not iced-bound" resolves to: yes, after de-icing three wrappers.
4. **Generated TypeScript types — sound.** Shown by the spike to be hand-written today (`dist/*` JS cast against ad-hoc shapes); drift between the two sides is the failure mode S5 closes. T2's proof decides between `ts-rs`, `tauri-specta` and `specta`; no pre-decision here either.
5. **Frontend layout — sound.** `apps/desktop` with `src-tauri` (workspace member; in T1's shape `crates/runtime` is what it depends on) and a Svelte `src/` in feature folders, one component per file, stores and a thin bridge layer separate from components (no direct `invoke` from a component), `max-lines` ~200 in lint/CI, Vitest next to the code. The README's "one responsibility per file" rule applies to the web side unchanged.
6. **Strangler plan — sound; the parity checklist below was restated from the actual code, not from memory.**
7. **CI — sound.** Node is already preinstalled on the CI runner (`ci.yml` comment); the Tauri job adds `npx tauri build` (57.5 s warm; WiX/NSIS download once; cold total not measured — flagged as an open measurement in T2 rather than claimed).
8. **Risks, each with how it is checked:**

   - **Svelte accessibility primitives**: WebView2 exposes the DOM to UIA (spike row 6: 62 elements / 45 named / 8 TextPattern vs iced's 1/1/0), but the accessible *behaviour* (dialog modal semantics, list/menu roles, keyboard) depends on the headless library, and the Svelte ecosystem of accessible primitives is **smaller than React's** — Bits UI or Melt UI are candidates, **unverified**. Checked in T2 by building a dialog/list/menu sample and probing UIA with the spike's `a11y-check.ps1`; the fail-loudly path is a switch decision recorded in T2, never an assumption carried into T3.
   - **WebView2 runtime dependency on end-user machines**: the dev box has it; an end user may not; NSIS/MSI behave differently on bootstrapping it. Checked in T9 by running the built installer on a machine representation without dev browsers and recording what the installer does; the spike never ran an install (row 7 says artifact size only).
   - **Lazy UIA tree**: WebView2 exposes its DOM lazily (first probe ~7 s after launch empty; full tree from loop 2 on). Checks must re-probe after a settle delay, and a single early probe never counts as evidence of absence.
   - **Windows-only evidence**: all spike numbers are one Windows machine; macOS/Linux unmeasured. If another OS is ever targeted, the numbers are re-measured there first — nothing here claims portability.
   - **Markdown re-render strategy must not be copied**: the spike re-parsed the whole growing document per delta (5.33/5.44 CPU s per 10 s at 50 ms deltas); the iced app's per-message `ThreadCache` pushes only the new tail. T5 implements incremental rendering; the spike's number exists to warn, not to be reproduced.
   - **`gentle-term` `end_to_end` failures in this checkout**: six spawn cases fail when the workspace runs in one go here (m18 Log L10: `os error 5/2` at `Command::spawn`, binary disappearing mid-run), byte-identical checkout in scratch passes, CI green. Cause unknown. Checked by recording the six named tests as expected environmental failures, never by "fixing" them tacitly.

### Parity checklist (S1, read from the code)

Sidebar: list of saved chats with search box; fixed-height rows virtualized near the viewport (`windowing` + `responsive`); first-pass loading state (`reading…`, partial rows, header counter, never a false "empty"); per-chat badges (working/queued/opening/finished-unread/needs-attention) from a per-frame activity snapshot; "new chat" becomes a stored chat on the child's first state reply. Thread: user bubbles, assistant markdown streamed incrementally (per-message cache, new tail only), model-error box, Copy action per finished message. Composer: one turn at a time, Send/Stop, disabled while opening/queued/working or ended. Dialogs: blocking extension confirm/select/input/editor with dialog-text state and autofocus. Notices: info/warning/error, dismissible, one action (Restart chat) does that one fix. Helpers: panel with counters and one card per task; helper thread with what it said, thought, ran (`format.rs` rules). Restart of a dead chat's child; a stored-chat global fatal (launcher unresolvable) banner. Close: throttled cache flush, every live child's graceful shutdown. Refresh: watcher batches + adaptive full-pass timer + first-pass progressive list, persisted index cache (throttled 30 s, flushed on close).

### Inventory of `crates/ui/src` (line counts from `wc -l`; MOVE = into `crates/runtime` unless noted)

Dispositions are by the iced-import check (imports only — a comment mentioning iced does not count):

| file (tests) | lines | fate |
|---|---|---|
| `main.rs` | 50 | REWRITE — the entry becomes `apps/desktop/src-tauri/main.rs` (window/title/settings move to `tauri.conf.json`) |
| `app.rs` | 122 | REWRITE — the wiring it does covers the Tauri bridge and TS stores of T3 |
| `app/boot.rs` | 52 | MOVE/MERGE — framework-free startup (host, index, watcher first boot) into the bridge's `setup` |
| `app/dispatch.rs` | 51 | REWRITE — becomes the intent-command handler + effects run in the Tauri bridge |
| `app/chat_command.rs` | 110 | REWRITE — the per-chat off-thread task become Rust-side workers; the opening 20 s safety net carries over |
| `app/chat_update.rs` | 88 | REWRITE — same folding the bridge does after each child event |
| `app/sessions_update.rs` | 76 | REWRITE — refresh flow wiring + the reap/close two timer steps live in the bridge |
| `app/chat_closed.rs` | 25 | REWRITE (in the bridge) |
| `app/send_prompt.rs` | 27 | REWRITE (in the bridge) |
| `app/visible.rs` | 112 | REWRITE — selection/visibility derives stay in Rust; WebView gets pushed state instead of a view helper |
| `app/frame.rs` | 72 | REWRITE — becomes `src/App.svelte`'s layout stack (order/subviews) |
| `backend.rs` | 123 | MOVE — zero iced imports; framework-free startup, into the bridge with `app/boot.rs` |
| `cache_path.rs` (tests) | 18 (+22) | MOVE — pure path rule |
| `chat_keys/` (tests) | 91 (+94) | MOVE — zero iced imports; the two-names registry |
| `chat_runtime/` (events 46, execute 69, host 40, open 29, rekey 43, root 59; tests 280+57+109+45+85 = 862 total) | 862 | MOVE — zero iced imports (verified); `execute` blocks on a child, so its **caller** in the bridge keeps running it off-thread, exactly as `app/chat_command.rs` does |
| `clock.rs` | 10 | MOVE |
| `message.rs` | 41 | DROP — iced message enum; bridge + TS events replace it |
| `refresh_flow.rs` (tests) | 195 (+195) | REWRITE after de-icing: the pass-heap decision part (settle, pump, retry-watcher, adaptive delay) can move once `Task::stream` is cut; only the two tasks (`Task::perform` API surface) stay iced |
| `refresh_queue.rs` (tests) | 85 (+127) | MOVE — pure |
| `save_throttle.rs` (tests) | 76 (+52) | MOVE — pure |
| `session_refresh.rs` (tests) | 184 (+110, +146, +166) | MOVE — `Shared` and the tests are iced-free except the stream-consumption imports (`block_on`/`StreamExt`) which the new bridge replaces |
| `session_refresh/progress.rs` (tests) | 83 (+52) | MOVE — pure |
| `session_refresh/worker.rs` | 137 | REWRITE-TAIL — `work` moves as-is; the iced `stream()`/`run()` wrappers (thread spawn + `iced::stream::channel`) become a `crates/runtime` thread + channel and the Tauri emit |
| `sessions.rs` | 67 | MOVE — pure mapping backend rows → sidebar items |
| `subscription.rs` | 84 | REWRITE — the chat-event/into-subscription idea becomes the one forwarder thread in `src-tauri` pumping `TaggedEvent`s into `emit` |
| `text.rs` (tests) | 14 (+9) | MOVE — pure truncate |
| `thread_cache.rs` (tests) | 64 (+87) | REWRITE — its incremental-tail-per-message idea must live on the web side with Svelte output; the iced `markdown::Content` type is not portable. This is a render cache, not state: not moved |
| `time_ago.rs` (tests) | 20 (+45) | MOVE — pure |
| `watch_source.rs` | 67 | REWRITE — receiver-drain wrapper becomes part of the bridge's watcher forwarder |
| `windowing.rs` (tests) | 66 (+125) | DROP — replaced by the web virtual list in T4; its f32-pixel arithmetic has no DOM counterpart. Kept as reference until T9 in case an admissibly custom window is needed |

`crates/ui/src/view/`, one line per file (fates in parentheses; all REWRITE destinations are Svelte under `apps/desktop/src/src/feature/`):

| file (tests) | lines | fate |
|---|---|---|
| `mod.rs` | 16 | DROP — the registry list is nothing but a folder in Svelte |
| `banner.rs` | 15 | REWRITE — fatal "gentle-shell is not available" bar, `chats/FatalBanner.svelte` |
| `composer.rs` | 33 | REWRITE — `composer/Composer.svelte` |
| `dialog.rs` | 100 | REWRITE — `dialogs/*.svelte` (confirm, select, input, editor — four components, not one 100-line file) |
| `header.rs` | 20 | REWRITE — `chats/Header.svelte` |
| `helper_thread.rs` | 117 | REWRITE — `helpers/HelperThread.svelte` |
| `helper_thread/format.rs` (tests) | 71 (+72) | MOVE — pure serde_json→text rules (tool status, args, output preview) |
| `helpers.rs` (tests) | 124 (+62) | REWRITE — `helpers/HelpersPanel.svelte`; the status_label/status_style drawer data moves with `format` |
| `notices.rs` | 51 | REWRITE — `notices/Notice.svelte` + list |
| `sidebar.rs` | 193 | REWRITE — `sidebar/*.svelte` (list, row, virtualized scroll; one component per file, and this 193-line file is already the size that forces the cut) |
| `sidebar/badge.rs` (tests) | 90 (+96) | MOVE — the `ChatActivity`→mark/colour/bold rule with its tests; arrives over the contract as plain data (S6) |
| `sidebar/status.rs` (tests) | 82 (+134) | MOVE — the header/loading `ListStatus` text rules incl. m17's hard-fought "never a false empty"; same route |
| `thread.rs` | 65 | REWRITE — `thread/Thread.svelte` + `Message.svelte` (markdown bubble vs Copy) |

Open decisions for the in-flight task to propose, not for the worker to decide: crate name (the doc's `crates/runtime` suggests itself but the parent owns naming), the generator (T2), the Svelte headless primitive (T2).

Recorded uncertainties:

- The inventory's "zero iced imports" claims come from `grep iced` over imports in this checkout, not from the spike's assertion alone; someone still has to try compilation of `crates/runtime` extracted in T1 before believing it compiles standalone.
- The `gentle-term` `end_to_end` six cases failing in the in-office workspace checkout (m18 Log) do not block T1's extraction, since none touch `crates/ui`; they are named environmental failures.
- app/UI-level feature parity is reviewed by features of `crates/ui`, so a bug present in iced today (e.g. stored history not being drawn in the stand-in's `get_messages` of `{}`) is NOT replay-parity: what is checked is the feature's behaviour compared with today's, not reproduced character-for-character behaviour.

## Log

- L1: "Me decidi por tauri, pero vamos a mantener el trabajo de componentes no todo metido en un solo archivo"
- L2 (stack): Svelte + TypeScript + Vite chosen by the user (the parent had recommended React). Trade-off recorded honestly: Svelte's ecosystem of accessible primitives is smaller, so dialog/list/menu accessibility must be verified early (T2 probes, never assumed); candidates kept: Bits UI, Melt UI — neither verified.
- L3 (standing rules from earlier sessions, still in force): answer in Spanish; choose by scalability and architecture (other people will use the app); one responsibility per file, split past ~150–200 lines; one branch and one PR per batch (~400 authored lines per slice); tests first with RED observed; `fmt` + `clippy -D warnings` before commit; merge only with CI green on the exact commit sha and the user's go; never the real model in tests (`tools/fake-gentle-shell` via `GENTLE_SHELL_BIN`); window-only screenshots.
- L6 (design round 3, 2026-11-08): the user rejected rounds 1-2 as too busy (51-77 buttons and 365-500 words per screen, demo controls on the main screen) and asked for new canvases from scratch: design/gentle-desk-v2/ with three directions (A workspace tree, B activity tabs, C workspace rail), each in a Chat and a Helpers artboard with the Chat | Helpers swap in the same area, plus a shared states board (00-states-board.html). Chosen artboard: none yet; the others are alternatives. Round 1-2 files stay only on the local branch feat/design-directions (733c3ad).
- L7 (2026-11-08): the user chose direction A (workspace tree, 01a/01b) as the one to implement; B and C stay as alternatives. Theme picker in A is now a custom listbox (the native select list ignored the page theme).
- L8 (2026-11-08): the user confirmed the chosen artboard is design/gentle-desk-v2/01a-tree-chat.html (A - Workspace tree, Chat view). 01b is its Helpers view; every other artboard is an alternative. No implementation started: T2 still needs the user OK.
- L9 (2026-11-08): user decisions after T2: PR #19 (T2a-T2b) merged as 039411c; adopt Bits UI as the accessible-primitives library (jsdom 6/6 for both candidates, UIA snapshots not like-for-like, so the choice rests on maintenance posture); remove the a11y lab (apps/desktop/src/a11y-lab, tools/probe.mjs, the @melt-ui/* deps and the preprocessor line in svelte.config.js) before release; the real app icon and bundling are left for the end. Finding: CI caught three things local runs and worker reports missed (clippy outer attribute, CRLF in the drift test, svelte-check without @types/node); run clippy and a no-dist check locally before calling a slice done.
