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
