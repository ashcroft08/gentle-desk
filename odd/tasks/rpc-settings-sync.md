# rpc-settings-sync: live Settings sync over gentle-shell's desktop RPC channel

The roadmap's step 2 of the desktop RPC adoption ("adopting gentle-shell's desktop
RPC channel, agreed 2026-11-14"): "`get_snapshot` plus the `config_changed` event,
so panels refresh while the terminal edits, instead of only when the terminal
closes." After the language step (0.2.0, done) and before the profiles/pins
editor and the subagents/jobs slices.

## Specs (the roadmap's words, verbatim)

- S1 (adopt what we lack): "Adopt what we lack; do not migrate what already works.
  Nothing here replaces the child-process chat transport, and nothing forces
  adoption." The channel: "gentle-shell 4.8.0 documents `docs/desktop-rpc.md`,
  contract `gentle-shell.desktop-rpc/v1`, active only with
  `GENTLE_SHELL_INTERACTIVE_HOST=1 pi --mode rpc`. The client sends
  `/gentle:desktop {json}` as an ordinary prompt and the domain answer arrives as
  an Extension UI widget correlated by `requestId`, so control traffic never
  enters the conversation. gentle-desk does not consume it today (verified: no
  `/gentle:desktop` reference under apps/desktop or crates)."
