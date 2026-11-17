# helper-stop: stop one running helper from its thread bar

The roadmap's "0.4.x: stopping a helper" (agreed 2026-11-16, maintainer:
"agenda helper Stop como feature propio"), next in the session queue after the
0.4.0 release. First real consumer of the /gentle:desktop transport (rpc-settings-sync T2) using each chat's own child.

## Specs (verbatim quotes)

- S1 (roadmap): "stop ONE running helper (subagent) without touching the chat or
  its other helpers."
- S2 (roadmap, what it needs): "a new intent (`StopHelper`) through the contract,
  the app-side execution that stops that helper's work and marks it
  `Ended`/`Cancelled` while the chat keeps running, and the helper thread bar's
  red square (the artboard's own `i-stop`), enabled only while that helper is
  running."
- S3 (roadmap correction 2026-11-16, T1's real probe of gentle-shell 4.8.0):
  "the desktop RPC channel DOES offer the capability - `cancel_subagent` and
  `cancel_job` are listed in `capabilities` - so this feature is built on the
  channel's transport (rpc-settings-sync T2), not on a backend that does not
  exist; only the app's own contract lacked it."
- S4 (roadmap): "Bilingual copy (`Stop` / `Detener`). Capability jump: minor bump
  when it ships."
- S5 (session queue, ordering): "Artboard primero (el cuadrado rojo en la barra
  del hilo del helper)."
- S6 (rpc-settings-sync L5 decision): "per-chat features (subagents, jobs,
  helper Stop) use each chat's own child."
- S7 (standing rules, rpc-settings-sync S4): "the maintainer approves any
  real-child command before it runs"; new copy bilingual; degrade silently (never
  break) when the channel is absent or the child does not answer.
- S8 (rpc-settings-sync L3 advisory, carried): helper/desktop requests must use
  ONE requestId counter per chat (ids are only unique per client today).

## Tasks

- [x] T1 (S5) artboard: the red square (`i-stop`) in the helper thread bar of
      design/gentle-desk-v2/01b-tree-helpers.html, right side of `ht-bar`,
      enabled only while that helper is running (the data already carries
      `stop:true`/`stop:false`); edit that artboard only, then open the canvas.
- [x] T2 (S2, S3) contract: `StopHelper` intent through the contract (Rust
      `contract.rs` + TS `bindings.ts`), mapped in `intent.rs`, reducer answers
      for a helper that is not running refused/ignored per the state's rules.
