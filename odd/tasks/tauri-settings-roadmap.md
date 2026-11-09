# tauri-settings-roadmap: settings by versions (decided 2026-11-09)

The user wants settings to grow in steps, with the GUI asking and the gentle-shell
tooling doing the real work. This file records the roadmap and what was verified, so it
is not investigated twice. Nothing here is built yet.

## Roadmap

- v1: Appearance only (the theme moves out of the main screen once a Settings screen
  exists), plus a Ctrl+K palette with gentle-desk's OWN actions (new chat, switch
  workspace, search, theme, open settings). Needs nothing from the shell.
- v2: model and persona. Candidate pieces exist in pi's RPC mode (below). Decide its scope
  when v1 runs in a real window.
- v3: orchestrator and subagent profiles. Blocked on a design choice (below).

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

## Not verified

- That bare `/gentle:profiles` fails with an `extension_error` in RPC (inferred).
- The contents of the user's real `~/.pi` files, and `ctx.hasUI` under `mode: "rpc"`.
- The exact text of the shell's status notices (parsing them would need it).
