# tauri-settings-roadmap: settings by app versions (updated 2026-11-12)

The user wants settings to grow in steps, with the GUI asking and the gentle-shell
tooling doing the real work. This file records the roadmap and what was verified, so it
is not investigated twice. Nothing here is built yet except the settings dialog below.

The old plan labels v1/v2/v3 are replaced by real app versions. Version convention:
while the app is Preview, every delivery bumps the patch number (0.1.9 -> 0.1.10);
a minor bump (0.2.0) marks a capability jump; 1.0.0 is a product decision, not a
calendar one. Version numbers never repeat or go down (updater compares them).

## Version plan (agreed 2026-11-12)

- 0.1.10: what is already on main, unreleased: Settings dialog with Appearance, Shell
  home, About and Updates (#135), Preview chip (#133). Publishable now.
- 0.1.11: model and thinking-level selector by existing RPC — SHIPPED 2026-11-13
  on feat/model-selector-0.1.11 (13 feature commits + review corrections; native
  review approved twice, including the design slice; see
  odd/tasks/model-selector-0.1.11.md).
- 0.1.12: SHIPPED 2026-11-13 as a 3-PR chain (#146 context seam, #147 settings reads +
  orchestrator mode + terminals, #148 web panes + context chip + update gate, #150 the
  terminal fix and the Unverified word): read-only profiles view (active profile, real
  fallback chains from `profile-fallbacks.json`, agent table), auth status from the store's
  provider NAMES with a per-provider `auth check --provider X --json --no-refresh`, Updates
  checked automatically at open against the npm registry with the chats-closed gate, the
  organic vs full-delegation switch via `GENTLE_PI_ORCHESTRATOR_MODE` (session-scoped, not
  persisted), a per-chat context chip from RPC `get_session_stats`, and "Open in terminal"
  through Windows Terminal. See odd/tasks/settings-0.1.12.md (L19 closes it).
- 0.1.13 (SHIPPED 2026-11-14): the context chip reads at a glance (#153) and the Updates
  pane's Update extensions action (#154).
- 0.1.14 (SHIPPED 2026-11-14): Close chat in the row menu (the Updates gate is
  satisfiable without quitting), a dot on the gear when pi or the shell is behind, the
  **Usage** section drawing the subscription numbers through `pi-usage-sources`' headless
  JSON mode, terminal windows that wait with a titled tab, and a Settings dialog that no
  longer closes on a press outside.
- 0.2.0 (SHIPPED-ON-BRANCH 2026-11-14, shipped on feat/language-0.2.0 pending release):
  **language** — English and Spanish for the app's own words: the system language on first run
  (any `es` locale → Spanish), an instant switch that needs no restart, and both the web copy and
  the Rust notices/errors/validations drawing from a bilingual catalog; nothing that comes from the
  shell or pi is translated. See odd/tasks/language-0.2.0.md. Not a patch: it is a capability jump,
  which is what a minor bump marks.
- Dropped on 2026-11-14, by the maintainer: a preference for how the terminal opens (Windows
  Terminal already honours the user's own settings; configuring over it is ceremony) and the
  Updates pane's empty state (the "not checked yet" line lives under a second; a failure already
  says why, so a placeholder would only hide a gap nobody sees).
- Profile editing: RESOLVED without a GUI - the Profiles pane's Edit in terminal opens the shell
  and the maintainer types `/gentle:profiles` there. Editing models inside the pane (clicking a
  field) is a luxury, not a need, and is not planned. Status 2026-11-14, corrected by the maintainer: this is the current state, not the end state - in a desktop app, moving work out of the terminal into the interface is product value, so a GUI Profiles/pins editor is now planned (see the 0.2.x adoption plan below).
- RESOLVED by the maintainer (2026-11-14): the subscription usage panel stays terminal-only - the maintainer uses `/gentle:usage` in the shell, no GUI overlay. Separately, gentle-shell's own repo now documents a structured desktop control channel (`docs/desktop-rpc.md`, schema `gentle-shell.desktop-rpc/v1`, activated with `GENTLE_SHELL_INTERACTIVE_HOST=1 pi --mode rpc`) whose advertised methods include `get_subscription_usage`, `refresh_subscription_usage`, `apply_profile`, `list_subagents` and `cancel_subagent`; if a GUI version is ever wanted again, that contract is where the data would come from. Not planned.
- RESOLVED by the maintainer (2026-11-14): profile and subagent editing stays solved through the maintainer's own gentle-shell package (its `/gentle:profiles` and the desktop RPC contract above expose `apply_profile` and subagent list/cancel), not a gentle-desk GUI. Not planned.

## 0.2.x: adopting gentle-shell's desktop RPC channel (agreed 2026-11-14)

- What the channel is: gentle-shell 4.8.0 documents `docs/desktop-rpc.md`, contract
  `gentle-shell.desktop-rpc/v1`, active only with `GENTLE_SHELL_INTERACTIVE_HOST=1 pi
  --mode rpc`. The client sends `/gentle:desktop {json}` as an ordinary prompt and the
  domain answer arrives as an Extension UI widget correlated by `requestId`, so control
  traffic never enters the conversation. gentle-desk does not consume it today
  (verified: no `/gentle:desktop` reference under apps/desktop or crates).
- Adopt what we lack; do not migrate what already works. Nothing here replaces the
  child-process chat transport, and nothing forces adoption.
- Order, each its own slice with a real-window check:
  1. **Language (0.2.0) first**, so new UI is born bilingual instead of rewritten later.
  2. **Live Settings sync**: `get_snapshot` plus the `config_changed` event, so panels
     refresh while the terminal edits, instead of only when the terminal closes.
  3. **Profiles and pins editor**: the largest one, sliced as read+state, model and
     effort per agent, fallback-chain editing, then the per-repo pin. It is safe now
     because `set_config` writes through gentle-pi's own writers and `writeRevision`
     rejects the write when the file changed underneath. Model pickers reuse the
     0.1.11 model catalog.
  4. **Subagents and jobs per chat**: `list_subagents`/`cancel_subagent` and
     `list_jobs`/`cancel_job`, plus the `rate_limit` event. Capability the desk does
     not have at all today.
- Deliberately NOT adopted, because the maintainer keeps these in the terminal by
  decision: the subscription usage panel stays `/gentle:usage` (the channel does expose
  `get_subscription_usage`, so it remains the data source if that decision ever
  changes), and the design canvas `open`/`stop` actions stay the `gentle_design`
  tool's job.
- Before relying on the channel, validate it against our own children with
  `GENTLE_SHELL_INTERACTIVE_HOST` set; it is a new surface and a child that ignores it
  must degrade to today's behaviour, never break.

## 0.2.x: removing a workspace, and deleting its chats (agreed 2026-11-14)

Target after the 0.2.0 language work (new UI must be born bilingual). Nothing here is
built or started; this records the decision and the measurements so it is not
re-argued.

- The premise, corrected: chats do not live in the workspace folder. They live in pi's
  session store (`<agent-dir>/sessions/<encoded-workspace-path>/<timestamp>_<id>.jsonl`,
  here `C:\Users\josep\.pi\agent\sessions`), indexed by the workspace's path. So
  removing a workspace entry cannot delete chats by itself, and deleting the folder from
  disk would only orphan them: still on disk, unreachable from the UI.
- Measured on the maintainer's machine (2026-11-14): 12 chats, 36 MB in total; the three
  largest are 10.8 MB, 6.4 MB and 5.9 MB, all from ONE project; this repo's own
  workspace holds 2 chats, 1.7 MB. A long agent session carries its tool output, so a
  single chat can outweigh a whole workspace list. Size is therefore shown in megabytes
  next to the count, not only a count.
- Decision (maintainer): "si quitas un workspace, debes eliminar los chats ya que eso
  ocupa espacio del usuario" was raised, and the agreed shape is the opt-in checkbox in
  the removal dialog — NOT an automatic consequence. Removing a workspace only forgets
  the bookmark; deleting its chats is a second, explicit, destructive choice with the
  number and the size in front of the user (`[ ] Also delete those 2 chats (1.7 MB)`),
  and the primary button becomes `Remove and delete 2 chats` only when it is checked.
  Rationale: removing a workspace is usually temporary (another drive, another phase),
  and chat history is the one artifact that cannot be regenerated; the app owns
  bookmarks, not the user's conversations.
- Dialog design (maintainer): "mejorar el diseño del modal ya que una card dentro del
  modal no se ve bien". Anatomy: the title is the question (`Remove this workspace?` /
  `Delete this chat?`); one consequence line in the user's words, with the scope in
  bold (`Delete 37 chats`), never the words "chat file"; NO inner card and no boxed
  path; the destructive button is last, red, and labelled with its scope; actions
  right-aligned. Where a path is shown, it is plain monospaced text with the middle
  elided (`C:\…\gentle-desk`), or behind a `Show file paths` disclosure. The workspace
  path IS the identity and belongs in the first sentence; a chat's internal
  `<timestamp>_<id>.jsonl` name is noise and moves to a row-menu `Copy file path`.
- Not a check to touch config: removing a workspace that has an open chat must first
  close that chat's child process and drop its cached session (process-per-chat,
  sessions-cache, refresh-watcher), or the app is left with children pointing at a
  folder no longer in the list. That teardown is the real work of the safe half and
  needs its own spec before any code.
