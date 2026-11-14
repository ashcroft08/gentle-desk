# settings-0.1.12: terminal-driven settings, updates, and context visibility

The maintainer's plan for the 0.1.12 delivery: the settings roadmap's version plan
(odd/tasks/tauri-settings-roadmap.md, agreed 2026-11-12) plus the additions decided in
session on 2026-11-13. Version convention: Preview deliveries bump the patch (0.1.11
shipped; this branch delivers 0.1.12).

## Specs

- S1 (roadmap 0.1.12, agreed 2026-11-12; refined 2026-11-13 by L11): "0.1.12: read-only profiles
  view (read profiles.json), auth status via `gentle-shell auth check --json`, "Open in terminal"
  buttons for login/logout and profile editing, organic vs full-delegation switch via the
  GENTLE_PI_ORCHESTRATOR_MODE env var at child launch (session-scoped, not persisted).
  Refinements: the Profiles header shows the ACTIVE profile ("default for new sessions") only;
  auth status lists the CONNECTED PROVIDER NAMES from auth.json (names only, never keys) with a
  per-provider `auth check --provider X --json --no-refresh` (ready/not_ready); no account
  identity is shown (the CLI does not expose one)."
- S2 (user, 2026-11-13, verbatim): "las actualizaciones de pi que son mediante los
  comandos "pi update"/"pi update --extensions" y para gentle-shell son "gentle-shell
  update"/"gentle-shell update --extensions" para agregar esto tambien".
- S3 (user, 2026-11-13, verbatim): "1.automática al abrir" — the update check runs
  automatically when the app opens (one check, not per settings visit).
- S4 (user, 2026-11-13, verbatim): "2.Actualizamos solo con todos los chats cerrados
  (simple y seguro)" — pi/gentle-shell updates run only when no chat is open/working.
- S5 (user, 2026-11-13, verbatim): "podriamos mostrar visualmente en cada chat el
  porcentaje de contexto?" — a visual context-percentage indicator per chat.
- S6 (user, 2026-11-13, verbatim, clarified 2026-11-13): "en configuracion hay la posibilidad
  de agregar una secion "Uso"..." — the user meant the SUBSCRIPTION usage view (gentle-shell's
  Alt+U overlay, "✿ Subscriptions": provider plan, per-window usedPercent and resets), not a
  token counter. Deferred out of 0.1.12: gentle-pi must expose it (get_usage RPC or a JSON
  subcommand) first; the token/cost JSONL ledger was DECLINED by the user ("mucha informacion").
- S7 (verified 2026-11-13, real commands): `pi update` exists (--self default, --extensions,
  --models, --all, --force); `gentle-shell update` exists (updates the Pi the launcher runs);
  `gentle-shell upgrade [--channel release|main] [--rollback]` upgrades the shell itself.
  No built-in "check" command; pi's pending update is detectable by comparing `pi --version`
  (1.5.2 installed) against the npm registry (2.0.5 published). gentle-shell is NOT on npm
  (E404 verified) — where `upgrade` reads its latest is an open point of T1.

## Tasks

- T1 (S6, S7) DONE - Feasibility probe (explorer mwf6k0t6, path:line evidence in the Log below):
  verdicts recorded; decided the Updates check's data sources and the contextUsage surface.
- T2 (S1) Profiles read-only view in Settings (reads profiles.json).
- T3 (S1) Auth status + "Open in terminal" for login/logout and profile editing.
- T4 (S1) Organic vs full-delegation switch (GENTLE_PI_ORCHESTRATOR_MODE at child launch,
  session-scoped, not persisted).
