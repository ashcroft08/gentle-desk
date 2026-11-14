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
- 0.1.13 (candidates): a Settings preference for how the terminal opens (new window,
  tab, maximized - today Windows Terminal decides), the Updates pane's empty state (a
  quiet versions row while nothing has been checked), and the subscription usage panel
  once gentle-pi exposes it (`get_usage` RPC or `/gentle:usage --json`; today the numbers
  live only in the child's memory, so the desk cannot read them).
- 0.2.0 (optional, user decision): profile and subagent editing in the GUI. Blocked on
  gentle-pi adding text subcommands such as `/gentle:profiles use <name>` and
  `/gentle:profiles mode <name> <mode>`.

## Old roadmap (superseded by the version plan)

- v1 (now 0.1.10): Appearance only, plus a Ctrl+K palette with gentle-desk's OWN actions
  (new chat, switch workspace, search, theme, open settings). Needs nothing from the shell.
- v2 (now 0.1.11): model and persona. Candidate pieces exist in pi's RPC mode (below).
- v3 (now 0.2.0): orchestrator and subagent profiles.

Decision for now: gentle-desk does not write gentle-pi's JSON files (profiles.json,
models.json, settings.json, ...).

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