- Open decisions for when it is picked up: whether a bulk delete moves the files to the
  OS trash instead of unlinking them (reversible, and it costs a new `trash`
  dependency, so an explicit maintainer decision), and whether the orphan case (a
  workspace folder deleted outside the app) gets its own Settings view (`Unused chats:
  N chats, M MB — Review`), which is where space reclamation honestly belongs.
- Visual work goes through the gentle-design canvas before `SettingsDialog`/`dialogs`
  change, not by hand in Svelte.

## Old roadmap (superseded by the version plan)

- v1 (now 0.1.10): Appearance only, plus a Ctrl+K palette with gentle-desk's OWN actions
  (new chat, switch workspace, search, theme, open settings). Needs nothing from the shell.
- v2 (now 0.1.11): model and persona. Candidate pieces exist in pi's RPC mode (below).
- v3 (now 0.2.0): orchestrator and subagent profiles.

Decision for now: gentle-desk does not write gentle-pi's JSON files (profiles.json,
models.json, settings.json, ...).

- The v1 **Ctrl+K command palette** (gentle-desk's own actions: new chat, switch workspace, search, theme, open settings) was never built and is DROPPED by the maintainer (2026-11-14). gentle-shell's own TUI already ships a command palette (alt+k) for shell commands, and no gentle-desk user asked for a GUI one.

## What was verified (code reading only; nothing was executed; no real shell was started)

Read: pi 1.5.0 (dist/modes/rpc, dist/core/agent-session.js) and gentle-pi 4.7.0, with the
user's permission for those folders only. Four key claims were re-read by the parent.

- A prompt whose text starts with `/` is routed to a registered extension command first
  (agent-session.js, prompt handling); the response carries `disposition: "handled"`.
  Unknown slash text goes to the model as plain text. pi's own `/model` is a TUI built-in,
  NOT an extension command, so it must not be sent in RPC.
- RPC accepts 33 command types, among them `get_commands`, `get_available_models`,
  `set_model` (session scoped), `set_thinking_level`, `get_state`. gentle-desk models only
  7 today.
- `ctx.ui.custom()` is a no-op stub in RPC mode. `select`, `confirm`, `input`, `editor`,
  `notify`, `setStatus` and string widgets work. gentle-desk already answers dialogs.
- `/gentle:models` is retired; the command is `/gentle:profiles`. Bare, it needs a TUI
  screen. Text subcommands: export, import, publish, export-routing, restore-routing,
  clear. There is no list, show, use, rename, duplicate, delete or edit as text.
- `/gentle:persona` works through a two-option `select` dialog.
- Most Alt+K palette entries are terminal-only or settings files of gentle-pi. YOLO and
  review permissions refuse to be granted outside a TUI, on purpose: gentle-desk must not
  try to drive them.

## v3 options (decision later)

1. Ask gentle-pi for text subcommands (for example `/gentle:profiles use <name>`). Keeps
   "the terminal does the work"; it is a change in another project.
2. gentle-desk writes the files itself. Reading is easy (kind/version envelopes). Writing
   needs validation, atomic replace and the shell's multi-file order with rollback, or the
   active-profile state desyncs.
Recommendation at the time: option 1.

## Verified 2026-11-12 (code reading of pi 1.5.2, gentle-pi 4.7.0, gentle-desk crates;
user-authorized reads outside the project; nothing executed against a live pi)