- S2 (the slice's goal, RE-SCOPED in L2): the original wording — "`get_snapshot` plus
  the `config_changed` event, so panels refresh while the terminal edits, instead
  of only when the terminal closes" — assumed the channel emits events for
  terminal edits. The spec (desktop-rpc.md of gentle-pi 4.8.0) says otherwise:
  `config_changed` fires only for Desktop-made changes ("a change made by
  `/gentle:profiles`, by the installer, or by another `pi` process becomes
  visible on the next read and emits nothing"), and "No watcher. Nothing is
  pushed except this slice's own `config_changed`." The re-scoped goal: a REAL
  sync engine, event-ready — connect, `get_snapshot`, track sequence and store
  revisions, poll cheaply, re-snapshot on a sequence gap (the spec's own rule),
  refresh the Profiles pane when a tracked store's revision changes via
  `get_config` — so when gentle-shell later emits events for external changes,
  the SAME engine consumes them unchanged. The focus-refresh stays as fallback.
- S3 (prerequisite validation, roadmap): "Before relying on the channel, validate
  it against our own children with `GENTLE_SHELL_INTERACTIVE_HOST` set; it is a
  new surface and a child that ignores it must degrade to today's behaviour,
  never break." Prior probe evidence (roadmap tail): a temp-folder probe got "no
  RPC response (timeout during setup)" — any rerun needs
  `GENTLE_SHELL_NO_AUTO_SETUP=1`, user-approved exact commands, and a longer
  timeout.
- S4 (standing rules): new copy bilingual (S7); the settings panels' current
  refresh path (refresh-watcher m16, sessions refresh) stays for what the channel
  does not carry; the maintainer approves any real-child command before it runs.

## Tasks

- [ ] T1 (S3) validate the channel with the maintainer: one approved command set
      against a real child (`GENTLE_SHELL_INTERACTIVE_HOST=1`), `get_snapshot`
      in, widget out, `config_changed` observed on an edit. Evidence recorded in
      this doc's Log. Blocked until the maintainer approves the exact commands.
- [ ] T2 (S1, S2) transport: the app-side plumbing to send `/gentle:desktop
      {json}`, correlate the `requestId` widget answer, and degrade silently
      (and to today's behaviour) when the child does not answer or the env is
      absent. TDD where testable; the seam is a fake-able transport.
- [ ] T3 (S2) live sync: Settings panels read the snapshot and refresh on
      `config_changed` while the terminal edits; the terminal-close refresh
      stays as the fallback. TDD.
- [ ] T4 (close) battery + real-window check (edit in the terminal, watch the
      panel move) + roadmap note + version per the maintainer.

## Log

- L1 (maintainer, 2026-11-16): "dale" starts the queue's step 2 — the RPC
  channel's live Settings sync. Scope is THIS slice only; the profiles/pins
  editor and subagents/jobs stay queued behind it.
- L2 (maintainer, 2026-11-16, after the explorer's finding): the explorer read
  the channel spec (desktop-rpc.md, gentle-pi@4.8.0) and found the slice's
  premise contradicted: terminal edits emit nothing, there is no watcher, and
  the channel touches pi-side state only (profiles/profile-fallbacks,
  permission mode) — never the app's own config.json. Presented three paths;
  the maintainer ruled: "No quiero la opción más fácil de implementar, quiero
  implementar la mejor aunque sea larga" — the re-scoped S2 sync engine above,
  plus an upstream issue asking gentle-shell for events on external changes
  (filed after T1's probe gives the evidence).
- L3 (parent, 2026-11-16): T2 delivered — the `/gentle:desktop` transport
  (`gentle-core::rpc::desktop`: validated `DesktopRequest`, typed answer parsing of
  the `gentle-desktop-rpc` widget, `DesktopChannel` seam, `DesktopClient` that asks
  `capabilities` first and only returns typed outcomes; `backend::chat_host::desktop`:
  `DesktopHub`, `HostDesktopChannel`, `Spawner::interactive()`), commit 5e05949
  (core 191/0, backend 251/0, clippy, fmt; the reducer untouched — the widget stays
  dropped from the conversation, pinned by tests). Native review, lineage
  review-82bdbec7f06cf28d (resilience + reliability, 15 paths, 1.9k lines, tier
  medium): the first reliability output was malformed JSON (101 KB) and refused at
  admission; a second run produced one CRITICAL, R3-1, corroborated by the refuter
  — the exchange deadline wrapped only the wait, so a child that stops reading its
  stdin would hang the caller in `ChatHost::send` instead of degrading to Timeout.
  One bounded correction (78 diff lines, plan 80), commit ab4a480: the write now
  runs on a detached short-lived thread inside the SAME deadline, the hub holds no
  waiter on timeout, RED observed (the new test failed at 0.50s). Targeted
  validation passed, APPROVED, authority burned. Cost: ~$0.093 across 5 model
  runs; the parent ran the refuter once beyond what the maintainer had authorized
  (the run facade proceeded on its own) — acknowledged to the maintainer, no
  further runs were launched without a fresh OK. Advisory (non-blocking, carried):
  R4-hub-request-id-collision → T3 must use ONE requestId counter per chat (ids are
  only unique per client today); R4-silent-degradation-no-trace → add a tracing
  point on Timeout/Inactive/Closed when a diagnostics channel exists.
- L4 (maintainer + parent, 2026-11-16): T1 DONE — the real-child probe, run with the
  maintainer's explicit approval of the exact command ("Apruebo el probe tal cual").
  Sandbox: a throwaway `--home` and `GENTLE_PI_CONFIG_HOME` under %TEMP%,
  `GENTLE_SHELL_INTERACTIVE_HOST=1`, `GENTLE_SHELL_NO_AUTO_SETUP=1` (the setup that
  contaminated the 2026-11-12 attempt), node + gentle-shell.mjs `--mode rpc`, two
  prompt lines (`capabilities`, then `get_snapshot` 8 s later). Result in ~36 s, no
  stray processes, the temp dirs and script removed afterwards. Findings:
  (1) the REAL envelope is `{schema, kind:"response", requestId, ok, data}` —
  exactly what the client parses; the doc's bare examples are its own
  inconsistency; the writer's open doubt is closed with evidence;
  (2) `capabilities.data` lists methods [capabilities, get_snapshot, get_config,
  set_config, get_permission_mode, set_permission_mode, get_subscription_usage,
  refresh_subscription_usage, get_design_canvas, list_jobs, cancel_job,
  apply_profile, list_subagents, cancel_subagent] and events [config_changed,
  permission_mode_changed, rate_limit, design_canvas, jobs_update, profile_applied,
  agents_update];
  (3) `get_snapshot` is `{sequence, state:{config:{stores:[...]}}}` with six stores
  (profiles, models[ro], profile_fallbacks, subagents[ro], pins, managed_assets[ro]),
  each with scope/readOnly/status/revision, revisions `sha256:` + 32 hex (NOT 64);
  (4) the command was registered before the first prompt arrived (`session_start`
  precedes stdin), so the 2026-11-12 timeout was the auto-setup, not the channel.
  The two real frames are pinned as `desktop-rpc-real.jsonl` with parsing and
  client-replay tests, commit 7d5cf67 (core 194/0 after the additions; the tests
  passed at once — no RED is invented for a passive pin of an already-correct
  parser). NOTE for the queue: `cancel_subagent` and `cancel_job` EXIST in the
  channel — the earlier roadmap claim that helper Stop has no backend was true only
  of the app's own contract (chat-level Abort); the feature can be built on the
  channel.
- L5 (maintainer, 2026-11-16): the T3 architecture question — which process answers
  the sync's snapshot, since the channel belongs to a pi child and the stores
  (profiles, pins) are global. Options: (1) reuse an open chat's child, (2) a
  temporary control child alive only while the Settings dialog is open, (3) a
  permanent control child. The maintainer asked for the parent's recommendation and
  accepted it: OPTION 2, "con spike primero". Rationale: sync and the next slice
  (the profiles/pins editor with `set_config`) both live in the Settings dialog;
  no permanent process; works with no chat open; per-chat features (subagents,
  jobs, helper Stop) use each chat's own child. The engine is built against a
  swappable channel provider so the choice can change without a rewrite. The spike
  comes FIRST and runs only in a throwaway sandbox home: (a) spawn-to-answer
  latency of a cold child, (b) the tree's memory once ready, (c) whether answering
  `/gentle:desktop` prompts creates any session file (the control child must never
  dirty the maintainer's chat history).