- [x] T3 (S2, S3, S6, S7, S8) app side: execution sends `cancel_subagent` through
      the foreground chat's own child (the T2 `DesktopClient`/`DesktopHub`
      transport), marks the helper `Ended`/`Cancelled`, the chat keeps running;
      silent degradation when the channel is inactive or the child times out
      (today's behaviour: no stop possible). TDD where testable.
- [x] T4 (S2, S4) web: the helper thread bar's stop button sends the intent,
      enabled only while that helper is running; bilingual copy Stop/Detener.
      TDD.
- [ ] T5 (close) battery + real-window check (stop a real helper, the chat keeps
      running) + roadmap note. Version per the maintainer.

## Log

- L1 (maintainer, session queue 2026-11-16): helper Stop is queue item 3; "el
  canal de gentle-shell ofrece cancel_subagent y cancel_job (confirmado en el
  probe real); usa el hijo de cada chat y es el primer consumidor real del
  transporte /gentle:desktop ya construido (gentle-core rpc::desktop + backend
  chat_host::desktop). Spec en el roadmap, sección "0.4.x: stopping a helper".
  Artboard primero (el cuadrado rojo en la barra del hilo del helper)."
- L2 (parent, 2026-11-17): T1 delivered — the stop button lives in the helper
  thread bar's right side (after the h2, `tip-r` so the tooltip clears the
  right edge): a 32×32 `--danger` icon button whose hover band fills red (the
  #175 hover style's rule: the hover is the fill), `#i-stop` at 14px, disabled
  (faint, no hover) whenever the helper's data says `stop:false`, with the tip
  reading "Stopped" and the aria-label naming the helper once stopped. Clicking
  it on the artboard flips that helper to `ended` in the list and the bar, so
  the direction shows the outcome: one helper stops, the chat keeps running.
  Canvas opened at the maintainer's URL; only 01b-tree-helpers.html edited.
- L3 (parent, 2026-11-17): T2, T3 and T4 delivered on `feat/helper-stop` in a
  SEPARATE worktree (`C:/Users/josep/dev/gentle-desk-helper-stop`), because the
  main checkout holds another session's uncommitted `chat-concurrency-limits`
  work that this feature must not touch. Slices (each compiles):
  ff7095f core transport (`cancel_subagent`, params exactly `{id}`, the typed
  outcomes and the capability gate), 69c5248 app reducer + runtime execution
  (`Intent::StopHelper` → one `ChatCommand::StopHelper` for the foreground chat's
  RUNNING helper only; per-chat `DesktopClient` cache = S8; `Input::HelperStopped`
  marks that one task `Cancelled`), b60a759 desktop bridge (contract, `intent.rs`,
  effects, bindings regenerated), 117daeb web (the bar's stop square + bilingual
  copy), 9b5b8a4 runtime tests for S8 (RED observed: removing `forget_stopper`
  fails the reopen test and only it). The maintainer approved OPTION A (the stop
  is a runtime command, reusable by the iced shell) after the writer stopped
  before editing to ask; `cancel_subagent`'s params shape was confirmed from
  gentle-pi's `docs/desktop-rpc.md` ("Params are exactly `{ id }`"). Helper ids
  verified equal to `list_subagents`' ids (same `TaskRecord.id`). Verification by
  the parent: cargo workspace green, clippy -D warnings clean, fmt clean, bindings
  drift guard byte-identical, vitest 738/738, svelte-check 0/0, eslint clean.
  DELIVERY: ONE PR — the capability is atomic (transport, reducer, bridge and
  button are dead halves without each other), so the slices are commit-level
  review units, not separate PRs.
- L4 (parent, 2026-11-17): T5's real-window check is the only end-to-end proof
  left; S7 forbids real children without the maintainer's per-run approval, so
  it waits for an approved session (stop a real helper, the chat keeps running,
  the counters and the card correct on the child's next frame).
- L5 (parent, 2026-11-17): T2-T4 REVIEWED AND APPROVED — lineage
  review-471c99ffe74e14b9 (risk + resilience + reliability), 32 paths, 1773
  lines, tier high, correction budget 200 (unused). Three model runs, ~$0.068
  total, APPROVED, authority burned. Delivered as PR #179. FOUR ADVISORY
  (non-blocking) findings, recorded here as BACKLOG — the review closed on this
  candidate and none of them reopens it:
  (1) R3-requestid-reset-on-existing-open — the OpenProcess arm drops the stop
  client BEFORE `open` reveals whether it started a new child, so re-clicking an
  already-open chat (Opened::Existing) discards a live client's counter and
  capabilities; the next stop restarts at `desktop-1`, which is S8's "one counter
  per chat" broken for that path. Small fix (forget only when a fresh child
  started), needs its own candidate. HIGHEST of the four.
  (2) R3-stop-client-cache-not-pruned-on-close — closing a chat keeps its client
  in the map for the process lifetime.
  (3) R4-1 — a chat's first stop is two sequential blocking exchanges
  (capabilities, then cancel) under that chat's Stopper mutex, so a stalling
  child holds the stop for up to two deadlines and a second stop of the same chat
  queues behind it; silent by S7.
  (4) R4-2 — `helper_stopped` overwrites the mark unconditionally, so a terminal
  status the child reported between the click and the accepted cancel is
  replaced by Cancelled (display-only; the child did accept the cancel).