- Auth: /login and /logout are TUI built-ins (interactive-mode.js:2741-2748), not
  extension commands. No RPC type among the 33 is auth-related (rpc-types.d.ts). Sent as
  prompt text, /login goes to the model. `gentle-shell auth login|logout|check` forwards
  pi's auth subcommands (launcher PI_SUBCOMMANDS), but `pi auth login` refuses a
  non-interactive terminal (auth-login.js:35-38); `auth check --json` and
  `print-api-key` have no such gate, so they are usable as subprocesses.
- auth.json: `<agent-dir>/auth.json`, keyed by provider id; each entry is
  {type:"api_key", key?, env?} or {type:"oauth", access, refresh, expires}. A RUNNING pi
  re-reads it per request via a revision check (auth-storage.js readLatestData), so an
  external login is picked up on the next turn. What does NOT refresh in a running child:
  the model list and selection snapshot (get_available_models and set_model read a
  snapshot built at create time; no RPC command triggers modelRuntime.refresh), so after
  a login the child shows the new provider only after a restart.
- Which models appear: only providers whose auth resolves (auth.json credential, runtime
  key, or env var key; model-runtime.js:448-458). set_model needs {provider, modelId} and
  errors "Model not found: <provider>/<modelId>"; response data is the Model object.
  get_available_models returns {models: Model[]} with fields id, name, provider, api,
  reasoning, thinkingLevelMap, contextWindow, maxTokens, cost, etc. Thinking levels:
  off|minimal|low|medium|high|xhigh|max; per-model levels via get_available_thinking_levels.