- T5 (S2-S4) Updates section: automatic check at open, versions shown, update buttons gated
  on zero open chats, output captured (precedent: shell install streaming, PR #112).
- T6 (S5) Context-percentage indicator per chat, drawn from the verified RPC surface.
- T7 (S6) DEFERRED out of 0.1.12 by the user's decision (L6): the subscription usage the user
  wants needs gentle-pi to expose it (get_usage RPC or `/gentle:usage --json`) — a separate
  request in the gentle-pi project; gentle-desk must NOT replicate the provider fetchers
  (credentials handling, duplicated parsers). The JSONL token/cost ledger alternative was
  declined as too much information. Placement UX is designed when the data surface exists. The user confirmed (2026-11-13):
the subscription panel goes to NEXT version's analysis; 0.1.12 ships the four pieces only
(Profiles read-only, Account/Open-in-terminal, Updates, context chip).

## Log

- L17 (2026-11-13, maintainer round on the running app; web side): three reported defects,
  fixed on feat/settings-0112. FEEDBACK (verbatim): Profiles — "esta mucha la informacion, hay
  forma de mejorarlo con pestañas"; Account — the pane sits on "Reading the sign-ins..."
  forever; the fallback chain is drawn twice (card pair AND a `fallback` table row). FIX 1
  (Account, bug): nothing dispatched `RefreshAuth` on appearance, so the only trigger was
  "Check again". `AccountPane.svelte` now mirrors the Profiles pane - one dispatch when the
  pane mounts and one on every `window` focus while it is the section on screen (the dialog
  draws only the selected section, so mount IS the appearance and unmount drops the
  listener) - and the quiet "Reading the sign-ins..." is the waiting state of the ~1s
  per-provider check that runs on the intent thread. FIX 2 (Profiles, the maintainer's ask):
  the pane is now two sub-tabs, Profile | Agents, a segmented control built from the theme
  tokens (`app.css` `.settings-pane .subtabs`, no new colour), with the chat header's
  tablist pattern (`chat/tabs.ts`): one tab stop, arrows walk and wrap, Home/End jump, and
  tab/panel name each other through `aria-controls`/`aria-labelledby`; both panels stay
  mounted with the inactive one `hidden`, the way App.svelte keeps Chat | Helpers. The rules
  live in the new pure module `settings/profile-tabs.ts` (`PROFILE_TABS`, copy, ids,
  `profileTabAfter`). Tab "Profile" holds the orchestrator switch with its "For chats opened
  from now on · not stored" note, the active-profile card (name accent, "Default for new
  sessions", ORCHESTRATOR + FALLBACK pairs) and the "Edit in terminal" button with its note;
  tab "Agents" holds the Agent | Model | Effort table and nothing else. Default tab: Profile.
  The pane's title and hint stay the dialog's. The file's own states (reading, failed with
  the safe retry, no profiles) sit above the pair, because both tabs read the same document.
  FIX 3 (fallback row, bug): `agents` filters `role.role !== 'fallback'` out of the table, so
  the chain is drawn once, in the card's FALLBACK pair. Design: design/settings-0112/
  02-ledger.html redrawn to the tabbed layout (same artboard, same manifest entry), including
  the Agents panel and the "no switch, no button, no fallback row" caption. Tests (test-first,
  RED observed then GREEN): 4 new in `settings/ProfilesPane.test.ts` (opens on the Profile tab
  with the switch, card and button and no table; switching to Agents with the aria wiring and
  neither switch nor button; arrow-key walk with one tab stop; the fallback role never renders
  a row) and 1 new in `settings/AccountPane.test.ts` (one check on mount, one more on focus);
  the existing "checks again" and "draws the active profile" tests were adapted to the new
  trigger and to the Agents tab, and `test-support/settings.ts`'s `pane()` now reads the
  dialog's own pane by id so a section's nested panel cannot be mistaken for it. Full gates
  green on apps/desktop: npm run lint, npm run check, npx vitest run (465/465), npm run build.
  Render check: the pane dumped from a disposable vitest render and screenshotted with Chrome
  `--headless=new` in both themes - Profile tab: segmented pair with the selected one on the
  tint and accent, mode switch readable in both themes, card and button aligned; Agents tab:
  the same pair, the table alone, no fallback row. Both themes keep the house contrast.

- L12 (2026-11-13, Rust side): the four surfaces have their Rust half. `get_session_stats`
  joined `RpcCommand` (codec + test); the app keeps one context reading per chat
  (`crate::context`), asks it on every `AgentSettled` and on open
  (`chat_runtime::request_after_open`), and the slice carries it as
  `ContextUsageView` with nulls, never zeros. `profiles.json` is read through
  `gentle_runtime::profiles` (strict envelope, active profile, reserved
  `orchestrator` role, workflow map, plus the shell's four-layer config-home
  rule); `auth.json` is read for its top-level KEY NAMES only
  (`gentle_runtime::auth`), and each provider gets `auth check --provider X
  --json --no-refresh` through a seam. `gentle_runtime::updates` compares strict
  `x.y.z` (prerelease and garbage are never older); the check runs once when the
  app opens and again only on "Check again", reading pi from
  `gentle-shell --version` against the npm registry and gentle-shell's version
  with no npm comparison at all (its channel is unknown here). The orchestrator
  mode goes to each new child through `GENTLE_PI_ORCHESTRATOR_MODE`
  (`ProcessSpawner`, interior mutability, no `set_var`) and is never stored. All
  of it is in the contract (`ProfilesView`, `AuthView`, `UpdatesView`,
  `ContextUsageView`) and the intents (`RefreshProfiles`, `RefreshAuth`,
  `CheckUpdates`, `SetOrchestratorMode`). Web rendering, the update BUTTONS and
  the chats-closed gate are the web writer's half.

- L8 (2026-11-13): design written at design/settings-0112/ — 01-rail.html (the settings
  rail grows: Updates pane with status rows and the chats-closed gate), 02-ledger.html
  (Profiles and Account as quiet ledgers), 03-chat-chip.html (context chip beside the
  composer with its five states). The chosen artboard will be recorded when handed off;
  the others stay alternatives.
- L15 (2026-11-13, native review of the Unverified fix, lineage review-b47114a668c5a5cb,
  tier high, lenses review-risk + review-reliability, 2 reviewer model runs via host relay,
  total cost ~USD 0.008): APPROVED, no correction. Advisory finding saved here verbatim:
  "R1-provider-not-found-overbroad" (lens review-risk,
  apps/desktop/src-tauri/src/settings/auth.rs:60-67, severity WARNING, disposition
  informational): the finding's title flags that the provider_not_found -> Unverified mapping
  may be overbroad (it treats every provider pi's registry does not name as unverified);
  non-blocking, later work. The reviewer result artifacts:
  rart1_f7162444b0bfa5d96429a3405e334af67511856981ddca64c2666cc286cf6547 (risk,
  sha256:46834e3beee35890b669d782ae6645abbf718e35bab8981e7442a30d10e4865b) and the
  reliability artifact of the same store revision b4503588474990240a3416b38ea0f88226a0813d6afca02c7b06214df947d2fd.
- L13 (2026-11-13, user): fallback route — "(a)" add the profile-fallbacks.json read to the
  Rust side so the Profiles pane's FALLBACK row shows the real chain; and "Levantas la app para
  probar todo lo nuevo implementado" — the app runs for the maintainer's hands-on test of all
  the new pieces, including open_terminal (whose spawn path no writer could execute).
- L14 (2026-11-13, Rust side, L13 option a): the FALLBACK row now shows the real chain.
  READ (gentle-pi, C:\Users\josep\.pi\agent\npm\node_modules\gentle-pi): the file is
  `profile-fallbacks.json`, a SIBLING of `profiles.json` — `fallbacksFilePath(configHome)`
  (lib/model-fallback-store.ts:33-35) against `profilesFilePath(configHome)`
  (lib/agent-profiles.ts:685-687), both from the same config home, so the desk's existing
  four-layer rule (L12, `gentle_runtime::profiles::config_home`) resolves both and
  `SettingsPaths` now carries the second path beside the first. SHAPE (verbatim, kind/version
  constants at lib/model-fallback-store.ts:9-13; a real document in
  gentle-pi tests/gentle-agents.test.ts:8274):
  `{"kind":"gentle-pi.profile_model_fallbacks","version":1,"profiles":{"home":{"*":[{"model":"fby/second-model","thinking":"low"}],"orchestrator":[{"model":"orx/backup","thinking":"high"}]}}}`
  — profile name → agent name (or the `*` default) → ordered steps of `{model, thinking?}`.
  Refusals are gentle-pi's own (`normalizeFallbacksFile`, lib/model-fallback-store.ts:52-80):
  exactly the three envelope keys, a profile name that passes its slug pattern, an agent name
  that is `*` or a safe name, at most 8 steps, unique models per chain, safe model ids and the
  seven known thinking levels; a broken file is "no chains", never rewritten. MAPPING: the
  contract does NOT change — the pane already draws a role named `fallback` as the pair, so
  each profile's chain reaches the view as one such row (`fallback.model` = the chain's models
  joined with `→`, `fallback.thinking` only when every step asks for the same effort, `tier`
  always null because a step carries none; a real role that happens to be named `fallback` is
  replaced by the chain, so the pane's keyed list keeps one row). The chain drawn is the
  orchestrator's own, else the profile's `*` — `chainFor` (lib/model-fallback-store.ts:102-107).
  A missing, unreadable or refused chain file leaves the row undrawn and the pane `Ready` (the
  sibling file is never a reason to fail the profiles read), and a profile the file does not
  name gets no row. Web untouched: no bindings regeneration was needed. Tests: 8 new in
  `crates/runtime/src/profiles.rs` (real shape + refusals + sibling path + attach), 6 new in
  `apps/desktop/src-tauri/src/settings/profiles.rs` (row, no file, refused file, orchestrator
  chain, other profile, mixed effort, existing role), 2 new in
  `apps/desktop/src-tauri/tests/settings.rs` (sibling wiring through `RefreshProfiles`, broken
  file leaves the pane ready).
- L11 (2026-11-13, user decisions): auth — "Leer nombres, ok" (gentle-desk reads provider NAMES
  from auth.json, never keys/tokens; per-provider check with --no-refresh so pi never refreshes
  OAuth tokens from the desk); profiles — "Solo activo ahora" (the JSONL session-binding parse is
  deferred; the panel shows the active profile's "default for new sessions" only). The artboard's
  "used in this session" wording is therefore reduced in implementation.
- L9 (2026-11-13, user via design comments): asked whether profiles can be CREATED and
  SWITCHED from the desk. Answer recorded: today only the terminal's /gentle:profiles TUI
  manages them (its own keys n new / u Use in this session); the desk panel explains that and
  the wish joins the 0.2.0 gentle-pi request (text subcommands use/new) already recorded in
  tauri-settings-roadmap.md.
- L7 (2026-11-13, user, verbatim): "entonces como no hay como exponerlos dejemoslo para
  analizarlo para la siguiente version, ahora si dale con los dibujos (artboards) de estas
  4 piezas." — design scope fixed: 01 Profiles, 02 Account, 03 Updates, 04 context chip.

- L1 (2026-11-12, roadmap): the 0.1.12 batch was agreed in odd/tasks/tauri-settings-roadmap.md
  ("read-only profiles view, auth status, Open in terminal, orchestrator mode switch").
- L2 (2026-11-13, user, verbatim): "ya que la 0.1.12 son ajustes de termminal queria agregar
  algo más, las actualizaciones de pi que son mediante los comandos "pi update"/"pi update
  --extensions" y para gentle-shell son "gentle-shell update"/"gentle-shell update
  --extensions" para agregar esto tambien no se si le llega el aviso de que hay alguna
  extension que actualizar o si pi hay que actualizar".
- L3 (2026-11-13, user decisions, verbatim): "1.automática al abrir, 2.Actualizamos solo con
  todos los chats cerrados (simple y seguro). Otra cosa que se me vino a la mente pero no se
  si entra para esta version o para la siguiente, podriamos mostrar visualmente en cada chat
  el porcentaje de contexto? y tambien en configuracion hay la posibilidad de agregar una
  secion "Uso" para ver los tokens que se van usando, en gentle-shell para ver eso es
  /gentle:usage".
- L4 (2026-11-13, user): "si dale" — the batch above (S1-S7, T1-T7) is authorized to start;
  design first, then piece-by-piece.
- L6 (2026-11-13, user, verbatim): "Que gentle-pi lo exponga, lo de los Stats que nombras es
  mucha informacion y quiero que me ayudes a pensar donde seria bueno mostrarlos y como ya que
  mucha informacion visual desgasta al usuario" — S6/T7 updated accordingly (deferred, pending
  a gentle-pi get_usage surface); the user asked for low-visual-load placement thinking.
- L5 (2026-11-13, T1 probe, explorer mwf6k0t6): (1) UsageStore is a plain in-memory Map
  (shell-usage.ts:621-622, only constructed at gentle-shell.ts:2068); /gentle:usage's
  subscription percentages live only in that memory for Codex/Anthropic/NaN — no file, no CLI
  JSON, NOT reachable from gentle-desk. But pi's session JSONL IS a persisted parseable token
  ledger (~/.pi/agent/sessions/<encoded-cwd>/<id>.jsonl; header line {type:"session",...},
  then {type:"message",message:{role:"assistant",usage:{input,output,cacheRead,cacheWrite,
  total,cost}}}; stats-collector.ts:7-10,143; session-scan.ts:209) — the source /gentle:stats
  reads; gentle-desk must duplicate its skip rules and cache incrementally. (2) gentle-shell
  `upgrade` release channel reads https://registry.npmjs.org/gentle-pi/latest with strict
  x.y.z numeric compare, prereleases rejected (main-channel.mjs:291-303,270,459-467); the
  main channel compares GitHub SHAs, never npm — a main-channel shell must not be compared
  against npm. `gentle-shell update --extensions` as such does not exist: `update` forwards to
  pi (launcher:1290-1296), bundled installs redirect self updates to `gentle-shell upgrade`
  (bundled.ts:591-612), and `--extensions-only` passes straight through (pi-update.mjs:23).
  Startup auto-check precedent: findStartupUpdate, 1.5 s budget, 24 h TTL cache in the
  launcher config (gentle-shell-startup-update.ts:109,29-40,120-131). Pi's latest:
  registry.npmjs.org/@earendil-works/pi-coding-agent/latest. (3) RPC: get_state does NOT
  carry contextUsage (rpc-mode.js:367-382); `get_session_stats` -> data.contextUsage =
  {tokens:number|null, contextWindow, percent:number|null} (rpc-mode.js:488-490,
  rpc-types.d.ts:303-306, agent-session.js:3590-3640) — request/response, needs polling or
  refresh on settled events; tokens/percent are legitimately null right after compaction
  (render "-", never 0%).

- L16 (2026-11-13, chain surgery before review): the 0.1.12 work is split into a three-PR
  chain, so the native review engine sees one reviewable diff per workspace instead of the
  whole ~9.8k-line range (its budget is the merge-base diff). The pre-chain history stays
  recoverable in the reflog only: the old linear feat/settings-0112 was df2174b (docs and
  design), df04873 (Rust), c0fae03 (the Unverified fix), baff543 (its review note), 49a6f6d
  (the fixture fix) and bd0d463 (web), all on top of fd1f002, which origin/main holds as
  c4a40d0. The chain, each branch cut from origin/main (c4a40d0):
  1. feat/settings-0112-context (8a607db) — ONLY the per-chat context seam: RpcCommand::
     GetSessionStats with its codec arm and test, `crate::context`, the chats
     store/lifecycle/queries/apply/mod context parts, `get_session_stats` in the open asks
     and on AgentSettled, ContextUsageView and its one ViewState field with its mapping, the
     fixture's context_usage, and bindings regenerated to exactly the context additions.
     19 files, 504 insertions, 24 deletions.
  2. feat/settings-0112-settings (stacked on 1) — the rest of the Rust work, this feature doc
     and the design artboards: the profiles/auth/updates reads, the spawner's
     GENTLE_PI_ORCHESTRATOR_MODE, the settings web seam, open_terminal, ProfilesView/
     AuthView/UpdatesView and open_chats with their intents and mappings, the src-tauri
     settings tests, the fixture's settings fields, and bindings regenerated to the full
     state. 37 files, 5575 insertions, 711 deletions.
  3. feat/settings-0112 (4953947, stacked on 2) — the existing web commit rebased, so its
     diff is ONLY apps/desktop/src/**: the six-section settings rail, the Updates/Profiles/
     Account panes, the context chip, app.css and App.svelte, and the AboutUpdates ->
     updates test rename the old docs commit carried. 31 files, 2129 insertions,
     403 deletions.
  Every branch was verified in full before handoff, all checks exit 0 on each of the three:
  cargo fmt --all --check, cargo clippy --workspace --all-targets -- -D warnings, cargo test
  --workspace, and in apps/desktop npm run lint, npm run check, npx vitest run (423/423 on
  branches 1 and 2, 460/460 on branch 3) and npm run build. Branches 1 and 2 keep the OLD web
  files, which compile against that branch's own bindings. bindings.ts is regenerated on
  each branch, never hand-merged. One repair, reported rather than hidden: the old Rust
  commit had spliced the new contract views between the two lines of ListStatusView's doc
  comment (in contract.rs, and so in bindings.ts); the chain keeps that comment whole, the
  only content difference from bd0d463's tree.

- L19 (2026-11-13, delivery complete): the feature landed on main as the approved 3-PR chain
  (#146 6fc8470 context seam; #147 f266ead settings reads + orchestrator mode + terminals;
  #148 197ec6e web panes + context chip + update gate); issue #144 closed. The auth surface
  was natively reviewed twice, both APPROVED and acknowledged: lineage
  review-b47114a668c5a5cb (the Unverified fix; advisory R1-provider-not-found-overbroad) and
  lineage review-60d4fdba89098aa0 (the auth link candidate, 1674 lines, 2 reviewer runs,
  advisories R3-001 crates/app/src/chats/apply.rs:52-55 and R3-002 crates/runtime/src/auth.rs:44-56;
  findings preserved at Temp/wn/auth-review-findings.md and quoted here by id and location).
  The wider settings range exceeded the native lens-context budget as one candidate (terminal
  stop lens_context_budget_exceeded at 9234 and 6790 lines; nothing created, no retry possible),
  so per the maintainer's decisions it was delivered as the chain, with CI green on every link
  and the maintainer's real-app test feeding three fixes (L17) into the final link. Open for the
  release: nothing in the feature; the maintainer's retest of the dev app (open_terminal never
  executed by a writer) and the 0.1.12 release phase remain maintainer decisions.

- L20 (2026-11-13, release): the terminal hand-offs were fixed for good after four measured
  rounds (direct spawn, cmd /k, inherited handles, PI_* env): a console-less GUI process cannot
  give a child a usable console, so the desk now opens **Windows Terminal** (`wt.exe -w -1
  new-tab -- launcher args`, detached) with the direct spawn as fallback; the maintainer
  confirmed both buttons work and are faster. The Settings design audit (rendered in Chrome,
  dark + light) found and fixed two things - the Profiles state line sat between the sub-tabs
  and the switch, and Account accented the secondary action - and left the tags, notes and tab
  strip as they were. 0.1.12 bumps the version in three files and ships from main; the roadmap
  records it as shipped and lists 0.1.13 candidates.

- L21 (2026-11-13, maintainer gap found on the running app; feat/update-extensions): the
  Updates pane could update pi and the shell but had NO way to update the installed
  EXTENSIONS, even though S2 named `pi update --extensions` and `gentle-shell update
  --extensions` as part of the batch. VERIFIED COMMAND TRUTH (reused, not re-probed): `pi
  update --extensions` updates the installed packages only; `gentle-shell update` FORWARDS
  to pi's updater, so pi's own flags pass straight through; and there is NO cheap way to
  know whether extensions are outdated (no registry check per package), so the action is
  offered WITHOUT a check - it is idempotent and safe to run any time. WHAT WAS ADDED
  (Rust): `TerminalActionView::UpdateExtensions` (contract.rs, its doc comment says why it
  needs no check) and its `action_args` arm returning `["update", "--extensions"]`
  (terminal.rs, commented with the pass-through reason); bindings regenerated with
  `GENTLE_DESK_REGEN_BINDINGS=1 cargo test -p gentle-desk-app --test contract_bindings`, so
  the web union now carries `"UpdateExtensions"`, and the terminal keeps the same home
  flags the chats' children get. WHAT WAS ADDED (web): a second button "Update
  extensions" in `UpdatesPane.svelte` - one Svelte snippet rendered in BOTH pane states,
  so no data drives it - next to Check again (and next to the accent Update when that one
  shows), DISABLED by the same chats-closed gate (S4) with `aria-describedby` on the gate
  hint while blocked, opening the terminal through the same `openTerminal` call. Copy in
  `updates-copy.ts`: `UPDATE_EXTENSIONS` = "Update extensions" and
  `UPDATE_EXTENSIONS_IN_TERMINAL` = "Update extensions opens a terminal window and updates
  the packages your shell has installed. There is no way to tell which ones are behind,
  so this button is always offered and safe to use any time." - no version, no promise, no
  fake "extensions are behind" state. The gate hint now says the update buttons (plural)
  come alive, because two of them are gated. TESTS (test-first, RED observed then GREEN):
  1 new in `terminal.rs` (`update_extensions_is_the_shells_update_with_pis_own_flag`; the
  RED was the E0599 the missing variant gave); 4 new in `UpdatesPane.test.ts` (always
  offered with its note next to Check again while everything is up to date; offered before
  the check answered, and never checking; one click opens the terminal on UpdateExtensions
  and nothing else; dead with 1 open chat, described by the gate line, opening nothing).
  The existing Updates and SettingsDialog tests stayed green unchanged because their
  `name: 'Update'` query is exact and never matches the new button. GATES, all exit 0:
  cargo fmt --all --check; cargo clippy --workspace --all-targets -- -D warnings; cargo
  test --workspace; and in apps/desktop npm run lint, npm run check (0 errors, 0
  warnings), npx vitest run (469/469) and npm run build. RENDER CHECK (the pane dumped
  from a disposable vitest render into a temp dir, a static page with app.css + theme.css
  inlined, screenshotted with Chrome `--headless=new`, dark theme, images read):
  update-available card - the warning tag, the `pi 1.5.2 -> 2.0.5` headline, the quiet
  versions row and ONE row of three buttons (accent Update, Update extensions, Check
  again) with the two notes under it; up-to-date card - the success tag, no Update,
  Update extensions and Check again side by side with only the extensions note; blocked
  variant - the accent Update and Update extensions both dimmed while Check again stays
  live, and the tinted gate line saying the update buttons come alive. No layout break,
  the notes wrap inside the card, the house contrast is kept. The dev-app retest of these
  two terminal runs stays the maintainer's, as in L19/L20.

- L22 (2026-11-14, maintainer gap found on the running app; feat/close-chat-and-update-dot,
  0.1.14): TWO related pieces reported together. (a) The Updates pane's chats-closed gate (S4)
  said "close every chat" but the desk had NO way to close one: the maintainer had to quit the
  app to satisfy it. (b) The app only ever told him an update existed once he opened Settings ->
  Updates. WHAT WAS ADDED (piece a, Rust): `Intent::CloseChat(ChatId)` and
  `ChatCommand::CloseProcess { chat }`, applied in `chats::intents::close_chat`; `Chats::close`
  removes the chat's entry, its foreground mark and its own queued prompt (the queue FIRST, so
  `set_queued` cannot re-create the entry it is dropping), and answers whether the app knew the
  chat - a chat this run never opened emits no command and no notice. `ChatRuntime::close` +
  `ChatCommand::CloseProcess` in `chat_runtime::execute` (new `Outcome::Closed { chat }`) stop
  ONE child through the public `ChatHost::close`, on the command's own worker thread because the
  host's grace is up to 2 s; the shell folds the outcome as `Input::ChatClosed { expected: true }`
  (idempotent with the child's own closed event, so no death notice for a chat the user put
  away). A chat that is working is REFUSED, never yanked mid-turn, with the delete flow's own
  sentence: `chats::WORKING_CHAT` ("This chat is working. Stop it first") is shared, and
  `close_working()` / `chat_delete::working_child()` only add the verb ("then close it." / "then
  delete it."), so the row menu never grows a second wording. Refusal is a Warning notice;
  closing is NOT deleting - nothing on disk is written and the session list keeps its row, so the
  row opens the chat again with a fresh child through `--session`. WHAT WAS ADDED (piece a, web):
  "Close chat" as the row menu's second item (Rename, Close chat, Delete - Delete stays last and
  red) through `onClose` -> `WorkspaceTree` -> `Sidebar::closeChat` ->
  `UiIntent::CloseChat`, so it rides the existing intent dispatch and is windowless-testable;
  bindings regenerated. The gate hint now names the way out: "Updates need every chat closed. 1
  chat is open - Close chat in its row menu, and the update buttons come alive." WHAT WAS ADDED
  (piece b, web only): the footer's gear draws a 6px accent dot when any row a terminal updates
  (`pi`, `shell`) has `update_available === true` - rule in `update/dot.ts::updateWaiting`, the
  app's own row excluded because the banner at the top already installs that one - and no number
  and no word on the bar; the dot is `aria-hidden` decoration and the gear's own accessible name
  carries the news ("Settings - an update is available"), because an inner label inside a
  labelled button is discarded by the name computation. NOT DONE, by the parent's fallback: the
  "one info notice per check" half. Reasons, all structural: the drawn notice strip drops every
  Info notice (`notices/visible.ts`), the notice list and its text are Rust-owned while the
  wording must come from the pane's own `HEADLINE` helper (`settings/updates-copy.ts`), and no
  `UiIntent` can open a Settings SECTION - `settingsOpen` and the selected section are view-local
  (`sidebar/Sidebar.svelte`, `settings/SettingsDialog.svelte`) - so an "Open Settings - Updates"
  action needs a new cross-component channel, not an extension of `NoticeView.action`. TESTS
  (test-first, RED observed then GREEN): RED was 12 E0599s (missing intent, command, `WORKING_CHAT`,
  `close_working`) and 3 failing vitest files (missing `update/dot`). 6 in
  `crates/app/tests/chat_close.rs` (open set drops + `CloseProcess` + foreground cleared; the
  other chats and their children untouched; session list and file untouched; the working refusal
  with the shared sentence; a queued prompt of the closed chat never goes out when a slot frees;
  an unopened chat changes nothing); 3 in `chat_runtime::tests::calls` (only that child stops and
  the others still take commands; a closed chat starts a fresh child on reopen; closing a chat
  with no child is not an error); 3 in `src-tauri/tests/chat_close.rs` (a REAL temp session file
  survives the intent, `open_chats` 0, revision unmoved; the working refusal keeps it open; an
  unopened chat asks for nothing); 6 in `update/dot.test.ts`, 3 in `sidebar/UpdateDot.test.ts`, 3
  in `sidebar/CloseChat.test.ts` (item order, one `CloseChat` intent, keyboard reach) plus the
  existing menu test updated to the three items. GATES, all exit 0: cargo fmt --all --check; cargo
  clippy --workspace --all-targets -- -D warnings; cargo test --workspace (1138 passed); and in
  apps/desktop npm run lint, npm run check (0 errors, 0 warnings), npx vitest run (481/481, 73
  files) and npm run build. RENDER CHECK (the real Sidebar dumped from a disposable vitest render
  into a temp dir outside the repo, a static page with theme.css + app.css + the injected
  component styles, screenshotted with Chrome `--headless=new`, dark theme, images read):
  foot-with-dot - Preview and the gear with the 6px accent dot on its upper-right, the icon and
  the row above it untouched; foot-without-dot - the same footer with no dot; 3x zoom - the dot
  is round, accent, clear of the gear's stroke; menu-close-chat - the card under the row with
  Rename / Close chat / Delete, labels whole, Delete still the red one, and the footer dot still
  drawn. The floating layer's position is pinned in the proof page (bits-ui measures it from the
  live layout, which a static dump has none of); the card itself is the component's. The dev-app
  retest of Close chat against a real child, and the version bump, stay the maintainer's.

- L23 (2026-11-14, the Usage section: S6's deferred half, delivered): the subscription usage
  view T7 deferred now has its data surface, so the seventh Settings section exists. DATA
  SOURCE (verified this session, not re-probed): `pi-usage-sources` (github.com/ashcroft08/
  pi-usage-sources) gained a headless JSON mode; `node <checkout>/usage-json.ts` prints ONE line,
  `{"schema":"pi-usage-sources/usage/v1","generatedAt":...,"sources":[{"provider","status",
  "fetchedAt","limits":[{"name","limitReached","windows":[{"label","usedPercent","resetAt",
  "windowSeconds"}]}]}]}`, with `plan` when a provider has one and `error` (already redacted)
  when it failed. Statuses ok / no-data / failed; a run takes ~5 s. WHAT WAS ADDED (Rust):
  `gentle_runtime::usage` holds both pure rules - the LOCATOR (`package_entry` reads the
  shell's `settings.json` package list for the entry naming the package, `checkout` turns
  `git:<host>/<owner>/<repo>` into `<agent home>/git/<host>/<owner>/<repo>` and a local entry
  into its path, refusing `..`, an empty segment and the `npm:` spelling, and `script` appends
  `usage-json.ts`) and the PARSER (`parse` refuses a document of another schema or shape, maps
  ok/no-data/failed, keeps `resetAt: null` as "no countdown", and `reset_label` phrases it with
  a new `time_ago::time_until` that rounds UP). `settings::usage_view` maps the four locator
  states (Installed / Missing / Unreadable / Failed) plus `Reading` into `UsageView`, reading
  the settings and the script through the existing `FileRead` seam and running node through a
  new `NodeProbe` seam with a 30 s budget (stdout/stderr drained on threads, overrun child
  killed, non-zero exit is "no document"). `SettingsPaths` carries the agent folder;
  `Intent::RefreshUsage` + `Bridge::refresh_usage` push Reading, run on a thread named
  `usage-read`, push the answer. `TerminalActionView::InstallUsageSources` runs
  `pi install git:github.com/ashcroft08/pi-usage-sources` in the house's terminal window, never
  silently. NOT at app open: a run costs seconds, so the pane asks once when the section is
  opened and on "Check again" (the cheap correct answer). WHAT WAS ADDED (web): the seventh
  rail entry Usage between Account and About (Lucide `gauge`), `settings/usage.ts` (pane states
  and the gauge rows, reusing the context chip's `toneOf`/`barWidth`), `settings/usage-copy.ts`,
  and `UsagePane.svelte`: one card per provider with its name, its plan when it has one, one row
  per limit and window with a bar + percentage + the shell's own "resets in 4h", the warning tag
  on a limit the provider marked reached, "No data" (never 0%) for a no-credentials provider,
  and the quiet loading line. Missing package: one line, "Install in terminal" and "Check
  again"; Unreadable/Failed: `Could not read the usage: <reason>` and the safe retry. Copy says
  the numbers come from the shell's own sources and that a provider without credentials shows
  no data. Tests (test-first where a rule is testable, RED observed for the time helper, the
  settings slice, the terminal arm, the bridge intent and the pane component; the runtime usage
  rules and the pure web rules were written in the same pass as their tests, so their evidence
  is GREEN only, reported honestly): 16 in `crates/runtime/src/usage/tests.rs` + 2 for
  `time_until`, 12 in `settings/usage.rs`, 4 in `tests/usage.rs` (through the real
  `Bridge::dispatch`, including "an open chat never holds the read"), the terminal arm
  assertions, 8 in `usage.test.ts`, 8 in `UsagePane.test.ts`, and the dialog's rail/hint/usage
  tests updated to seven sections. GATES, all exit 0: cargo fmt --all --check; cargo clippy
  --workspace --all-targets -- -D warnings; cargo test --workspace (1175 passed); and in
  apps/desktop npm run lint, npm run check (0 errors, 0 warnings), npx vitest run (499/499, 75
  files) and npm run build. RENDER CHECK (the real pane rendered through Svelte SSR with the
  project's own vite config, the REAL document captured from `node usage-json.ts` this session,
  app.css + theme.css inlined, screenshotted with Chrome `--headless=new`, images read, dark and
  light): four provider cards - opencode-go "No data", antigravity's two week windows at 0%,
  commandcode (plan "individual goat") at 5%/88% (warning tone) and credits 44%, claude-bridge
  (plan "pro") at 21% and 96% (danger tone), every row with its right-aligned "resets in ...";
  the missing state's one line, accent "Install in terminal" and "Check again" with its note;
  the failed state's reason line, retry and "Checking again is safe." Both themes keep the
  house contrast and nothing overflows the card. NOT RUN by a writer: the real dev app against
  the real `node` run (the maintainer's retest), and open_terminal for the install button.