- Home: gentle-desk spawns `gentle-shell --mode rpc` with --link / --isolated (default)
  / --home (crates/backend/src/home.rs). Choice stored in
  %APPDATA%/gentle-desk/config.json under key shell_home ("link"|"isolated"); this
  machine has "link". Settings dialog shows the choice, not the resolved path.
  For same-home terminal commands: `gentle-shell --link auth login` (writes
  ~/.pi/agent/auth.json) or `gentle-shell --isolated auth login` (writes
  ~/.gentle-shell/agent/auth.json); env PI_CODING_AGENT_DIR must stay consistent.
- Profiles: stored as <config-home>/gentle-ai/profiles.json, envelope
  {kind:"gentle-pi.agent_model_profiles", version:1, profiles, active?, workflow?}.
  AgentModelConfig maps role -> {model, thinking, tier?}; the reserved role
  "orchestrator" is the orchestrator route. Active profile: global `active`, plus
  per-repo layers (profile-pin.json in git common dir, .pi/gentle-ai/profile.json in the
  worktree). Config home resolution: GENTLE_PI_CONFIG_HOME > GENTLE_SHELL_CONFIG_HOME >
  ~/.pi/gentle-ai; standalone shell gets ~/.gentle-shell/gentle-ai.
- Organic vs full delegation: ORCHESTRATOR_MODES=["organic","full-delegation"], default
  organic, stored per profile in the sibling `workflow` map of profiles.json. Effective
  mode: env GENTLE_PI_ORCHESTRATOR_MODE > session profile workflow > default. Changed
  only by panel key `w` (ctx.ui.custom, TUI-only); no text subcommand exists
  (toggleProfileOrchestratorMode has no caller in the extension).
- Profile editing: one ctx.ui.custom panel (RoutingTablePanel) does everything: rows for
  orchestrator + all roles; actions use-session, apply-global, save, create, duplicate,
  rename, delete, export/import, pin. Entirely TUI-only. Text subcommands that exist:
  export <name>, import, publish <name>, export-routing, restore-routing (asks a
  ctx.ui.confirm), clear. Applying a profile rewrites models.json, agent frontmatter,
  subagents.json and settings.json, in an order the panel owns.
- gentle-desk composer has NO model chip today: Composer.svelte renders textarea,
  send/stop, disabled reason, images notice, key hints. The "Model High v" chip in the
  screenshot is the gentle-pi TUI routing table. Command path today: RpcCommand
  (8 variants) -> codec.rs -> app intents/input/events -> runtime execute (generic,
  no change needed) -> process.rs writes the line.
- Decision kept: gentle-desk never writes gentle-pi's JSON files, and never writes
  auth.json. "Open in terminal" is the alternate route for TUI-only features.

## Not verified

- What bare /gentle:profiles over RPC does when ctx.ui.custom returns undefined
  (silent cancel vs extension_error). Inferred: the panel cannot work.
- That export/import/export-routing run fully headless via prompt RPC (they call
  runProfilesPanelAction, which may touch the panel).
- Whether a live login in another terminal refreshes get_available_models in the
  running child (code says no, until restart).
- Live RPC round-trip: a one-shot test with GENTLE_SHELL_NO_AUTO_SETUP=1 in a temp
  home was attempted 2026-11-12 and ABORTED: first-run auto-setup installed companion
  packages, upgraded the managed engram binary and reordered the user PATH outside the
  temp folder; no RPC response was obtained (timeout during setup). Do not rerun without
  GENTLE_SHELL_NO_AUTO_SETUP=1, user-approved exact commands, and a longer timeout.
- The exact text of the shell's status notices (parsing them would need it).
- ctx.hasUI under mode "rpc".
